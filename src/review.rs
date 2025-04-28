use anyhow::Result;
use std::process::Command;
use crate::config::Config;
use crate::ai_service;
use crate::github;
use crate::gerrit;
use log::{debug, info};

pub async fn review_remote_changes(config: &Config, url: &str) -> Result<String> {
    debug!("开始审查远程代码改动: {}", url);

    // 获取 commit 信息和 diff 内容
    let (commit_message, diff) = if url.contains("github.com") {
        if url.contains("/pull/") {
            // PR 目前不需要显示 commit 信息
            (String::new(), github::get_pr_diff(url).await?)
        } else if url.contains("/commit/") {
            let msg = github::get_commit_info(url).await?;
            let diff = github::get_commit_diff(url).await?;
            (msg, diff)
        } else {
            return Err(anyhow::anyhow!("无效的GitHub URL，必须是PR或commit链接"));
        }
    } else if url.contains("/+/") {
        // Gerrit 链接
        let msg = gerrit::get_change_info(url).await?;
        let diff = gerrit::get_change_diff(url).await?;
        (msg, diff)
    } else {
        return Err(anyhow::anyhow!("无效的URL，必须是GitHub或Gerrit链接"));
    };

    if diff.trim().is_empty() {
        return Err(anyhow::anyhow!("未发现任何代码改动"));
    }

    // 翻译 commit 信息（如果存在且是英文）
    let mut review = String::new();
    if !commit_message.is_empty() {
        let commit_info = if commit_message.chars().all(|c| c.is_ascii() || c.is_whitespace()) {
            // 如果是纯英文，则翻译成中文
            let translator = ai_service::create_translator(config).await?;
            let prompt = "请将以下提交信息翻译成中文：\n\n".to_string() + &commit_message;
            let chinese = translator.chat("你是一个代码提交信息翻译助手。", &prompt).await?;
            format!("提交信息：\n{}\n\n中文翻译：\n{}\n\n", commit_message, chinese)
        } else {
            format!("提交信息：\n{}\n\n", commit_message)
        };
        review.push_str(&commit_info);
    }

    // 代码审查
    let translator = ai_service::create_translator(config).await?;
    info!("正在使用 {:?} 服务进行代码审查...", config.default_service);

    let system_prompt = get_review_prompt();
    let review_result = translator.chat(&system_prompt, &diff).await?;
    review.push_str(&review_result);

    Ok(review)
}

pub async fn review_changes(config: &Config, no_review: bool) -> Result<Option<String>> {
    // 如果命令行指定了 --no-review 或配置文件中禁用了 ai_review，则跳过审查
    if no_review {
        info!("已通过 --no-review 参数禁用代码审查");
        return Ok(None);
    }

    if !config.ai_review {
        info!("AI代码审查功能已在配置中禁用，可以使用 git-commit-helper ai-review --enable 启用");
        return Ok(None);
    }

    // 获取当前改动的差异
    let diff = get_staged_changes()?;
    if diff.trim().is_empty() {
        info!("没有检测到暂存的代码改动");
        return Ok(None);
    }

    // 获取当前改动的差异
    let diff = get_staged_changes()?;
    if diff.trim().is_empty() {
        info!("没有检测到暂存的代码改动");
        return Ok(None);
    }

    // 使用配置的 AI 服务进行代码审查
    let translator = ai_service::create_translator(config).await?;
    info!("正在使用 {:?} 服务进行代码审查...", config.default_service);

    let system_prompt = get_review_prompt();
    let review = translator.chat(&system_prompt, &diff).await?;

    Ok(Some(review))
}

// 构建代码审查提示语
fn get_review_prompt() -> String {
    format!(
        r#"您是一位专业的代码审查者，请对以下代码变更进行审查并给出中文评价。请着重关注：

1. 代码质量：
   - 代码是否清晰易懂
   - 变量和函数命名是否恰当
   - 代码结构是否合理

2. 潜在问题：
   - 可能的bug
   - 边界条件处理
   - 异常情况的处理
   - 资源使用和释放

3. 最佳实践：
   - 是否符合编程规范
   - 是否遵循设计模式
   - 代码重用性
   - 模块化和解耦

4. 性能考虑：
   - 算法效率
   - 资源使用效率
   - 可能的性能瓶颈

5. 安全性：
   - 输入验证
   - 数据安全
   - 权限检查

请以"代码审查报告："开头，使用简洁的语言描述发现的问题和改进建议。如果代码符合最佳实践，也请给出正面的评价。
"#)
}

fn get_staged_changes() -> Result<String> {
    let output = Command::new("git")
        .args(&["diff", "--cached"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("获取暂存区改动失败"));
    }

    Ok(String::from_utf8(output.stdout)?)
}

pub fn should_skip_review(message: &str) -> bool {
    message.starts_with("Merge") ||
    message.starts_with("Cherry-pick") ||
    message.starts_with("Revert")
}
