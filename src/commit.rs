use regex::Regex;
use crate::ai_service;
use crate::config;
use crate::git;

/// 从提交消息中提取 Change-Id
fn extract_change_id(message: &str) -> Option<String> {
    let change_id_regex = Regex::new(r"(?m)^Change-Id:\s*(.+)$").ok()?;
    change_id_regex.captures(message)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().trim().to_string())
}

/// 将 Change-Id 添加到提交消息中（如果还没有的话）
fn append_change_id(message: &str, change_id: &str) -> String {
    // 检查消息中是否已经有 Change-Id
    if message.contains("Change-Id:") {
        return message.to_string();
    }
    
    let mut result = message.to_string();
    
    // 确保消息末尾有换行
    if !result.ends_with('\n') {
        result.push('\n');
    }
    
    // 添加 Change-Id（如果有其他标记，Change-Id 应该在最后）
    result.push('\n');
    result.push_str(&format!("Change-Id: {}", change_id));
    
    result
}

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

// 无Log字段版本的提示词模板
const ENGLISH_PROMPT_TEMPLATE_NO_LOG: &str = r#"Please analyze the git diff content and generate a commit message in English only:
1. First line: type: message (under 50 characters)
2. Empty line after the title
3. Detailed explanation in English (what was changed and why)
4. Empty line after explanation
5. Influence section with black-box testing recommendations
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

Influence:
1. Test user registration with valid and invalid inputs
2. Verify login functionality with correct and incorrect credentials
3. Test JWT token generation and validation
4. Verify password security and hashing
5. Test token refresh mechanism and expiration handling
6. Verify access control for protected endpoints

Please respond with ONLY the commit message following this format,
DO NOT end commit titles with any punctuation."#;

const CHINESE_PROMPT_TEMPLATE_NO_LOG: &str = r#"请分析以下 git diff 内容，并按照以下格式生成提交信息：
1. 第一行为标题：type: message（不超过50个字符）

3. 详细的中文说明（解释做了什么改动以及为什么需要这些改动）
4. 说明下方空一行
5. Influence 部分，提供黑盒测试的重点和范围
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

Influence:
1. 测试用户注册功能，包括有效和无效输入
2. 验证登录功能，测试正确和错误的凭据
3. 测试 JWT 令牌生成和验证流程
4. 验证密码安全性和哈希处理
5. 测试令牌刷新机制和过期处理
6. 验证受保护端点的访问控制"#;

const BILINGUAL_PROMPT_TEMPLATE_NO_LOG: &str = r#"Please analyze the git diff content and generate a detailed bilingual commit message with:
1. First line in English: type: message (under 50 characters)
2. Empty line after the title
3. Detailed explanation in English (what was changed and why)
4. Empty line after English explanation
5. Influence section in English with black-box testing recommendations

7. Chinese title and explanation (translate the English content)
8. Empty line after Chinese explanation
9. Chinese Influence section (translate the English testing suggestions)
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

Influence:
1. 测试用户注册功能，包括有效和无效输入
2. 验证登录功能，测试正确和错误的凭据
3. 测试 JWT 令牌生成和验证流程
4. 验证密码安全性和哈希处理
5. 测试令牌刷新机制和过期处理
6. 验证受保护端点的访问控制

Please respond with ONLY the commit message following this format,
DO NOT end commit titles with any punctuation."#;

// 无测试建议无Log字段版本的提示词模板
const ENGLISH_PROMPT_TEMPLATE_NO_TEST_NO_LOG: &str = r#"Please analyze the git diff content and generate a commit message in English only:
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

const CHINESE_PROMPT_TEMPLATE_NO_TEST_NO_LOG: &str = r#"请分析以下 git diff 内容，并按照以下格式生成提交信息：
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

const BILINGUAL_PROMPT_TEMPLATE_NO_TEST_NO_LOG: &str = r#"Please analyze the git diff content and generate a detailed bilingual commit message with:
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

    fn template(&self, include_test_suggestions: bool, include_log: bool) -> &'static str {
        match (self, include_test_suggestions, include_log) {
            (Self::EnglishOnly, true, true) => ENGLISH_PROMPT_TEMPLATE,
            (Self::EnglishOnly, false, true) => ENGLISH_PROMPT_TEMPLATE_NO_TEST,
            (Self::EnglishOnly, true, false) => ENGLISH_PROMPT_TEMPLATE_NO_LOG,
            (Self::EnglishOnly, false, false) => ENGLISH_PROMPT_TEMPLATE_NO_TEST_NO_LOG,
            (Self::ChineseOnly, true, true) => CHINESE_PROMPT_TEMPLATE,
            (Self::ChineseOnly, false, true) => CHINESE_PROMPT_TEMPLATE_NO_TEST,
            (Self::ChineseOnly, true, false) => CHINESE_PROMPT_TEMPLATE_NO_LOG,
            (Self::ChineseOnly, false, false) => CHINESE_PROMPT_TEMPLATE_NO_TEST_NO_LOG,
            (Self::Bilingual, true, true) => BILINGUAL_PROMPT_TEMPLATE,
            (Self::Bilingual, false, true) => BILINGUAL_PROMPT_TEMPLATE_NO_TEST,
            (Self::Bilingual, true, false) => BILINGUAL_PROMPT_TEMPLATE_NO_LOG,
            (Self::Bilingual, false, false) => BILINGUAL_PROMPT_TEMPLATE_NO_TEST_NO_LOG,
        }
    }
}

// 统一的提示词构建函数
fn build_prompt(mode: LanguageMode, user_message: Option<&str>, include_test_suggestions: bool, include_log: bool, original_message: Option<&str>) -> String {
    let mut prompt = String::from(mode.template(include_test_suggestions, include_log));

    // 如果有原始提交信息（amend 模式），先添加它作为参考
    if let Some(orig_msg) = original_message {
        match mode {
            LanguageMode::ChineseOnly => {
                prompt.push_str(&format!("\n\n原始提交信息（请参考但不要完全照搬）：\n{}\n", orig_msg));
            }
            _ => {
                prompt.push_str(&format!("\n\nOriginal commit message (for reference, but create improved version):\n{}\n", orig_msg));
            }
        }
    }

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
    amend: bool,
    no_review: bool,
    no_translate: bool,
    mut only_chinese: bool,
    mut only_english: bool,
    no_influence: bool,
    no_log: bool,
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

    // 根据是否是 amend 模式选择不同的 diff
    let diff = if amend {
        println!("正在分析上一次提交的更改内容...");
        git::get_last_commit_diff()?
    } else {
        get_staged_diff()?
    };

    if diff.is_empty() {
        if amend {
            return Err(anyhow::anyhow!("无法获取上一次提交的差异内容，可能没有足够的提交历史"));
        } else {
            return Err(anyhow::anyhow!("没有已暂存的改动，请先使用 git add 添加改动"));
        }
    }

    let config = config::Config::load()?;

    // 在确认有差异内容后执行代码审查（对于 amend 模式，我们跳过审查，因为是对已有提交的修改）
    if !amend && !no_review && config.ai_review {
        info!("正在进行代码审查...");
        if let Some(review) = review::review_changes(&config, no_review).await? {
            println!("\n{}\n", review);
        }
    }

    // 设置环境变量标记跳过后续的代码审查
    std::env::set_var("GIT_COMMIT_HELPER_SKIP_REVIEW", "1");

    // 确定语言模式并构建提示词，考虑是否包含测试建议
    let language_mode = LanguageMode::determine(only_chinese, only_english);
    let include_test_suggestions = !no_influence;
    let include_log = !no_log;
    
    // 在 amend 模式下获取原始提交消息作为参考
    let (original_message, original_change_id) = if amend {
        match git::get_last_commit_message() {
            Ok(msg) => {
                let change_id = extract_change_id(&msg);
                (Some(msg), change_id)
            }
            Err(_) => (None, None)
        }
    } else {
        (None, None)
    };
    
    let prompt = build_prompt(
        language_mode, 
        message.as_deref(), 
        include_test_suggestions, 
        include_log,
        original_message.as_deref()
    );

    debug!("生成的提示信息：\n{}", prompt);

    info!("使用 {:?} 服务生成提交信息", config.default_service);
    let service = config.get_default_service()?;
    let translator = ai_service::create_translator_for_service(service).await?;

    if amend {
        println!("\n正在基于上一次提交的更改生成新的提交信息...");
        // 显示原提交信息供参考
        if let Some(ref original_msg) = original_message {
            println!("原提交信息:");
            println!("----------------------------------------");
            println!("{}", original_msg.trim());
            println!("----------------------------------------\n");
        }
    } else {
        println!("\n正在生成提交信息建议...");
    }

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
    } else if amend {
        // 在 amend 模式下且未提供新的 issues 参数时，从原提交中保留 issue 引用（Fixes:/PMS:）
        if let Some(ref orig_msg) = original_message {
            let orig_commit = CommitMessage::parse(orig_msg);
            let issue_marks: Vec<String> = orig_commit.marks.iter()
                .filter(|mark| {
                    let mark_lower = mark.to_lowercase();
                    mark_lower.starts_with("fixes:") || mark_lower.starts_with("pms:")
                })
                .cloned()
                .collect();
            // 只添加新内容中尚未包含的标记
            let marks_to_add: Vec<String> = issue_marks.into_iter()
                .filter(|mark| {
                    let mark_key = mark.split(':').next().unwrap_or("").trim().to_lowercase();
                    !content.lines().any(|line| {
                        line.trim().split(':').next()
                            .map_or(false, |k| k.trim().to_lowercase() == mark_key)
                    })
                })
                .collect();
            if !marks_to_add.is_empty() {
                if !content.ends_with('\n') {
                    content.push('\n');
                }
                content.push('\n');
                content.push_str(&marks_to_add.join("\n"));
            }
        }
    }
    
    // 在 amend 模式下，如果原提交有 Change-Id，保留它
    if amend {
        if let Some(change_id) = original_change_id {
            content = append_change_id(&content, &change_id);
        }
    }

    // 预览生成的提交信息
    if amend {
        println!("\n生成的修改后提交信息预览:");
    } else {
        println!("\n生成的提交信息预览:");
    }
    println!("----------------------------------------");
    println!("{}", content);
    println!("----------------------------------------");

    // 询问用户是否确认提交
    let prompt_text = if amend {
        "是否使用此提交信息修改上一次提交？"
    } else {
        "是否使用此提交信息？"
    };

    if !Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt(prompt_text)
        .default(true)
        .interact()?
    {
        if amend {
            println!("已取消修改上一次提交");
        } else {
            println!("已取消提交");
        }
        return Ok(());
    }

    // 执行git commit
    let mut cmd = Command::new("git");
    cmd.current_dir(std::env::current_dir()?);
    cmd.arg("commit");
    
    if amend {
        cmd.arg("--amend");
    }
    
    cmd.arg("-m").arg(content);
    
    let status = cmd.status()?;

    // 清理环境变量（无论命令是否执行成功）
    std::env::remove_var("GIT_COMMIT_HELPER_SKIP_REVIEW");
    std::env::remove_var("GIT_COMMIT_HELPER_NO_TRANSLATE");

    if !status.success() {
        let action = if amend { "修改提交" } else { "提交" };
        return Err(anyhow::anyhow!("git commit 命令执行失败，{} 失败", action));
    }

    if amend {
        println!("修改提交成功！");
    } else {
        println!("提交成功！");
    }
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

    let message = translator.translate(&prompt, &config::TranslateDirection::ChineseToEnglish).await?.to_string();

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

    #[test]
    fn test_extract_change_id() {
        let message = "feat: add new feature\n\nThis is a commit message\n\nChange-Id: I1234567890abcdef1234567890abcdef12345678\n";
        let change_id = extract_change_id(message);
        assert_eq!(change_id, Some("I1234567890abcdef1234567890abcdef12345678".to_string()));
    }

    #[test]
    fn test_extract_change_id_not_found() {
        let message = "feat: add new feature\n\nThis is a commit message without change id\n";
        let change_id = extract_change_id(message);
        assert_eq!(change_id, None);
    }

    #[test]
    fn test_append_change_id() {
        let message = "feat: add new feature\n\nThis is a commit message\n";
        let change_id = "I1234567890abcdef1234567890abcdef12345678";
        let result = append_change_id(message, change_id);
        assert!(result.contains("Change-Id: I1234567890abcdef1234567890abcdef12345678"));
    }

    #[test]
    fn test_append_change_id_already_exists() {
        let message = "feat: add new feature\n\nThis is a commit message\n\nChange-Id: I1234567890abcdef1234567890abcdef12345678\n";
        let change_id = "I1234567890abcdef1234567890abcdef12345678";
        let result = append_change_id(message, change_id);
        // 应该返回原消息，不重复添加
        assert_eq!(result, message);
    }

    #[test]
    fn test_append_change_id_with_other_marks() {
        let message = "feat: add new feature\n\nThis is a commit message\n\nFixes: #123\nPMS: BUG-456\n";
        let change_id = "I1234567890abcdef1234567890abcdef12345678";
        let result = append_change_id(message, change_id);
        assert!(result.contains("Fixes: #123"));
        assert!(result.contains("PMS: BUG-456"));
        assert!(result.contains("Change-Id: I1234567890abcdef1234567890abcdef12345678"));
        // Change-Id 应该在最后
        assert!(result.ends_with("Change-Id: I1234567890abcdef1234567890abcdef12345678"));
    }

    // 测试 amend 模式下从原提交消息中提取 issue 引用标记
    #[test]
    fn test_extract_issue_marks_from_original_commit() {
        let original = "fix: some bug\n\nDescription here\n\nFixes: #123\nPMS: BUG-456\nChange-Id: Iabc123\n";
        let orig_commit = CommitMessage::parse(original);
        let issue_marks: Vec<String> = orig_commit.marks.iter()
            .filter(|mark| {
                let mark_lower = mark.to_lowercase();
                mark_lower.starts_with("fixes:") || mark_lower.starts_with("pms:")
            })
            .cloned()
            .collect();
        assert_eq!(issue_marks.len(), 2);
        assert!(issue_marks.contains(&"Fixes: #123".to_string()));
        assert!(issue_marks.contains(&"PMS: BUG-456".to_string()));
        // Change-Id 不应被提取为 issue 标记
        assert!(!issue_marks.iter().any(|m| m.starts_with("Change-Id:")));
    }

    // 测试 amend 时新内容已包含 issue 引用时不重复添加
    #[test]
    fn test_amend_no_duplicate_issue_marks() {
        let original = "fix: some bug\n\nFixes: #123\n";
        let orig_commit = CommitMessage::parse(original);
        let issue_marks: Vec<String> = orig_commit.marks.iter()
            .filter(|mark| {
                let mark_lower = mark.to_lowercase();
                mark_lower.starts_with("fixes:") || mark_lower.starts_with("pms:")
            })
            .cloned()
            .collect();

        // 新内容已包含 Fixes:
        let new_content = "fix: improved bug fix\n\nBetter description\n\nFixes: #123";
        let marks_to_add: Vec<String> = issue_marks.into_iter()
            .filter(|mark| {
                let mark_key = mark.split(':').next().unwrap_or("").trim().to_lowercase();
                !new_content.lines().any(|line| {
                    line.trim().split(':').next()
                        .map_or(false, |k| k.trim().to_lowercase() == mark_key)
                })
            })
            .collect();
        // 不应重复添加
        assert!(marks_to_add.is_empty());
    }

    // 测试 amend 时新内容不含 issue 引用时正确添加
    #[test]
    fn test_amend_preserves_issue_marks_when_missing() {
        let original = "fix: some bug\n\nFixes: #123\nPMS: BUG-456\n";
        let orig_commit = CommitMessage::parse(original);
        let issue_marks: Vec<String> = orig_commit.marks.iter()
            .filter(|mark| {
                let mark_lower = mark.to_lowercase();
                mark_lower.starts_with("fixes:") || mark_lower.starts_with("pms:")
            })
            .cloned()
            .collect();

        // 新内容不含 issue 引用
        let new_content = "fix: improved bug fix\n\nBetter description";
        let marks_to_add: Vec<String> = issue_marks.into_iter()
            .filter(|mark| {
                let mark_key = mark.split(':').next().unwrap_or("").trim().to_lowercase();
                !new_content.lines().any(|line| {
                    line.trim().split(':').next()
                        .map_or(false, |k| k.trim().to_lowercase() == mark_key)
                })
            })
            .collect();
        assert_eq!(marks_to_add.len(), 2);
        assert!(marks_to_add.contains(&"Fixes: #123".to_string()));
        assert!(marks_to_add.contains(&"PMS: BUG-456".to_string()));
    }
}
