use regex::Regex;
use crate::ai_service;
use crate::config;
use crate::git;

// 语言模式枚举
#[derive(Debug, Clone, Copy)]
enum LanguageMode {
    ChineseOnly,
    EnglishOnly,
    Bilingual,
}

/// 解析 issues 参数并生成相应的引用字段
fn parse_issue_reference(issues: &str) -> anyhow::Result<String> {
    let mut fixes_refs = Vec::new();
    let mut pms_refs = Vec::new();

    // 按空格和逗号分割多个链接
    let links: Vec<&str> = issues.split_whitespace()
        .flat_map(|s| s.split(','))
        .filter(|s| !s.is_empty())
        .collect();

    for link in links {
        let link = link.trim();

        // 处理 GitHub issue URL
        if link.starts_with("https://github.com/") {
            match parse_github_issue(link) {
                Ok(ref_str) => {
                    // 提取 Fixes: 后面的部分
                    if let Some(fix_ref) = ref_str.strip_prefix("Fixes: ") {
                        fixes_refs.push(fix_ref.to_string());
                    }
                }
                Err(e) => return Err(e),
            }
        }
        // 处理 PMS 链接
        else if link.contains("pms.uniontech.com") {
            match parse_pms_link(link) {
                Ok(ref_str) => {
                    // 提取 PMS: 后面的部分
                    if let Some(pms_ref) = ref_str.strip_prefix("PMS: ") {
                        pms_refs.push(pms_ref.to_string());
                    }
                }
                Err(e) => return Err(e),
            }
        }
        // 处理简单的 issue 数字（假设是当前项目的 GitHub issue）
        else if let Ok(_issue_num) = link.parse::<u32>() {
            fixes_refs.push(format!("#{}", link));
        }
        else {
            return Err(anyhow::anyhow!("无法解析 issue 引用格式: {}", link));
        }
    }

    // 组合结果
    let mut result = Vec::new();

    if !fixes_refs.is_empty() {
        result.push(format!("Fixes: {}", fixes_refs.join(" ")));
    }

    if !pms_refs.is_empty() {
        result.push(format!("PMS: {}", pms_refs.join(" ")));
    }

    if result.is_empty() {
        return Err(anyhow::anyhow!("没有找到有效的 issue 引用"));
    }

    Ok(result.join("\n"))
}

/// 解析 GitHub issue URL 并生成 Fixes 字段
fn parse_github_issue(url: &str) -> anyhow::Result<String> {
    let issue_regex = Regex::new(r"https://github\.com/([^/]+/[^/]+)/issues/(\d+)")?;

    if let Some(captures) = issue_regex.captures(url) {
        let repo = captures.get(1).unwrap().as_str();
        let issue_num = captures.get(2).unwrap().as_str();

        // 检查是否是当前项目
        if is_current_project_repo(repo)? {
            Ok(format!("Fixes: #{}", issue_num))
        } else {
            Ok(format!("Fixes: {}#{}", repo, issue_num))
        }
    } else {
        Err(anyhow::anyhow!("无效的 GitHub issue URL 格式: {}", url))
    }
}

/// 解析 PMS 链接并生成 PMS 字段
fn parse_pms_link(url: &str) -> anyhow::Result<String> {
    // 匹配 bug-view、task-view、story-view 格式
    let bug_regex = Regex::new(r"bug-view-(\d+)\.html")?;
    let task_regex = Regex::new(r"task-view-(\d+)\.html")?;
    let story_regex = Regex::new(r"story-view-(\d+)\.html")?;

    if let Some(captures) = bug_regex.captures(url) {
        let id = &captures[1];
        Ok(format!("PMS: BUG-{}", id))
    } else if let Some(captures) = task_regex.captures(url) {
        let id = &captures[1];
        Ok(format!("PMS: TASK-{}", id))
    } else if let Some(captures) = story_regex.captures(url) {
        let id = &captures[1];
        Ok(format!("PMS: STORY-{}", id))
    } else {
        Err(anyhow::anyhow!("无效的 PMS 链接格式: {}", url))
    }
}

/// 检查给定的仓库是否是当前项目的仓库
fn is_current_project_repo(repo: &str) -> anyhow::Result<bool> {
    use std::process::Command;

    // 获取当前仓库的远程 URL
    let output = Command::new("git")
        .args(&["remote", "get-url", "origin"])
        .output()?;

    if !output.status.success() {
        return Ok(false);
    }

    let remote_url = String::from_utf8_lossy(&output.stdout);
    let remote_url = remote_url.trim();

    // 从远程 URL 中提取仓库名称
    // 支持 HTTPS 和 SSH 格式
    let repo_regex = Regex::new(r"github\.com[:/]([^/]+/[^/\.]+)")?;

    if let Some(captures) = repo_regex.captures(remote_url) {
        let current_repo = captures.get(1).unwrap().as_str();
        Ok(current_repo == repo)
    } else {
        Ok(false)
    }
}

// 提示词模板常量
const ENGLISH_PROMPT_TEMPLATE: &str = r#"Please analyze the git diff content and generate a commit message in English only:
1. First line: type: message (under 50 characters)
2. Empty line after the title
3. Detailed explanation in English (what was changed and why)
4. Empty line after explanation
5. Log field (ONLY if this change involves user-facing features/UI changes that product managers would communicate to users)
6. Empty line after Log field (if present)
7. Influence section with black-box testing recommendations
8. Type must be one of: feat/fix/docs/style/refactor/test/chore
9. Focus on both WHAT changed and WHY it was necessary
10. Include any important technical details or context
11. DO NOT include any Chinese content
12. DO NOT wrap the response in any markdown or code block markers

Example response format:
feat: add user authentication module

1. Implement JWT-based authentication system
2. Add user login and registration endpoints
3. Include password hashing with bcrypt
4. Set up token refresh mechanism

Log: Added user authentication feature with login and registration

Influence:
1. Test user registration with valid and invalid inputs
2. Verify login functionality with correct and incorrect credentials
3. Test JWT token generation and validation
4. Verify password security and hashing
5. Test token refresh mechanism and expiration handling
6. Verify access control for protected endpoints

Please respond with ONLY the commit message following this format,
DO NOT end commit titles with any punctuation."#;

const CHINESE_PROMPT_TEMPLATE: &str = r#"请分析以下 git diff 内容，并按照以下格式生成提交信息：
1. 第一行为标题：type: message（不超过50个字符）

3. 详细的中文说明（解释做了什么改动以及为什么需要这些改动）
4. 说明下方空一行
5. Log 字段（仅当此次变更涉及用户可感知的功能/UI层面变化，产品经理会向用户说明的内容时才添加）
6. Log 字段下方空一行（如果存在 Log 字段）
7. Influence 部分，提供黑盒测试的重点和范围
8. type 必须是以下之一：feat/fix/docs/style/refactor/test/chore
9. 关注点：变更内容（做了什么）和变更原因（为什么）
10. 包含重要的技术细节或上下文
11. 不要使用任何 markdown 或代码块标记
12. 标题结尾不要使用标点符号

示例格式：
feat: 添加用户认证模块

1. 实现基于 JWT 的认证系统
2. 添加用户登录和注册端点
3. 包含使用 bcrypt 的密码哈希处理
4. 设置令牌刷新机制

Log: 新增用户登录注册功能

Influence:
1. 测试用户注册功能，包括有效和无效输入
2. 验证登录功能，测试正确和错误的凭据
3. 测试 JWT 令牌生成和验证流程
4. 验证密码安全性和哈希处理
5. 测试令牌刷新机制和过期处理
6. 验证受保护端点的访问控制"#;

const BILINGUAL_PROMPT_TEMPLATE: &str = r#"Please analyze the git diff content and generate a detailed bilingual commit message with:
1. First line in English: type: message (under 50 characters)
2. Empty line after the title
3. Detailed explanation in English (what was changed and why)
4. Empty line after English explanation
5. Log field in English (ONLY if this change involves user-facing features/UI changes)
6. Empty line after English Log field (if present)
7. Influence section in English with black-box testing recommendations

9. Chinese title and explanation (translate the English content)
10. Empty line after Chinese explanation
11. Chinese Log field (translate the English Log field, only if present)
12. Empty line after Chinese Log field (if present)
13. Chinese Influence section (translate the English testing suggestions)
14. Type must be one of: feat/fix/docs/style/refactor/test/chore
15. Focus on both WHAT changed and WHY it was necessary
16. Include any important technical details or context
17. DO NOT wrap the response in any markdown or code block markers

Example response format:
feat: add user authentication module

1. Implement JWT-based authentication system
2. Add user login and registration endpoints
3. Include password hashing with bcrypt
4. Set up token refresh mechanism

Log: Added user authentication feature with login and registration

Influence:
1. Test user registration with valid and invalid inputs
2. Verify login functionality with correct and incorrect credentials
3. Test JWT token generation and validation
4. Verify password security and hashing
5. Test token refresh mechanism and expiration handling
6. Verify access control for protected endpoints

feat: 添加用户认证模块

1. 实现基于 JWT 的认证系统
2. 添加用户登录和注册端点
3. 包含使用 bcrypt 的密码哈希处理
4. 设置令牌刷新机制

Log: 新增用户登录注册功能

Influence:
1. 测试用户注册功能，包括有效和无效输入
2. 验证登录功能，测试正确和错误的凭据
3. 测试 JWT 令牌生成和验证流程
4. 验证密码安全性和哈希处理
5. 测试令牌刷新机制和过期处理
6. 验证受保护端点的访问控制

Please respond with ONLY the commit message following this format,
DO NOT end commit titles with any punctuation."#;

// 无测试建议版本的提示词模板
const ENGLISH_PROMPT_TEMPLATE_NO_TEST: &str = r#"Please analyze the git diff content and generate a commit message in English only:
1. First line: type: message (under 50 characters)
2. Empty line after the title
3. Detailed explanation in English (what was changed and why)
4. Empty line after explanation
5. Log field (ONLY if this change involves user-facing features/UI changes that product managers would communicate to users)
6. Type must be one of: feat/fix/docs/style/refactor/test/chore
7. Focus on both WHAT changed and WHY it was necessary
8. Include any important technical details or context
9. DO NOT include any Chinese content
10. DO NOT wrap the response in any markdown or code block markers

Example response format:
feat: add user authentication module

1. Implement JWT-based authentication system
2. Add user login and registration endpoints
3. Include password hashing with bcrypt
4. Set up token refresh mechanism

Log: Added user authentication feature with login and registration

Please respond with ONLY the commit message following this format,
DO NOT end commit titles with any punctuation."#;

const CHINESE_PROMPT_TEMPLATE_NO_TEST: &str = r#"请分析以下 git diff 内容，并按照以下格式生成提交信息：
1. 第一行为标题：type: message（不超过50个字符）
2. 标题下方空一行
3. 详细的中文说明（解释做了什么改动以及为什么需要这些改动）
4. 说明下方空一行
5. Log 字段（仅当此次变更涉及用户可感知的功能/UI层面变化时才添加）
6. type 必须是以下之一：feat/fix/docs/style/refactor/test/chore
7. 关注点：变更内容（做了什么）和变更原因（为什么）
8. 包含重要的技术细节或上下文
9. 不要使用任何 markdown 或代码块标记
10. 标题结尾不要使用标点符号

示例格式：
feat: 添加用户认证模块

1. 实现基于 JWT 的认证系统
2. 添加用户登录和注册端点
3. 包含使用 bcrypt 的密码哈希处理
4. 设置令牌刷新机制

Log: 新增用户登录注册功能"#;

const BILINGUAL_PROMPT_TEMPLATE_NO_TEST: &str = r#"Please analyze the git diff content and generate a detailed bilingual commit message with:
1. First line in English: type: message (under 50 characters)
2. Empty line after the title
3. Detailed explanation in English (what was changed and why)
4. Empty line after English explanation
5. Log field in English (ONLY if this change involves user-facing features/UI changes)
6. Empty line after English Log field (if present)
7. Chinese title and explanation (translate the English content)
8. Empty line after Chinese explanation
9. Chinese Log field (translate the English Log field, only if present)
10. Type must be one of: feat/fix/docs/style/refactor/test/chore
11. Focus on both WHAT changed and WHY it was necessary
12. Include any important technical details or context
13. DO NOT wrap the response in any markdown or code block markers

Example response format:
feat: add user authentication module

1. Implement JWT-based authentication system
2. Add user login and registration endpoints
3. Include password hashing with bcrypt
4. Set up token refresh mechanism

Log: Added user authentication feature with login and registration

feat: 添加用户认证模块

1. 实现基于 JWT 的认证系统
2. 添加用户登录和注册端点
3. 包含使用 bcrypt 的密码哈希处理
4. 设置令牌刷新机制

Log: 新增用户登录注册功能

Please respond with ONLY the commit message following this format,
DO NOT end commit titles with any punctuation."#;

impl LanguageMode {
    fn determine(only_chinese: bool, only_english: bool) -> Self {
        if only_english {
            Self::EnglishOnly
        } else if only_chinese {
            Self::ChineseOnly
        } else {
            Self::Bilingual
        }
    }

    fn template(&self, include_test_suggestions: bool) -> &'static str {
        match (self, include_test_suggestions) {
            (Self::EnglishOnly, true) => ENGLISH_PROMPT_TEMPLATE,
            (Self::EnglishOnly, false) => ENGLISH_PROMPT_TEMPLATE_NO_TEST,
            (Self::ChineseOnly, true) => CHINESE_PROMPT_TEMPLATE,
            (Self::ChineseOnly, false) => CHINESE_PROMPT_TEMPLATE_NO_TEST,
            (Self::Bilingual, true) => BILINGUAL_PROMPT_TEMPLATE,
            (Self::Bilingual, false) => BILINGUAL_PROMPT_TEMPLATE_NO_TEST,
        }
    }
}

// 统一的提示词构建函数
fn build_prompt(mode: LanguageMode, user_message: Option<&str>, include_test_suggestions: bool) -> String {
    let mut prompt = String::from(mode.template(include_test_suggestions));

    if let Some(msg) = user_message {
        match mode {
            LanguageMode::ChineseOnly => {
                prompt.push_str(&format!("\n\n用户描述：\n{}\n\n变更内容：\n", msg));
            }
            _ => {
                prompt.push_str(&format!("\n\nUser Description:\n{}\n\nChanges:\n", msg));
            }
        }
    } else {
        match mode {
            LanguageMode::ChineseOnly => {
                prompt.push_str("\n\n变更内容：\n");
            }
            _ => {
                prompt.push_str("\n\nHere are the changes:\n");
            }
        }
    }

    prompt
}

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
    mut only_chinese: bool,
    mut only_english: bool,
    no_test_suggestions: bool,
    issues: Option<String>,
) -> anyhow::Result<()> {
    // 加载配置，如果指定了参数则使用参数值，否则使用配置中的默认值
    if let Ok(config) = config::Config::load() {
        if !only_chinese && !only_english {
            only_chinese = config.only_chinese;
            only_english = config.only_english;
        }
    }

    // 处理语言选项冲突：only_english 优先级最高
    if only_english {
        only_chinese = false;
    } else if only_chinese {
        only_english = false;
    }

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

    // 设置环境变量
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

    // 确定语言模式并构建提示词，考虑是否包含测试建议
    let language_mode = LanguageMode::determine(only_chinese, only_english);
    let include_test_suggestions = !no_test_suggestions;
    let prompt = build_prompt(language_mode, message.as_deref(), include_test_suggestions);

    debug!("生成的提示信息：\n{}", prompt);

    info!("使用 {:?} 服务生成提交信息", config.default_service);
    let service = config.get_default_service()?;
    let translator = ai_service::create_translator_for_service(service).await?;

    println!("\n正在生成提交信息建议...");
    let mut message = translator.chat(&prompt, &diff).await?
        .trim_start_matches("[NO_TRANSLATE]")
        .trim_start_matches("、、、plaintext")
        .trim()
        .to_string();

    // 如果提供了具体的type，确保使用该type
    if let Some(t) = commit_type {
        message = ensure_commit_type(&message, &[t]);
    }

    // 处理换行
    let mut content = message.lines().map(|line| {
        if line.trim().is_empty() {
            line.to_string()
        } else {
            git::wrap_text(line, 72)
        }
    }).collect::<Vec<_>>().join("\n");

    // 如果指定了 issues 参数，添加引用字段
    if let Some(issues_str) = issues {
        match parse_issue_reference(&issues_str) {
            Ok(reference) => {
                // 在提交信息末尾添加空行和引用字段
                if !content.ends_with('\n') {
                    content.push('\n');
                }
                content.push('\n');
                content.push_str(&reference);
            }
            Err(e) => {
                eprintln!("警告: 解析 issues 参数失败: {}", e);
            }
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_issue_url() {
        let url = "https://github.com/zccrs/git-commit-helper/issues/123";
        let result = parse_github_issue(url).unwrap();
        // 由于无法在测试环境中获取git remote，这里只测试格式解析
        assert!(result.contains("#123") || result.contains("zccrs/git-commit-helper#123"));
    }

    #[test]
    fn test_parse_pms_bug_link() {
        let url = "https://pms.uniontech.com/bug-view-320461.html";
        let result = parse_pms_link(url).unwrap();
        assert_eq!(result, "PMS: BUG-320461");
    }

    #[test]
    fn test_parse_pms_task_link() {
        let url = "https://pms.uniontech.com/task-view-374223.html";
        let result = parse_pms_link(url).unwrap();
        assert_eq!(result, "PMS: TASK-374223");
    }

    #[test]
    fn test_parse_pms_story_link() {
        let url = "https://pms.uniontech.com/story-view-38949.html";
        let result = parse_pms_link(url).unwrap();
        assert_eq!(result, "PMS: STORY-38949");
    }

    #[test]
    fn test_parse_issue_number() {
        let issue = "123";
        let result = parse_issue_reference(issue).unwrap();
        assert_eq!(result, "Fixes: #123");
    }

    #[test]
    fn test_parse_invalid_format() {
        let invalid = "invalid-format";
        let result = parse_issue_reference(invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_multiple_issues() {
        let issues = "123 456 https://github.com/owner/repo/issues/789";
        let result = parse_issue_reference(issues).unwrap();
        assert!(result.contains("Fixes: #123 #456"));
        assert!(result.contains("owner/repo#789") || result.contains("#789"));
    }

    #[test]
    fn test_parse_multiple_pms_links() {
        let issues = "https://pms.uniontech.com/bug-view-320461.html https://pms.uniontech.com/task-view-374223.html https://pms.uniontech.com/story-view-38949.html";
        let result = parse_pms_link_multiple(issues).unwrap();
        assert_eq!(result, "PMS: BUG-320461 TASK-374223 STORY-38949");
    }

    #[test]
    fn test_parse_mixed_issues_and_pms() {
        let issues = "123 https://pms.uniontech.com/bug-view-320461.html https://github.com/owner/repo/issues/456";
        let result = parse_issue_reference(issues).unwrap();

        // 结果应该包含两行，分别是 Fixes 和 PMS
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);

        let fixes_line = lines.iter().find(|&&line| line.starts_with("Fixes:")).unwrap();
        let pms_line = lines.iter().find(|&&line| line.starts_with("PMS:")).unwrap();

        assert!(fixes_line.contains("#123"));
        assert!(fixes_line.contains("owner/repo#456") || fixes_line.contains("#456"));
        assert!(pms_line.contains("BUG-320461"));
    }

    #[test]
    fn test_parse_comma_separated_issues() {
        let issues = "123,456,789";
        let result = parse_issue_reference(issues).unwrap();
        assert_eq!(result, "Fixes: #123 #456 #789");
    }

    #[test]
    fn test_parse_mixed_separators() {
        let issues = "123 456,789 https://pms.uniontech.com/task-view-374223.html";
        let result = parse_issue_reference(issues).unwrap();

        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);

        let fixes_line = lines.iter().find(|&&line| line.starts_with("Fixes:")).unwrap();
        let pms_line = lines.iter().find(|&&line| line.starts_with("PMS:")).unwrap();

        assert_eq!(*fixes_line, "Fixes: #123 #456 #789");
        assert_eq!(*pms_line, "PMS: TASK-374223");
    }

    // 辅助测试函数
    fn parse_pms_link_multiple(issues: &str) -> anyhow::Result<String> {
        parse_issue_reference(issues)
    }
}
