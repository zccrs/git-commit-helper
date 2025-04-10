use async_trait::async_trait;
use regex::Regex;

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
        let mut lines = content.lines().peekable();
        
        let title = lines.next().unwrap_or("").to_string();
        let mut body = Vec::new();
        let mut marks = Vec::new();
        let mut is_body = false;

        while let Some(line) = lines.next() {
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

        CommitMessage {
            title,
            body: if body.is_empty() { None } else { Some(body.join("\n")) },
            marks,
        }
    }

    pub fn format(&self) -> String {
        let mut result = Vec::new();
        result.push(self.title.clone());
        
        if let Some(body) = &self.body {
            result.push(String::new());
            result.push(body.clone());
        }
        
        if !self.marks.is_empty() {
            result.push(String::new());
            result.extend(self.marks.clone());
        }
        
        result.join("\n")
    }
}
