use crate::translator::{CommitMessage, ai_service};
use dialoguer::Confirm;
use log::{debug, info};
use std::path::Path;
use textwrap::fill;

const MAX_LINE_LENGTH: usize = 72;

pub async fn process_commit_msg(path: &Path) -> anyhow::Result<()> {
    debug!("开始处理提交消息: {}", path.display());
    let content = std::fs::read_to_string(path)?;
    let msg = CommitMessage::parse(&content);
    
    if !contains_chinese(&msg.title) {
        debug!("未检测到中文内容，跳过翻译");
        return Ok(());
    }

    info!("检测到中文内容，准备翻译");

    if !Confirm::new()
        .with_prompt("检测到提交信息包含中文，是否需要翻译？")
        .default(true)
        .interact()? {
        return Ok(());
    }

    let config = crate::config::Config::load()?;
    info!("开始翻译流程，默认使用 {:?} 服务", config.default_service);
    
    let en_title = ai_service::translate_with_fallback(&config, &msg.title).await?;
    let en_title = wrap_text(&en_title, MAX_LINE_LENGTH);

    let body = if let Some(body) = &msg.body {
        let en_body = ai_service::translate_with_fallback(&config, body).await?;
        Some(format!("{}\n\n{}", 
            wrap_text(&en_body, MAX_LINE_LENGTH),
            wrap_text(body, MAX_LINE_LENGTH)))
    } else {
        None
    };

    let new_msg = CommitMessage {
        title: en_title,
        body,
        marks: msg.marks,
    };

    info!("翻译完成，正在写入文件");
    std::fs::write(path, new_msg.format())?;
    info!("处理完成");
    Ok(())
}

fn contains_chinese(text: &str) -> bool {
    text.chars().any(|c| c as u32 >= 0x4E00 && c as u32 <= 0x9FFF)
}

fn wrap_text(text: &str, max_length: usize) -> String {
    fill(text, max_length)
}
