use crate::translator::{CommitMessage, ai_service};
use dialoguer::Confirm;
use log::{debug, info};
use std::path::Path;
use textwrap::fill;

const MAX_LINE_LENGTH: usize = 72;

fn is_auto_generated_commit(title: &str) -> bool {
    let patterns = ["Merge", "Cherry-pick", "Revert"];
    patterns.iter().any(|pattern| title.starts_with(pattern))
}

pub async fn process_commit_msg(path: &Path) -> anyhow::Result<()> {
    debug!("开始处理提交消息: {}", path.display());
    let content = std::fs::read_to_string(path)?;
    let msg = CommitMessage::parse(&content);
    
    // 检查是否是自动生成的提交消息
    if is_auto_generated_commit(&msg.title) {
        debug!("检测到自动生成的提交消息，跳过翻译");
        return Ok(());
    }

    if !contains_chinese(&msg.title) {
        debug!("未检测到中文内容，跳过翻译");
        return Ok(());
    }

    info!("检测到中文内容，准备翻译");

    if !Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("检测到提交信息包含中文，是否需要翻译？")
        .default(true)
        .interact()? {
        return Ok(());
    }

    let config = crate::config::Config::load()?;
    info!("开始翻译流程，默认使用 {:?} 服务", config.default_service);
    
    // 翻译标题
    let en_title = ai_service::translate_with_fallback(&config, &msg.title).await?;
    let en_title = wrap_text(&en_title, MAX_LINE_LENGTH);
    let original_title = msg.title.clone();

    // 翻译正文（如果有的话）
    let (en_body, cn_body) = if let Some(body) = &msg.body {
        let en_body = ai_service::translate_with_fallback(&config, body).await?;
        (Some(wrap_text(&en_body, MAX_LINE_LENGTH)), Some(body.clone()))
    } else {
        (None, None)
    };

    // 构建新的消息结构
    let new_msg = CommitMessage {
        title: en_title,
        body: Some(format_body(en_body.as_deref(), &original_title, cn_body.as_deref(), &msg.marks)),
        marks: vec![],  // 标记已经包含在 body 中了
    };

    info!("翻译完成，正在写入文件");
    std::fs::write(path, new_msg.format())?;
    info!("处理完成");
    Ok(())
}

fn format_body(en_body: Option<&str>, cn_title: &str, cn_body: Option<&str>, marks: &[String]) -> String {
    let mut parts = Vec::new();

    // 1. 英文正文
    if let Some(body) = en_body {
        parts.push(body.to_string());
        parts.push(String::new());  // 空行分隔
    }

    // 2. 中文标题
    parts.push(cn_title.to_string());

    // 3. 中文正文
    if let Some(body) = cn_body {
        parts.push(String::new());  // 空行分隔
        parts.push(body.to_string());
    }

    // 4. 其他标记
    if !marks.is_empty() {
        parts.push(String::new());  // 空行分隔
        parts.extend(marks.iter().cloned());
    }

    parts.join("\n")
}

fn contains_chinese(text: &str) -> bool {
    text.chars().any(|c| c as u32 >= 0x4E00 && c as u32 <= 0x9FFF)
}

fn wrap_text(text: &str, max_length: usize) -> String {
    fill(text, max_length)
}
