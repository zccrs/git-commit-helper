use crate::config;
use crate::git;
use crate::translator::ai_service;
use anyhow::Result;
use dialoguer::{Input, Confirm};
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
    let translator = ai_service::create_translator_for_service(service)?;
    
    println!("\n正在生成提交信息建议...");
    let mut message = translator.translate(&prompt).await?;

    // 如果提供了具体的type，确保使用该type
    if let Some(t) = commit_type {
        message = ensure_commit_type(&message, &t);
    }

    let title = message.lines().next().unwrap_or_default().to_string();
    let body = message.lines().skip(2).collect::<Vec<_>>().join("\n");

    // 保存为临时提交信息文件
    let temp_file = std::env::temp_dir().join("COMMIT_EDITMSG");
    let mut content = if body.is_empty() {
        title
    } else {
        format!("{}\n\n{}", title, body)
    };
    
    // 预览生成的提交信息
    println!("\n生成的提交信息预览:");
    println!("----------------------------------------");
    println!("{}", content);
    println!("----------------------------------------");

    // 询问用户是否需要翻译
    if Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("是否需要翻译为中英双语格式？")
        .default(true)
        .interact()? 
    {
        std::fs::write(&temp_file, &content)?;
        git::process_commit_msg(&temp_file).await?;
        content = std::fs::read_to_string(&temp_file)?;
        
        println!("\n翻译后的提交信息预览:");
        println!("----------------------------------------");
        println!("{}", content);
        println!("----------------------------------------");
    }

    // 询问用户是否确认提交
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

fn get_staged_diff() -> Result<String> {
    let output = Command::new("git")
        .args(["diff", "--cached", "--no-prefix"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("执行 git diff 命令失败"));
    }

    Ok(String::from_utf8(output.stdout)?)
}

fn ensure_commit_type(message: &str, required_type: &str) -> String {
    let first_line = message.lines().next().unwrap_or_default();
    
    if let Some(colon_pos) = first_line.find(':') {
        let current_type = first_line[..colon_pos].trim();
        if current_type != required_type {
            return format!("{}: {}", 
                required_type, 
                first_line[colon_pos + 1..].trim()
            ) + &message[first_line.len()..];
        }
    } else {
        return format!("{}: {}", required_type, first_line) + &message[first_line.len()..];
    }
    
    message.to_string()
}
