use async_trait::async_trait;
use regex::Regex;
use copilot_client::Message;

pub mod ai_service;

pub struct CommitMessage {
    pub title: String,
    pub body: Option<String>,
    pub marks: Vec<String>,
}

#[async_trait]
pub trait Translator {
    async fn translate(&self, text: &str) -> anyhow::Result<String>;
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

        result.join("\n")
    }
}
