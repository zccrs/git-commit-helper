use anyhow::Result;
use std::process::Command;
use crate::config::Config;
use crate::translator::ai_service;
use log::{debug, info};

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

    // 构建提示语
    let prompt = format!(
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

以下是代码变更内容：
{}
"#,
        diff
    );

    // 使用配置的 AI 服务进行代码审查
    let translator = ai_service::create_translator(config).await?;
    info!("正在使用 {:?} 服务进行代码审查...", config.default_service);
    let review = translator.translate(&prompt).await?;

    Ok(Some(review))
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

pub fn should_skip_review(content: &str) -> bool {
    // 跳过合并提交等自动生成的提交
    content.starts_with("Merge") ||
    content.starts_with("Cherry-pick") ||
    content.starts_with("Revert")
}
