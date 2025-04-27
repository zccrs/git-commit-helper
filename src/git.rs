use crate::commit::CommitMessage;
use crate::ai_service;
use crate::review;
use dialoguer::Confirm;
use log::{debug, info};
use std::path::Path;
use textwrap::fill;

const MAX_LINE_LENGTH: usize = 72;

fn is_auto_generated_commit(title: &str) -> bool {
    let patterns = ["Merge", "Cherry-pick", "Revert"];
    patterns.iter().any(|pattern| title.starts_with(pattern))
}

pub async fn process_commit_msg(path: &Path, no_review: bool) -> anyhow::Result<()> {
    debug!("开始处理提交消息: {}", path.display());

    // 检查环境变量是否设置了跳过审查
    if std::env::var("GIT_COMMIT_HELPER_SKIP_REVIEW").is_ok() {
        debug!("检测到跳过审查环境变量，跳过代码审查");
        return Ok(());
    }

    let content = std::fs::read_to_string(path)?;
    let msg = CommitMessage::parse(&content);

    // 检查是否是自动生成的提交消息
    if is_auto_generated_commit(&msg.title) {
        debug!("检测到自动生成的提交消息，跳过翻译和审查");
        return Ok(());
    }

    // 获取配置并执行代码审查
    let config = crate::config::Config::load()?;
    if !review::should_skip_review(&msg.title) {
        info!("正在进行代码审查...");
        if let Some(review) = review::review_changes(&config, no_review).await? {
            // 直接在终端显示审查结果
            println!("\n{}\n", review);
        }
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
    let mut body_parts = Vec::new();

    // 添加英文和中文内容
    if let Some(en_body) = en_body {
        body_parts.push(en_body);
        body_parts.push(String::new());
    }

    body_parts.push(original_title);

    if let Some(body) = cn_body {
        body_parts.push(String::new());
        body_parts.push(wrap_text(&body, MAX_LINE_LENGTH));
    }

    let new_msg = CommitMessage {
        title: en_title,
        body: Some(body_parts.join("\n")),
        marks: msg.marks, // 保持原有标记不变
    };

    info!("翻译完成，正在写入文件");
    std::fs::write(path, new_msg.format())?;
    info!("处理完成");
    Ok(())
}

fn contains_chinese(text: &str) -> bool {
    text.chars().any(|c| c as u32 >= 0x4E00 && c as u32 <= 0x9FFF)
}

pub fn wrap_text(text: &str, max_length: usize) -> String {
    fill(text, max_length)
}
