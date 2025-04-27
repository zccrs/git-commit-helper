use regex::Regex;
use crate::ai_service;
use crate::config;

pub struct CommitMessage {
    pub title: String,
    pub body: Option<String>,
    pub marks: Vec<String>,
}

impl CommitMessage {
    pub fn parse(content: &str) -> Self {
        let mark_regex = Regex::new(r"^[a-zA-Z-]+:\s*.+$").unwrap();
        let comment_regex = Regex::new(r"^#.*$").unwrap();
        let mut lines = content.lines().peekable();

        // 获取第一个非注释行作为标题
        let title = lines
            .by_ref()
            .find(|line| !comment_regex.is_match(line.trim()))
            .unwrap_or("")
            .to_string();

        let mut body = Vec::new();
        let mut marks = Vec::new();
        let mut is_body = false;

        while let Some(line) = lines.next() {
            // 跳过注释行
            if comment_regex.is_match(line.trim()) {
                continue;
            }

            if line.trim().is_empty() {
                if !is_body && body.is_empty() {
                    continue;
                }
                is_body = true;
                body.push(line.to_string());
            } else if mark_regex.is_match(line) {
                marks.push(line.to_string());
            } else {
                is_body = true;
                body.push(line.to_string());
            }
        }

        // 移除body末尾的空行
        while body.last().map_or(false, |line| line.trim().is_empty()) {
            body.pop();
        }

        // 移除 body 中的注释行
        let body = if body.is_empty() {
            None
        } else {
            Some(body
                .into_iter()
                .filter(|line| !comment_regex.is_match(line.trim()))
                .collect::<Vec<_>>()
                .join("\n"))
        };

        CommitMessage {
            title,
            body,
            marks,
        }
    }

    pub fn format(&self) -> String {
        let mut result = Vec::new();
        result.push(self.title.clone());

        if let Some(body) = &self.body {
            if !body.is_empty() {
                result.push(String::new());
                result.push(body.clone());
            }
        }

        // 添加标记
        if !self.marks.is_empty() {
            if !result.last().map_or(false, |s| s.is_empty()) {
                result.push(String::new());  // 添加空行分隔
            }
            result.extend(self.marks.clone());
        }

        result.join("\n")
    }
}

use crate::git;
use crate::review;
use dialoguer::Confirm;
use log::{debug, info};
use std::process::Command;

pub async fn generate_commit_message(
    commit_type: Option<String>,
    message: Option<String>,
    auto_add: bool,
    no_review: bool,
    no_translate: bool,
) -> anyhow::Result<()> {
    // 如果指定了 -a 参数，先执行 git add -u
    if auto_add {
        info!("自动添加已修改的文件...");
        let status = Command::new("git")
            .args(["add", "-u"])
            .status()?;

        if !status.success() {
            return Err(anyhow::anyhow!("执行 git add -u 命令失败"));
        }
    }

    // 设置不翻译的环境变量
    if no_translate {
        std::env::set_var("GIT_COMMIT_HELPER_NO_TRANSLATE", "1");
    }

    let diff = get_staged_diff()?;
    if diff.is_empty() {
        return Err(anyhow::anyhow!("没有已暂存的改动，请先使用 git add 添加改动"));
    }

    let config = config::Config::load()?;

    // 在确认有暂存的改动后执行代码审查
    if !no_review && config.ai_review {
        info!("正在进行代码审查...");
        if let Some(review) = review::review_changes(&config, no_review).await? {
            println!("\n{}\n", review);
        }
    }

    // 设置环境变量标记跳过后续的代码审查
    std::env::set_var("GIT_COMMIT_HELPER_SKIP_REVIEW", "1");

    let prompt = match message {
        Some(msg) => format!(
            "I will show you some git diff content and a user description. Please help me generate a standardized git commit message.\
            Please output in plain text format without any markdown or other markup languages.\
            The user description is the focus of this change, while git diff serves as a reference.\
            Commit message format requirements:\
            1. First line is the title, briefly explaining the changes\
            2. Title should be concise, no more than 50 characters\
            3. Title format: type: message, where type is the change type and message is the change description\
            4. If a specific type is provided, it must be used\
            5. If no type is provided, use one of the following types based on the changes:\
               feat: new feature\
               fix: bug fix\
               docs: documentation changes\
               style: code style adjustments\
               refactor: code refactoring\
               test: test related\
               chore: build or auxiliary tool changes\
            Note: Output should only contain plain text commit message, no format markers like markdown.\
            \n\nUser Description:\n{}\n\nChanges:\n{}",
            msg, diff
        ),
        None => format!(
            "I will show you some git diff content. Please summarize these changes and generate a standardized git commit message.\
            Please output in plain text format without any markdown or other markup languages.\
            Commit message format requirements:\
            1. First line is the title, briefly explaining the changes\
            2. Title should be concise, no more than 50 characters\
            3. Title format: type: message, where type is the change type and message is the change description\
            4. If a specific type is provided, it must be used\
            5. If no type is provided, use one of the following types based on the changes:\
               feat: new feature\
               fix: bug fix\
               docs: documentation changes\
               style: code style adjustments\
               refactor: code refactoring\
               test: test related\
               chore: build or auxiliary tool changes\
            Note: Output should only contain plain text commit message, no format markers like markdown.\
            \n\nHere are the changes:\n{}",
            diff
        )
    };

    debug!("生成的提示信息：\n{}", prompt);

    info!("使用 {:?} 服务生成提交信息", config.default_service);
    let service = config.get_default_service()?;
    let translator = ai_service::create_translator_for_service(service).await?;

    println!("\n正在生成提交信息建议...");
    let mut message = translator.translate(&prompt).await?.to_string();

    // 移除各种 AI 返回的元信息标记
    message = message
        .trim_start_matches("[NO_TRANSLATE]")
        .trim_start_matches("、、、plaintext")
        .trim()
        .to_string();

    // 如果提供了具体的type，确保使用该type
    if let Some(t) = commit_type {
        message = ensure_commit_type(&message, &[t]);
    }

    // 处理换行
    let content = message.lines().map(|line| {
        if line.trim().is_empty() {
            line.to_string()
        } else {
            git::wrap_text(line, 72)
        }
    }).collect::<Vec<_>>().join("\n");

    // 预览生成的提交信息
    println!("\n生成的提交信息预览:");
    println!("----------------------------------------");
    println!("{}", content);
    println!("----------------------------------------");

    // 移除翻译相关的询问，直接询问用户是否确认提交
    if !Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("是否使用此提交信息？")
        .default(true)
        .interact()?
    {
        println!("已取消提交");
        return Ok(());
    }

    // 执行git commit
    let status = Command::new("git")
        .current_dir(std::env::current_dir()?)
        .arg("commit")
        .arg("-m")
        .arg(content)
        .status()?;

    // 清理环境变量（无论命令是否执行成功）
    std::env::remove_var("GIT_COMMIT_HELPER_SKIP_REVIEW");
    std::env::remove_var("GIT_COMMIT_HELPER_NO_TRANSLATE");

    if !status.success() {
        return Err(anyhow::anyhow!("git commit 命令执行失败"));
    }

    println!("提交成功！");
    Ok(())
}

#[allow(dead_code)]
pub async fn generate_commit_suggestion(commit_types: &[String], user_description: Option<String>) -> anyhow::Result<String> {
    let config = crate::config::Config::load()?;
    let service = config.services.iter()
        .find(|s| s.service == config.default_service)
        .ok_or_else(|| anyhow::anyhow!("找不到默认服务的配置"))?;

    let translator = ai_service::create_translator_for_service(service).await?;
    let prompt = match user_description {
        Some(desc) => format!("用户描述：\n{}\n\n改动内容：\n{}", desc, get_staged_diff()?),
        None => get_staged_diff()?
    };

    let message = translator.translate(&prompt).await?.to_string();

    // 移除各种 AI 返回的元信息标记
    let message = message
        .trim_start_matches("[NO_TRANSLATE]")
        .trim_start_matches("plaintext")
        .trim()
        .to_string();

    // 如果有指定的提交类型，确保使用这些类型
    if !commit_types.is_empty() {
        return Ok(ensure_commit_type(&message, commit_types));
    }

    Ok(message)
}

fn get_staged_diff() -> anyhow::Result<String> {
    let output = Command::new("git")
        .args(["diff", "--cached", "--no-prefix"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("执行 git diff 命令失败"));
    }

    Ok(String::from_utf8(output.stdout)?)
}

fn ensure_commit_type(message: &str, commit_types: &[String]) -> String {
    let first_line = message.lines().next().unwrap_or_default();

    if let Some(colon_pos) = first_line.find(':') {
        let current_type = first_line[..colon_pos].trim();
        if !commit_types.contains(&current_type.to_string()) {
            return format!("{}: {}",
                &commit_types[0],
                first_line[colon_pos + 1..].trim()
            ) + &message[first_line.len()..];
        }
    } else {
        return format!("{}: {}", &commit_types[0], first_line) + &message[first_line.len()..];
    }

    message.to_string()
}
