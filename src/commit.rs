use regex::Regex;
use crate::ai_service;
use crate::config;

// 语言模式枚举
#[derive(Debug, Clone, Copy)]
enum LanguageMode {
    ChineseOnly,
    EnglishOnly,
    Bilingual,
}

// 提示词模板常量
const ENGLISH_PROMPT_TEMPLATE: &str = r#"Please analyze the git diff content and generate a commit message in English only:
1. First line: type: message (under 50 characters)
2. Empty line after the title
3. Detailed explanation in English (what was changed and why)
4. Empty line after explanation
5. Testing Suggestions section with black-box testing recommendations
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

Testing Suggestions:
- Test user registration with valid and invalid inputs
- Verify login functionality with correct and incorrect credentials
- Test JWT token generation and validation
- Verify password security and hashing
- Test token refresh mechanism and expiration handling
- Verify access control for protected endpoints

Please respond with ONLY the commit message following this format,
DO NOT end commit titles with any punctuation."#;

const CHINESE_PROMPT_TEMPLATE: &str = r#"请分析以下 git diff 内容，并按照以下格式生成提交信息：
1. 第一行为标题：type: message（不超过50个字符）
2. 标题下方空一行
3. 详细的中文说明（解释做了什么改动以及为什么需要这些改动）
4. 说明下方空一行
5. 测试建议部分，提供黑盒测试的重点和范围
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

测试建议：
- 测试用户注册功能，包括有效和无效输入
- 验证登录功能，测试正确和错误的凭据
- 测试 JWT 令牌生成和验证流程
- 验证密码安全性和哈希处理
- 测试令牌刷新机制和过期处理
- 验证受保护端点的访问控制"#;

const BILINGUAL_PROMPT_TEMPLATE: &str = r#"Please analyze the git diff content and generate a detailed bilingual commit message with:
1. First line in English: type: message (under 50 characters)
2. Empty line after the title
3. Detailed explanation in English (what was changed and why)
4. Empty line after English explanation
5. Testing Suggestions section in English with black-box testing recommendations
6. Empty line after English testing suggestions
7. Chinese title and explanation (translate the English content)
8. Empty line after Chinese explanation
9. Chinese Testing Suggestions section (translate the English testing suggestions)
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

Testing Suggestions:
- Test user registration with valid and invalid inputs
- Verify login functionality with correct and incorrect credentials
- Test JWT token generation and validation
- Verify password security and hashing
- Test token refresh mechanism and expiration handling
- Verify access control for protected endpoints

feat: 添加用户认证模块

1. 实现基于 JWT 的认证系统
2. 添加用户登录和注册端点
3. 包含使用 bcrypt 的密码哈希处理
4. 设置令牌刷新机制

测试建议：
- 测试用户注册功能，包括有效和无效输入
- 验证登录功能，测试正确和错误的凭据
- 测试 JWT 令牌生成和验证流程
- 验证密码安全性和哈希处理
- 测试令牌刷新机制和过期处理
- 验证受保护端点的访问控制

Please respond with ONLY the commit message following this format,
DO NOT end commit titles with any punctuation."#;

// 无测试建议版本的提示词模板
const ENGLISH_PROMPT_TEMPLATE_NO_TEST: &str = r#"Please analyze the git diff content and generate a commit message in English only:
1. First line: type: message (under 50 characters)
2. Empty line after the title
3. Detailed explanation in English (what was changed and why)
4. Type must be one of: feat/fix/docs/style/refactor/test/chore
5. Focus on both WHAT changed and WHY it was necessary
6. Include any important technical details or context
7. DO NOT include any Chinese content
8. DO NOT wrap the response in any markdown or code block markers

Example response format:
feat: add user authentication module

1. Implement JWT-based authentication system
2. Add user login and registration endpoints
3. Include password hashing with bcrypt
4. Set up token refresh mechanism

Please respond with ONLY the commit message following this format,
DO NOT end commit titles with any punctuation."#;

const CHINESE_PROMPT_TEMPLATE_NO_TEST: &str = r#"请分析以下 git diff 内容，并按照以下格式生成提交信息：
1. 第一行为标题：type: message（不超过50个字符）
2. 标题下方空一行
3. 详细的中文说明（解释做了什么改动以及为什么需要这些改动）
4. type 必须是以下之一：feat/fix/docs/style/refactor/test/chore
5. 关注点：变更内容（做了什么）和变更原因（为什么）
6. 包含重要的技术细节或上下文
7. 不要使用任何 markdown 或代码块标记
8. 标题结尾不要使用标点符号

示例格式：
feat: 添加用户认证模块

1. 实现基于 JWT 的认证系统
2. 添加用户登录和注册端点
3. 包含使用 bcrypt 的密码哈希处理
4. 设置令牌刷新机制"#;

const BILINGUAL_PROMPT_TEMPLATE_NO_TEST: &str = r#"Please analyze the git diff content and generate a detailed bilingual commit message with:
1. First line in English: type: message (under 50 characters)
2. Empty line after the title
3. Detailed explanation in English (what was changed and why)
4. Empty line after English explanation
5. Chinese title and explanation (translate the English content)
6. Type must be one of: feat/fix/docs/style/refactor/test/chore
7. Focus on both WHAT changed and WHY it was necessary
8. Include any important technical details or context
9. DO NOT wrap the response in any markdown or code block markers

Example response format:
feat: add user authentication module

1. Implement JWT-based authentication system
2. Add user login and registration endpoints
3. Include password hashing with bcrypt
4. Set up token refresh mechanism

feat: 添加用户认证模块

1. 实现基于 JWT 的认证系统
2. 添加用户登录和注册端点
3. 包含使用 bcrypt 的密码哈希处理
4. 设置令牌刷新机制

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
    mut only_chinese: bool,
    mut only_english: bool,
    no_test_suggestions: bool,
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
