use crate::config;
use crate::git;  // 添加这个导入
use crate::translator::ai_service;
use anyhow::Result;
use dialoguer::{Confirm};
use log::{debug, info};
use std::process::Command;

pub async fn generate_commit_message(commit_type: Option<String>) -> Result<()> {
    let diff = get_staged_diff()?;
    if diff.is_empty() {
        return Err(anyhow::anyhow!("没有已暂存的改动，请先使用 git add 添加改动"));
    }

    let prompt = format!(
        "我将给你展示一些 git diff 的内容，请你帮我总结这些改动并生成一个符合规范的 git commit 信息。\
        提交信息的格式要求：\
        1. 第一行为标题，简要说明改动内容\
        2. 标题要精简，不超过50个字符\
        3. 标题的格式为：type: message，其中type为改动类型，message为改动说明\
        4. 如果提供了具体的type则必须使用该type\
        5. 如果没有提供type，则根据改动内容自行判断使用以下类型之一：\
           feat: 新功能\
           fix: 修复问题\
           docs: 文档变更\
           style: 代码格式调整\
           refactor: 代码重构\
           test: 测试相关\
           chore: 构建或辅助工具变更\
        \n\n以下是改动内容：\n{}", 
        diff
    );

    debug!("生成的提示信息：\n{}", prompt);

    let config = config::Config::load()?;
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

    if !status.success() {
        return Err(anyhow::anyhow!("git commit 命令执行失败"));
    }

    println!("提交成功！");
    Ok(())
}

#[allow(dead_code)]
pub async fn generate_commit_suggestion(commit_types: &[String]) -> anyhow::Result<String> {
    let config = crate::config::Config::load()?;
    let service = config.services.iter()
        .find(|s| s.service == config.default_service)
        .ok_or_else(|| anyhow::anyhow!("找不到默认服务的配置"))?;

    let translator = ai_service::create_translator_for_service(service).await?;
    let prompt = get_staged_diff()?;

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

fn get_staged_diff() -> Result<String> {
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
