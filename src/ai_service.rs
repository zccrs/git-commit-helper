use async_trait::async_trait;
use dialoguer::{Confirm, Select};
use log::{debug, info, warn};
use crate::config::{AIService, Config, AIServiceConfig};
use crate::terminal_format::print_progress;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}
use copilot_client::CopilotClient;

#[async_trait]
pub trait AiService: Send + Sync {
    async fn translate(&self, text: &str) -> anyhow::Result<String> {
        // 使用翻译的 prompt
        let system_prompt = get_translation_prompt(text);
        Ok(self.chat(&system_prompt, text).await?)
    }

    async fn chat(&self, system_prompt: &str, user_content: &str) -> anyhow::Result<String>;
}

pub use AiService as Translator; // 为了兼容性，保留原有的 Translator 类型

pub struct DeepSeekTranslator {
    api_key: String,
    endpoint: String,
    model: String,
    timeout_seconds: u64,
    max_tokens: u64,
}

pub struct OpenAITranslator {
    api_key: String,
    endpoint: String,
    model: String,
    timeout_seconds: u64,
    max_tokens: u64,
}

pub struct ClaudeTranslator {
    api_key: String,
    endpoint: String,
    model: String,
    timeout_seconds: u64,
    max_tokens: u64,
}

pub struct CopilotTranslator {
    client: CopilotClient,
    model: String,
}

pub struct GeminiTranslator {
    api_key: String,
    endpoint: String,
    model: String,
    timeout_seconds: u64,
    max_tokens: u64,
}

pub struct GrokTranslator {
    api_key: String,
    endpoint: String,
    model: String,
    timeout_seconds: u64,
    max_tokens: u64,
}

pub struct QwenTranslator {
    api_key: String,
    endpoint: String,
    model: String,
    timeout_seconds: u64,
    max_tokens: u64,
}

impl DeepSeekTranslator {
    pub fn new(config: &AIServiceConfig) -> Self {
        Self {
            api_key: config.api_key.clone(),
            endpoint: config.api_endpoint.clone()
                .unwrap_or_else(|| "https://api.deepseek.com/v1".into()),
            model: config.model.clone()
                .unwrap_or_else(|| "deepseek-chat".into()),
            timeout_seconds: crate::config::Config::load()
                .map(|c| c.timeout_seconds)
                .unwrap_or(20),
            max_tokens: crate::config::Config::load()
                .map(|c| c.max_tokens)
                .unwrap_or(2048),
        }
    }
}

impl OpenAITranslator {
    pub fn new(config: &AIServiceConfig) -> Self {
        Self {
            api_key: config.api_key.clone(),
            endpoint: config.api_endpoint.clone()
                .unwrap_or_else(|| "https://api.openai.com/v1".into()),
            model: config.model.clone()
                .unwrap_or_else(|| "gpt-3.5-turbo".into()),
            timeout_seconds: crate::config::Config::load()
                .map(|c| c.timeout_seconds)
                .unwrap_or(20),
            max_tokens: crate::config::Config::load()
                .map(|c| c.max_tokens)
                .unwrap_or(2048),
        }
    }
}

impl ClaudeTranslator {
    pub fn new(config: &AIServiceConfig) -> Self {
        Self {
            api_key: config.api_key.clone(),
            endpoint: config.api_endpoint.clone()
                .unwrap_or_else(|| "https://api.anthropic.com/v1".into()),
            model: config.model.clone()
                .unwrap_or_else(|| "claude-3-sonnet-20240229".into()),
            timeout_seconds: crate::config::Config::load()
                .map(|c| c.timeout_seconds)
                .unwrap_or(20),
            max_tokens: crate::config::Config::load()
                .map(|c| c.max_tokens)
                .unwrap_or(2048),
        }
    }
}

impl CopilotTranslator {
    pub fn new(client: CopilotClient, model: String) -> Self {
        Self { client, model }
    }
}

impl GeminiTranslator {
    pub fn new(config: &AIServiceConfig) -> Self {
        Self {
            api_key: config.api_key.clone(),
            endpoint: config.api_endpoint.clone()
                .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".into()),
            model: config.model.clone()
                .unwrap_or_else(|| "gemini-2.0-flash".into()),
            timeout_seconds: crate::config::Config::load()
                .map(|c| c.timeout_seconds)
                .unwrap_or(20),
            max_tokens: crate::config::Config::load()
                .map(|c| c.max_tokens)
                .unwrap_or(2048),
        }
    }
}

impl GrokTranslator {
    pub fn new(config: &AIServiceConfig) -> Self {
        Self {
            api_key: config.api_key.clone(),
            endpoint: config.api_endpoint.clone()
                .unwrap_or_else(|| "https://api.x.ai/v1".into()),
            model: config.model.clone()
                .unwrap_or_else(|| "grok-3-latest".into()),
            timeout_seconds: crate::config::Config::load()
                .map(|c| c.timeout_seconds)
                .unwrap_or(20),
            max_tokens: crate::config::Config::load()
                .map(|c| c.max_tokens)
                .unwrap_or(2048),
        }
    }
}

impl QwenTranslator {
    pub fn new(config: &AIServiceConfig) -> Self {
        Self {
            api_key: config.api_key.clone(),
            endpoint: config.api_endpoint.clone()
                .unwrap_or_else(|| "https://dashscope.aliyuncs.com/compatible-mode/v1".into()),
            model: config.model.clone()
                .unwrap_or_else(|| "qwen-plus".into()),
            timeout_seconds: crate::config::Config::load()
                .map(|c| c.timeout_seconds)
                .unwrap_or(20),
            max_tokens: crate::config::Config::load()
                .map(|c| c.max_tokens)
                .unwrap_or(2048),
        }
    }
}

// 添加一个新的工具函数
fn wrap_chinese_text(text: &str, max_width: usize) -> String {
    let mut result = String::new();
    let mut current_line = String::new();
    let mut current_width = 0;

    for c in text.chars() {
        let char_width = if c.is_ascii() { 1 } else { 2 };

        if current_width + char_width > max_width {
            result.push_str(&current_line);
            result.push('\n');
            current_line.clear();
            current_width = 0;
        }

        current_line.push(c);
        current_width += char_width;
    }

    if !current_line.is_empty() {
        result.push_str(&current_line);
    }

    result
}

fn get_translation_prompt(text: &str) -> String {
    let prompt = format!(
    r#"You are a professional translator. Please translate the following Chinese text to English.
    Important rules:
    1. Keep all English content, numbers, and English punctuation unchanged
    2. Do not translate any content inside English double quotes
    3. Preserve the case of all English words
    4. Only return the English translation, DO NOT include the original Chinese text
    5. Keep simple and concise, no need to rewrite or expand the content

    Example response format:
    feat: add support for external plugins

    1. Implement plugin loading mechanism
    2. Add plugin configuration interface
    3. Setup plugin discovery path: "/插件"

    Text to translate:
    {}"#,
    wrap_chinese_text(text, 72));

    debug!("生成的提示词:\n{}", prompt);
    prompt
}

#[async_trait]
impl AiService for DeepSeekTranslator {
    async fn chat(&self, system_prompt: &str, user_content: &str) -> anyhow::Result<String> {
        debug!("使用 DeepSeek，API Endpoint: {}", self.endpoint);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(self.timeout_seconds))
            .build()?;

        let url = format!("{}/chat/completions", self.endpoint);
        let messages = vec![
            serde_json::json!({
                "role": "system",
                "content": system_prompt
            }),
            serde_json::json!({
                "role": "user",
                "content": user_content
            })
        ];
        debug!("发送给 DeepSeek 的消息:\n{}", serde_json::to_string_pretty(&messages)?);
        let body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "max_tokens": self.max_tokens
        });

        let ai_host = match url.split('/').nth(2) {
            Some(host) => host,
            None => "api.deepseek.com",
        };
        print_progress(&format!("正在请求 {} 进行AI对话", ai_host), None);

        loop {
            match client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .json(&body)
                .send()
                .await
            {
                Ok(response) => {
                    print_progress(&format!("正在请求 {} 进行AI对话", ai_host), Some(100));
                    debug!("收到响应: {:#?}", response);

                    if !response.status().is_success() {
                        let error_json = response.json::<serde_json::Value>().await?;
                        debug!("响应内容: {}", serde_json::to_string_pretty(&error_json)?);
                        return Err(anyhow::anyhow!("API 调用失败: {}",
                            error_json["error"]["message"].as_str().unwrap_or("未知错误")));
                    }

                    let result = response.json::<serde_json::Value>().await?;
                    debug!("响应内容: {}", serde_json::to_string_pretty(&result)?);

                    let response = result["choices"][0]["message"]["content"]
                        .as_str()
                        .unwrap_or_default();
                    return Ok(response.to_string());
                }
                Err(e) if e.is_timeout() => {
                    warn!("请求超时: {}", e);
                    if !Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
                        .with_prompt("请求超时，是否重试？")
                        .default(true)
                        .interact()? {
                        return Err(anyhow::anyhow!("请求超时"));
                    }
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
}

#[async_trait]
impl AiService for OpenAITranslator {
    async fn chat(&self, system_prompt: &str, user_content: &str) -> anyhow::Result<String> {
        debug!("使用 OpenAI，API Endpoint: {}", self.endpoint);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(self.timeout_seconds))
            .build()?;

        let url = format!("{}/chat/completions", self.endpoint);
        let messages = vec![
            serde_json::json!({
                "role": "system",
                "content": system_prompt
            }),
            serde_json::json!({
                "role": "user",
                "content": user_content
            })
        ];
        debug!("发送给 OpenAI 的消息:\n{}", serde_json::to_string_pretty(&messages)?);
        let body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "max_tokens": self.max_tokens
        });

        let ai_host = match url.split('/').nth(2) {
            Some(host) => host,
            None => "api.openai.com",
        };
        print_progress(&format!("正在请求 {} 进行AI对话", ai_host), None);

        loop {
            match client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .json(&body)
                .send()
                .await
            {
                Ok(response) => {
                    print_progress(&format!("正在请求 {} 进行AI对话", ai_host), Some(100));
                    debug!("收到响应: {:#?}", response);

                    if !response.status().is_success() {
                        let error_json = response.json::<serde_json::Value>().await?;
                        debug!("响应内容: {}", serde_json::to_string_pretty(&error_json)?);
                        return Err(anyhow::anyhow!("API 调用失败: {}",
                            error_json["error"]["message"].as_str().unwrap_or("未知错误")));
                    }

                    let result = response.json::<serde_json::Value>().await?;
                    debug!("响应内容: {}", serde_json::to_string_pretty(&result)?);

                    let response = result["choices"][0]["message"]["content"]
                        .as_str()
                        .unwrap_or_default();
                    return Ok(response.to_string());
                }
                Err(e) if e.is_timeout() => {
                    warn!("请求超时: {}", e);
                    if !Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
                        .with_prompt("请求超时，是否重试？")
                        .default(true)
                        .interact()? {
                        return Err(anyhow::anyhow!("请求超时"));
                    }
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
}

#[async_trait]
impl AiService for ClaudeTranslator {
    async fn chat(&self, system_prompt: &str, user_content: &str) -> anyhow::Result<String> {
        debug!("使用 Claude，API Endpoint: {}", self.endpoint);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(self.timeout_seconds))
            .build()?;

        let url = format!("{}/messages", self.endpoint);
        let messages = vec![
            serde_json::json!({
                "role": "system",
                "content": system_prompt
            }),
            serde_json::json!({
                "role": "user",
                "content": user_content
            })
        ];
        debug!("发送给 Claude 的消息:\n{}", serde_json::to_string_pretty(&messages)?);
        let body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "max_tokens": self.max_tokens
        });

        let ai_host = match url.split('/').nth(2) {
            Some(host) => host,
            None => "api.anthropic.com",
        };
        print_progress(&format!("正在请求 {} 进行AI对话", ai_host), None);

        loop {
            match client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("anthropic-version", "2023-06-01")
                .json(&body)
                .send()
                .await
            {
                Ok(response) => {
                    print_progress(&format!("正在请求 {} 进行AI对话", ai_host), Some(100));
                    debug!("收到响应: {:#?}", response);

                    if !response.status().is_success() {
                        let error_json = response.json::<serde_json::Value>().await?;
                        debug!("响应内容: {}", serde_json::to_string_pretty(&error_json)?);
                        return Err(anyhow::anyhow!("API 调用失败: {}",
                            error_json["error"]["message"].as_str().unwrap_or("未知错误")));
                    }

                    let result = response.json::<serde_json::Value>().await?;
                    debug!("响应内容: {}", serde_json::to_string_pretty(&result)?);

                    let response = result["content"][0]["text"]
                        .as_str()
                        .unwrap_or_default();
                    return Ok(response.to_string());
                }
                Err(e) if e.is_timeout() => {
                    warn!("请求超时: {}", e);
                    if !Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
                        .with_prompt("请求超时，是否重试？")
                        .default(true)
                        .interact()? {
                        return Err(anyhow::anyhow!("请求超时"));
                    }
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
}

#[async_trait]
impl AiService for CopilotTranslator {
    async fn chat(&self, system_prompt: &str, user_content: &str) -> anyhow::Result<String> {
        debug!("使用 Copilot");
        let ai_host = "copilot.local";
        print_progress(&format!("正在请求 {} 进行AI对话", ai_host), None);

        let messages = vec![
            copilot_client::Message {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            copilot_client::Message {
                role: "user".to_string(),
                content: user_content.to_string(),
            },
        ];
        debug!("发送给 Copilot 的消息:\n{}", serde_json::to_string_pretty(&messages)?);
        let response = self.client.chat_completion(messages, self.model.clone()).await?;
        print_progress(&format!("正在请求 {} 进行AI对话", ai_host), Some(100));
        let result = response.choices.get(0)
            .map(|choice| choice.message.content.clone())
            .unwrap_or_default();
        Ok(result)
    }
}

#[async_trait]
impl AiService for GeminiTranslator {
    async fn chat(&self, system_prompt: &str, user_content: &str) -> anyhow::Result<String> {
        debug!("使用 Gemini，API Endpoint: {}", self.endpoint);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(self.timeout_seconds))
            .build()?;

        let url = format!("{}/models/{}:generateContent?key={}", self.endpoint, self.model, self.api_key);
        let prompt = format!("{}\n\n{}", system_prompt, user_content);
        debug!("发送给 Gemini 的内容:\n{}", prompt);
        let body = serde_json::json!({
            "contents": [{
                "parts": [{
                    "text": prompt
                }]
            }],
            "max_tokens": self.max_tokens
        });

        let ai_host = match url.split('/').nth(2) {
            Some(host) => host,
            None => "generativelanguage.googleapis.com",
        };
        print_progress(&format!("正在请求 {} 进行AI对话", ai_host), None);

        loop {
            match client
                .post(&url)
                .json(&body)
                .send()
                .await
            {
                Ok(response) => {
                    print_progress(&format!("正在请求 {} 进行AI对话", ai_host), Some(100));
                    debug!("收到响应: {:#?}", response);

                    if !response.status().is_success() {
                        let error_json = response.json::<serde_json::Value>().await?;
                        debug!("响应内容: {}", serde_json::to_string_pretty(&error_json)?);
                        return Err(anyhow::anyhow!("API 调用失败: {}",
                            error_json["error"]["message"].as_str().unwrap_or("未知错误")));
                    }

                    let result = response.json::<serde_json::Value>().await?;
                    debug!("响应内容: {}", serde_json::to_string_pretty(&result)?);

                    let response = result["candidates"][0]["content"]["parts"][0]["text"]
                        .as_str()
                        .unwrap_or_default();
                    return Ok(response.to_string());
                }
                Err(e) if e.is_timeout() => {
                    warn!("请求超时: {}", e);
                    if !Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
                        .with_prompt("请求超时，是否重试？")
                        .default(true)
                        .interact()? {
                        return Err(anyhow::anyhow!("请求超时"));
                    }
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
}

#[async_trait]
impl AiService for GrokTranslator {
    async fn chat(&self, system_prompt: &str, user_content: &str) -> anyhow::Result<String> {
        debug!("使用 Grok，API Endpoint: {}", self.endpoint);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(self.timeout_seconds))
            .build()?;

        let url = format!("{}/chat/completions", self.endpoint);
        let messages = vec![
            serde_json::json!({
                "role": "system",
                "content": system_prompt
            }),
            serde_json::json!({
                "role": "user",
                "content": user_content
            })
        ];
        debug!("发送给 Grok 的消息:\n{}", serde_json::to_string_pretty(&messages)?);
        let body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "max_tokens": self.max_tokens
        });

        let ai_host = match url.split('/').nth(2) {
            Some(host) => host,
            None => "api.x.ai",
        };
        print_progress(&format!("正在请求 {} 进行AI对话", ai_host), None);

        loop {
            match client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .json(&body)
                .send()
                .await
            {
                Ok(response) => {
                    print_progress(&format!("正在请求 {} 进行AI对话", ai_host), Some(100));
                    debug!("收到响应: {:#?}", response);

                    if !response.status().is_success() {
                        let error_json = response.json::<serde_json::Value>().await?;
                        debug!("响应内容: {}", serde_json::to_string_pretty(&error_json)?);
                        return Err(anyhow::anyhow!("API 调用失败: {}",
                            error_json["error"]["message"].as_str().unwrap_or("未知错误")));
                    }

                    let result = response.json::<serde_json::Value>().await?;
                    debug!("响应内容: {}", serde_json::to_string_pretty(&result)?);

                    let response = result["choices"][0]["message"]["content"]
                        .as_str()
                        .unwrap_or_default();
                    return Ok(response.to_string());
                }
                Err(e) if e.is_timeout() => {
                    warn!("请求超时: {}", e);
                    if !Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
                        .with_prompt("请求超时，是否重试？")
                        .default(true)
                        .interact()? {
                        return Err(anyhow::anyhow!("请求超时"));
                    }
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
}

#[async_trait]
impl AiService for QwenTranslator {
    async fn chat(&self, system_prompt: &str, user_content: &str) -> anyhow::Result<String> {
        debug!("使用 Qwen，API Endpoint: {}", self.endpoint);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(self.timeout_seconds))
            .build()?;

        let url = format!("{}/chat/completions", self.endpoint);
        let messages = vec![
            serde_json::json!({
                "role": "system",
                "content": system_prompt
            }),
            serde_json::json!({
                "role": "user",
                "content": user_content
            })
        ];
        debug!("发送给 Qwen 的消息:\n{}", serde_json::to_string_pretty(&messages)?);
        let body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "temperature": 0.1,
            "max_tokens": self.max_tokens
        });

        let ai_host = match url.split('/').nth(2) {
            Some(host) => host,
            None => "dashscope.aliyuncs.com",
        };
        print_progress(&format!("正在请求 {} 进行AI对话", ai_host), None);

        loop {
            match client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .json(&body)
                .send()
                .await
            {
                Ok(response) => {
                    print_progress(&format!("正在请求 {} 进行AI对话", ai_host), Some(100));
                    debug!("收到响应: {:#?}", response);

                    if !response.status().is_success() {
                        let error_json = response.json::<serde_json::Value>().await?;
                        debug!("响应内容: {}", serde_json::to_string_pretty(&error_json)?);
                        return Err(anyhow::anyhow!("API 调用失败: {}",
                            error_json["error"]["message"].as_str().unwrap_or("未知错误")));
                    }

                    let result = response.json::<serde_json::Value>().await?;
                    debug!("响应内容: {}", serde_json::to_string_pretty(&result)?);

                    let response = result["choices"][0]["message"]["content"]
                        .as_str()
                        .unwrap_or_default();
                    return Ok(response.to_string());
                }
                Err(e) if e.is_timeout() => {
                    warn!("请求超时: {}", e);
                    if !Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
                        .with_prompt("请求超时，是否重试？")
                        .default(true)
                        .interact()? {
                        return Err(anyhow::anyhow!("请求超时"));
                    }
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
}

pub async fn create_translator(config: &Config) -> anyhow::Result<Box<dyn Translator>> {
    info!("创建 {:?} AI服务", config.default_service);
    let service_config = config.services.iter()
        .find(|s| s.service == config.default_service)
        .ok_or_else(|| anyhow::anyhow!("找不到默认服务的配置"))?;
    create_translator_for_service(service_config).await
}

pub async fn translate_with_fallback(config: &Config, text: &str) -> anyhow::Result<String> {
    let mut tried_services = Vec::new();

    // 如果已设置环境变量，直接返回原文
    if std::env::var("GIT_COMMIT_HELPER_NO_TRANSLATE").is_ok() {
        return Ok(text.trim().to_string());
    }

    debug!("尝试使用默认服务 {:?}", config.default_service);
    if let Some(result) = try_translate(&config.default_service, config, text).await {
        return result;
    }
    tried_services.push(config.default_service.clone());

    for service_config in &config.services {
        if tried_services.contains(&service_config.service) {
            continue;
        }

        debug!("尝试使用备选服务 {:?}", service_config.service);
        if let Some(result) = try_translate(&service_config.service, config, text).await {
            return result;
        }
        tried_services.push(service_config.service.clone());
    }

    while let Some(service) = select_retry_service(config, &tried_services)? {
        debug!("用户选择使用 {:?} 重试", service);
        if let Some(result) = try_translate(&service, config, text).await {
            return result;
        }
        tried_services.push(service);
    }

    Err(anyhow::anyhow!("所有AI服务均失败"))
}

async fn try_translate(service: &AIService, config: &Config, text: &str) -> Option<anyhow::Result<String>> {
    let service_config = config.services.iter()
        .find(|s| s.service == *service)?;

    let translator = create_translator_for_service(service_config).await.ok()?;
    match translator.translate(text).await {
        Ok(result) => Some(Ok(result)),
        Err(e) => {
            warn!("{:?} 服务翻译失败: {}", service, e);
            None
        }
    }
}

fn select_retry_service(config: &Config, tried_services: &[AIService]) -> anyhow::Result<Option<AIService>> {
    let available_services: Vec<_> = config.services.iter()
        .filter(|s| !tried_services.contains(&s.service))
        .collect();

    if available_services.is_empty() {
        return Ok(None);
    }

    let options: Vec<String> = available_services.iter()
        .map(|s| format!("{:?}", s.service))
        .collect();

    println!("\n之前的翻译尝试都失败了，是否要使用其他服务重试？");
    if !Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .default(true)
        .interact()? {
        return Ok(None);
    }

    let selection = Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("请选择要使用的服务")
        .items(&options)
        .default(0)
        .interact()?;

    Ok(Some(available_services[selection].service.clone()))
}

pub async fn create_translator_for_service(service_config: &AIServiceConfig) -> anyhow::Result<Box<dyn Translator>> {
    // 获取全局配置中的超时时间
    let config = crate::config::Config::load().ok();
    let timeout = config.as_ref().map(|c| c.timeout_seconds).unwrap_or(20);

    Ok(match service_config.service {
        AIService::DeepSeek => {
            let mut translator = DeepSeekTranslator::new(service_config);
            translator.timeout_seconds = timeout;
            Box::new(translator)
        },
        AIService::OpenAI => {
            let mut translator = OpenAITranslator::new(service_config);
            translator.timeout_seconds = timeout;
            Box::new(translator)
        },
        AIService::Claude => {
            let mut translator = ClaudeTranslator::new(service_config);
            translator.timeout_seconds = timeout;
            Box::new(translator)
        },
        AIService::Copilot => {
            let editor_version = "1.0.0".to_string();
            let client = CopilotClient::new_with_models(service_config.api_key.clone(), editor_version).await?;
            let model_id = service_config.model.clone().unwrap_or_else(|| "copilot-chat".to_string());
            Box::new(CopilotTranslator::new(client, model_id))
        },
        AIService::Gemini => {
            let mut translator = GeminiTranslator::new(service_config);
            translator.timeout_seconds = timeout;
            Box::new(translator)
        },
        AIService::Grok => {
            let mut translator = GrokTranslator::new(service_config);
            translator.timeout_seconds = timeout;
            Box::new(translator)
        },
        AIService::Qwen => {
            let mut translator = QwenTranslator::new(service_config);
            translator.timeout_seconds = timeout;
            Box::new(translator)
        },
    })
}
