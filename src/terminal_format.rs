// 终端彩色输出工具模块
// 用于统一管理ANSI颜色和结构化输出

use std::io::{self, Write};

pub struct Style;

impl Style {
    pub const RESET: &'static str = "\x1b[0m";
    pub const BOLD: &'static str = "\x1b[1m";
    pub const BLUE: &'static str = "\x1b[34m";
    pub const BRIGHT_BLUE: &'static str = "\x1b[94m";
    pub const BRIGHT_GREEN: &'static str = "\x1b[92m";
    pub const BRIGHT_YELLOW: &'static str = "\x1b[93m";
    pub const BRIGHT_RED: &'static str = "\x1b[91m";
    pub const GRAY: &'static str = "\x1b[90m";

    // 分隔线
    pub fn separator() -> String {
        format!("{}{}───────────────────────────────────────────────{}\n", Self::GRAY, Self::BOLD, Self::RESET)
    }

    // 标题
    pub fn title(text: &str) -> String {
        format!("{}{}{}{}{}", Self::BRIGHT_BLUE, Self::BOLD, text, Self::RESET, "\n")
    }

    // 绿色内容（如翻译/成功）
    pub fn green(text: &str) -> String {
        format!("{}{}{}{}", Self::BRIGHT_GREEN, text, Self::RESET, "\n")
    }

    // 蓝色内容（如描述）
    pub fn blue(text: &str) -> String {
        format!("{}{}{}{}", Self::BLUE, text, Self::RESET, "\n")
    }

    // 黄色内容（如警告/代码审查报告标题）
    pub fn yellow(text: &str) -> String {
        format!("{}{}{}{}", Self::BRIGHT_YELLOW, text, Self::RESET, "\n")
    }

    // 红色内容（如错误）
    pub fn red(text: &str) -> String {
        format!("{}{}{}{}", Self::BRIGHT_RED, text, Self::RESET, "\n")
    }

    // 普通正文
    pub fn plain(text: &str) -> String {
        format!("{}{}", text, "\n")
    }
}

/// 进度提示工具函数
/// 示例：print_progress("正在请求 github.com 获取PR内容", Some(30));
///      print_progress("正在请求 github.com 获取PR内容", Some(100));
///      print_progress("正在请求 api.openai.com 进行代码审查", None);
pub fn print_progress(msg: &str, percent: Option<u8>) {
    // 构造进度文本
    let progress = match percent {
        Some(p) => format!(" {}%...", p),
        None => " ...".to_string(),
    };
    // \r回到行首，补足空格清除残留
    let text = format!("\r{}{}{}", msg, progress, "      ");
    print!("{}", text);
    io::stdout().flush().ok();
}
