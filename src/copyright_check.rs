use anyhow::Result;
use std::process::Command;
use log::{debug, info, warn};
use chrono::Datelike;
use serde_json::Value;

/// 文件信息结构
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub file_type: String,
    pub content: String,
    pub is_new: bool,
}

/// 版权检查结果
#[derive(Debug)]
pub struct CopyrightCheckResult {
    pub has_issues: bool,
    pub issues: Vec<String>,
}

/// 获取暂存区的文件信息
pub fn get_staged_files() -> Result<Vec<FileInfo>> {
    // 获取暂存文件列表
    let files_output = Command::new("git")
        .args(&["diff", "--cached", "--name-status"])
        .output()?;

    if !files_output.status.success() {
        return Err(anyhow::anyhow!("获取暂存文件列表失败"));
    }

    let files_status = String::from_utf8(files_output.stdout)?;
    let mut files_info = Vec::new();

    // 解析文件状态并获取文件内容
    for line in files_status.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let status = parts[0];
            let path = parts[1];
            
            // 判断是否为新文件
            let is_new = status == "A";
            
            // 获取文件类型
            let file_type = get_file_type(path);
            
            // 获取文件内容（只获取文件头部，用于版权检查）
            let content = get_file_content(path, is_new)?;
            
            files_info.push(FileInfo {
                path: path.to_string(),
                file_type,
                content,
                is_new,
            });
        }
    }

    Ok(files_info)
}

/// 获取文件类型
fn get_file_type(path: &str) -> String {
    if let Some(ext) = path.rsplit('.').next() {
        match ext.to_lowercase().as_str() {
            "rs" => "Rust".to_string(),
            "py" => "Python".to_string(),
            "js" | "ts" => "JavaScript/TypeScript".to_string(),
            "java" => "Java".to_string(),
            "c" | "h" => "C".to_string(),
            "cpp" | "hpp" | "cc" | "cxx" => "C++".to_string(),
            "go" => "Go".to_string(),
            "sh" => "Shell".to_string(),
            "md" => "Markdown".to_string(),
            "txt" => "Text".to_string(),
            "json" | "yaml" | "yml" | "toml" => "Config".to_string(),
            _ => format!("Unknown ({})", ext),
        }
    } else {
        "Unknown".to_string()
    }
}

/// 获取文件内容（前50行）
fn get_file_content(path: &str, is_new: bool) -> Result<String> {
    // 对于新文件，获取暂存区的内容
    // 对于修改的文件，获取工作区的最新内容
    if is_new {
        // 新文件：从暂存区获取
        let output = Command::new("git")
            .args(&["show", &format!(":{}", path)])
            .output()?;

        if output.status.success() {
            let content = String::from_utf8(output.stdout)?;
            // 只返回前 50 行，用于版权检查
            let lines: Vec<&str> = content.lines().take(50).collect();
            Ok(lines.join("\n"))
        } else {
            // 如果获取失败，尝试读取工作区文件
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    let lines: Vec<&str> = content.lines().take(50).collect();
                    Ok(lines.join("\n"))
                }
                Err(_) => Ok(String::new()),
            }
        }
    } else {
        // 修改的文件：直接读取工作区的最新内容
        match std::fs::read_to_string(path) {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().take(50).collect();
                Ok(lines.join("\n"))
            }
            Err(_) => {
                // 如果工作区文件不存在，尝试从暂存区获取
                let output = Command::new("git")
                    .args(&["show", &format!(":{}", path)])
                    .output()?;

                if output.status.success() {
                    let content = String::from_utf8(output.stdout)?;
                    let lines: Vec<&str> = content.lines().take(50).collect();
                    Ok(lines.join("\n"))
                } else {
                    Ok(String::new())
                }
            }
        }
    }
}

/// 检查单个文件的版权
fn check_file_copyright(file: &FileInfo, current_year: i32) -> Vec<String> {
    let mut issues = Vec::new();
    let content = &file.content;
    
    // 检查是否包含版权声明
    let has_copyright = content.contains("Copyright") || 
                       content.contains("copyright") ||
                       content.contains("版权");
    
    if !has_copyright {
        issues.push(format!("文件 {} 缺少版权声明", file.path));
    }
    
    // 检查版权年份
    // 支持两种格式：单个年份（2023）和范围格式（2023 - 2026）
    let year_pattern = regex::Regex::new(r"(?:Copyright|copyright|版权).*?(\d{4})(?:\s*-\s*(\d{4}))?").unwrap();
    if let Some(caps) = year_pattern.captures(content) {
        if let Some(year_str) = caps.get(1) {
            if let Ok(start_year) = year_str.as_str().parse::<i32>() {
                // 检查是否有结束年份（范围格式）
                let end_year = if let Some(end_year_str) = caps.get(2) {
                    end_year_str.as_str().parse::<i32>().ok()
                } else {
                    None
                };
                
                // 如果是范围格式，检查结束年份；否则检查单个年份
                let year_to_check = end_year.unwrap_or(start_year);
                
                if year_to_check < current_year {
                    issues.push(format!(
                        "文件 {} 的版权年份 {} 可能需要更新到 {}",
                        file.path, year_to_check, current_year
                    ));
                }
            }
        }
    }
    
    // 检查许可证声明
    let has_license = content.contains("License") || 
                      content.contains("license") ||
                      content.contains("MIT") ||
                      content.contains("GPL") ||
                      content.contains("Apache") ||
                      content.contains("BSD");
    
    if !has_license {
        issues.push(format!("文件 {} 缺少许可证声明", file.path));
    }
    
    issues
}

/// 检查所有暂存文件的版权
pub fn check_copyright() -> Result<CopyrightCheckResult> {
    info!("开始检查暂存文件的版权信息...");
    
    let files = get_staged_files()?;
    let mut all_issues = Vec::new();
    
    // 获取当前年份
    let current_year = chrono::Local::now().year();
    
    for file in &files {
        debug!("检查文件: {}", file.path);
        let issues = check_file_copyright(file, current_year);
        all_issues.extend(issues);
    }
    
    let has_issues = !all_issues.is_empty();
    
    if has_issues {
        warn!("发现 {} 个版权相关问题", all_issues.len());
        for issue in &all_issues {
            warn!("  - {}", issue);
        }
    } else {
        info!("所有文件的版权检查通过");
    }
    
    Ok(CopyrightCheckResult {
        has_issues,
        issues: all_issues,
    })
}

/// 清理 AI 响应，移除 markdown 代码块标记
fn clean_ai_response(response: &str) -> String {
    let response = response.trim();
    
    // 移除 ```json 或 ``` 等代码块标记
    if response.starts_with("```") {
        let lines: Vec<&str> = response.lines().collect();
        if lines.len() > 2 {
            // 移除第一行和最后一行的代码块标记
            return lines[1..lines.len()-1].join("\n");
        }
    }
    
    response.to_string()
}

/// 使用 AI 检查版权信息
pub async fn check_copyright_with_ai(config: &crate::config::Config) -> Result<CopyrightCheckResult> {
    info!("开始使用 AI 检查暂存文件的版权信息...");
    
    let files = get_staged_files()?;
    let mut all_issues = Vec::new();
    
    // 获取当前年份
    let current_year = chrono::Local::now().year();
    
    // 创建 AI 服务
    let translator = crate::ai_service::create_translator(config).await?;
    
    for file in &files {
        debug!("使用 AI 检查文件: {}", file.path);
        
        // 构建 AI prompt
        let prompt = build_copyright_check_prompt(&file.content, current_year);
        
        // 调用 AI
        match translator.chat("你是一个版权声明审核助手。", &prompt).await {
            Ok(ai_response) => {
                debug!("AI 响应: {}", ai_response);
                
                // 清理 AI 响应，移除 markdown 代码块标记
                let cleaned_response = clean_ai_response(&ai_response);
                
                // 解析 AI 返回的 JSON
                if let Ok(json) = serde_json::from_str::<Value>(&cleaned_response) {
                    // 检查是否有版权声明
                    if let Some(has_copyright) = json.get("has_copyright").and_then(|v| v.as_bool()) {
                        if !has_copyright {
                            all_issues.push(format!("文件 {} 缺少版权声明", file.path));
                        }
                    }
                    
                    // 检查是否有许可证声明
                    if let Some(has_license) = json.get("has_license").and_then(|v| v.as_bool()) {
                        if !has_license {
                            all_issues.push(format!("文件 {} 缺少许可证声明", file.path));
                        }
                    }
                    
                    // 检查年份是否需要更新
                    if let Some(year_needs_update) = json.get("year_needs_update").and_then(|v| v.as_bool()) {
                        if year_needs_update {
                            if let Some(current_year_str) = json.get("current_year").and_then(|v| v.as_str()) {
                                all_issues.push(format!(
                                    "文件 {} 的版权年份可能需要更新到 {}",
                                    file.path, current_year_str
                                ));
                            }
                        }
                    }
                    
                    // 添加 AI 识别的其他问题
                    if let Some(issues) = json.get("issues").and_then(|v| v.as_array()) {
                        for issue in issues {
                            if let Some(issue_str) = issue.as_str() {
                                all_issues.push(format!("文件 {} - {}", file.path, issue_str));
                            }
                        }
                    }
                } else {
                    // JSON 解析失败，回退到硬编码检查
                    warn!("AI 返回格式错误，回退到硬编码检查: {}", file.path);
                    let issues = check_file_copyright(file, current_year);
                    all_issues.extend(issues);
                }
            }
            Err(e) => {
                // AI 调用失败，回退到硬编码检查
                warn!("AI 检查失败，回退到硬编码检查: {} - {}", file.path, e);
                let issues = check_file_copyright(file, current_year);
                all_issues.extend(issues);
            }
        }
    }
    
    let has_issues = !all_issues.is_empty();
    
    if has_issues {
        warn!("发现 {} 个版权相关问题", all_issues.len());
        for issue in &all_issues {
            warn!("  - {}", issue);
        }
    } else {
        info!("所有文件的版权检查通过");
    }
    
    Ok(CopyrightCheckResult {
        has_issues,
        issues: all_issues,
    })
}

/// 构建版权检查的 AI prompt
fn build_copyright_check_prompt(file_content: &str, current_year: i32) -> String {
    format!(
        r#"请检查以下文件内容中的版权声明。

当前年份：{}

请检查以下内容：
1. 是否包含版权声明（Copyright、copyright、版权等关键词）
2. 版权年份是否需要更新到当前年份 {}
3. 是否包含许可证声明（License、MIT、GPL、Apache、BSD 等）

文件内容（前50行）：
{}

请以 JSON 格式返回检查结果，格式如下：
{{
  "has_copyright": true/false,
  "has_license": true/false,
  "year_needs_update": true/false,
  "current_year": "当前年份",
  "issues": ["其他问题的描述列表"]
}}

注意：
- 只返回 JSON，不要包含其他文字说明
- 如果版权年份小于当前年份，year_needs_update 应为 true
- issues 数组中可以包含其他需要提醒的问题"#,
        current_year, current_year, file_content
    )
}

/// 格式化版权检查结果为可读的字符串
pub fn format_copyright_result(result: &CopyrightCheckResult) -> String {
    if !result.has_issues {
        return "✓ 版权检查通过".to_string();
    }
    
    let mut output = String::new();
    output.push_str("✗ 版权检查发现问题：\n");
    for issue in &result.issues {
        output.push_str(&format!("  - {}\n", issue));
    }
    output
}
