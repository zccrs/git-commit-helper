use async_trait::async_trait;
use dialoguer::{Confirm, Select};
use log::{debug, info, warn};
use crate::config::{AIService, Config, AIServiceConfig};
use crate::translator::{Translator, Message};
use super::CopilotClient;

pub struct DeepSeekTranslator {
    api_key: String,
    endpoint: String,
    model: String,
}

pub struct OpenAITranslator {
    api_key: String,
    endpoint: String,
    model: String,
}

pub struct ClaudeTranslator {
    api_key: String,
    endpoint: String,
    model: String,
}

pub struct CopilotTranslator {
    client: CopilotClient,
    model: String,
}

pub struct GeminiTranslator {
    api_key: String,
    endpoint: String,
    model: String,
}

pub struct GrokTranslator {
    api_key: String,
    endpoint: String,
    model: String,
}

impl DeepSeekTranslator {
    pub fn new(config: &AIServiceConfig) -> Self {
        Self {
            api_key: config.api_key.clone(),
            endpoint: config.api_endpoint.clone()
                .unwrap_or_else(|| "https://api.deepseek.com/v1".into()),
            model: config.model.clone()
                .unwrap_or_else(|| "deepseek-chat".into()),
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
                .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1".into()),
            model: config.model.clone()
                .unwrap_or_else(|| "gemini-pro".into()),
        }
    }
}

impl GrokTranslator {
    pub fn new(config: &AIServiceConfig) -> Self {
        Self {
            api_key: config.api_key.clone(),
            endpoint: config.api_endpoint.clone()
                .unwrap_or_else(|| "https://api.grok.x.ai/v1".into()),
            model: config.model.clone()
                .unwrap_or_else(|| "grok-1".into()),
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
    if text.contains("diff --git") {
        // Git diff 内容的提示语
        return String::from(
            "Please analyze the git diff content and generate a detailed bilingual commit message with:
            1. First line in English: type: message (under 50 characters)
            2. Empty line after the title
            3. Detailed explanation in English (what was changed and why)
            4. Empty line after English explanation
            5. Chinese title and explanation (translate the English content)
            6. Type must be one of: feat/fix/docs/style/refactor/test/chore
            7. Focus on both WHAT changed and WHY it was necessary
            8. Include any important technical details or context
            9. Prefix the entire message with '[NO_TRANSLATE]' to prevent re-translation
            10. DO NOT wrap the response in any markdown, code block markers like

            Example response format:
            [NO_TRANSLATE]
            feat: add user authentication module

            - Implement JWT-based authentication system
            - Add user login and registration endpoints
            - Include password hashing with bcrypt
            - Set up token refresh mechanism

            feat: 添加用户认证模块

            - 实现基于 JWT 的认证系统
            - 添加用户登录和注册端点
            - 包含使用 bcrypt 的密码哈希处理
            - 设置令牌刷新机制

            Please respond with ONLY the commit message following this format, 
            DO NOT end commit titles with any punctuation.");
    }

    format!(
        r#"You are a professional translator. Please translate the following Chinese text to English. 
        Important rules:
        1. Keep all English content, numbers, and English punctuation unchanged
        2. Do not translate any content inside English double quotes
        3. Preserve the case of all English words
        4. For the original Chinese content, add line breaks to keep each line under 72 characters
        5. Return both the wrapped Chinese text and its English translation

        Text to translate:
        
        Chinese (wrapped):
        {}

        Please provide the English translation:"#,
        wrap_chinese_text(text, 72)
    )
}

fn extract_translation(response: &str) -> String {
    // 处理带有 NO_TRANSLATE 标记的内容
    if response.trim().starts_with("[NO_TRANSLATE]") {
        return response.trim()
            .strip_prefix("[NO_TRANSLATE]")
            .unwrap_or(response)
            .trim()
            .to_string();
    }

    // 查找最后一个 "English translation:" 后的内容
    if let Some(idx) = response.rfind("English translation:") {
        let translation = response[idx..].lines()
            .skip(1)  // 跳过 "English translation:" 行
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        return translation.trim().to_string();
    }
    response.trim().to_string()
}

#[async_trait]
impl Translator for DeepSeekTranslator {
    async fn translate(&self, text: &str) -> anyhow::Result<String> {
        debug!("使用 DeepSeek 进行翻译，API Endpoint: {}", self.endpoint);
        let client = reqwest::Client::new();
        let url = format!("{}/chat/completions", self.endpoint);
        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": get_translation_prompt(text)
                },
                {
                    "role": "user",
                    "content": text
                }
            ]
        });
        
        debug!("发送请求到: {}", url);
        debug!("请求体: {}", serde_json::to_string_pretty(&body).unwrap_or_default());

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?;

        debug!("收到响应: {:#?}", response);

        if !response.status().is_success() {
            let error_json = response.json::<serde_json::Value>().await?;
            debug!("响应内容: {}", serde_json::to_string_pretty(&error_json)?);
            return Err(anyhow::anyhow!("API 调用失败: {}", 
                error_json["error"]["message"].as_str().unwrap_or("未知错误")));
        }

        let result = response.json::<serde_json::Value>().await?;
        debug!("响应内容: {}", serde_json::to_string_pretty(&result)?);

        let translation = result["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or_default();
        Ok(extract_translation(translation))
    }
}

#[async_trait]
impl Translator for OpenAITranslator {
    async fn translate(&self, text: &str) -> anyhow::Result<String> {
        debug!("使用 OpenAI 进行翻译，API Endpoint: {}", self.endpoint);
        let client = reqwest::Client::new();
        let url = format!("{}/chat/completions", self.endpoint);
        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": get_translation_prompt(text)
                },
                {
                    "role": "user",
                    "content": text
                }
            ]
        });

        debug!("发送请求到: {}", url);
        debug!("请求体: {}", serde_json::to_string_pretty(&body).unwrap_or_default());

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?;

        debug!("收到响应: {:#?}", response);

        if !response.status().is_success() {
            let error_json = response.json::<serde_json::Value>().await?;
            debug!("响应内容: {}", serde_json::to_string_pretty(&error_json)?);
            return Err(anyhow::anyhow!("API 调用失败: {}", 
                error_json["error"]["message"].as_str().unwrap_or("未知错误")));
        }

        let result = response.json::<serde_json::Value>().await?;
        debug!("响应内容: {}", serde_json::to_string_pretty(&result)?);

        let translation = result["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or_default();
        Ok(extract_translation(translation))
    }
}

#[async_trait]
impl Translator for ClaudeTranslator {
    async fn translate(&self, text: &str) -> anyhow::Result<String> {
        debug!("使用 Claude 进行翻译，API Endpoint: {}", self.endpoint);
        let client = reqwest::Client::new();
        let url = format!("{}/messages", self.endpoint);
        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": get_translation_prompt(text)
                },
                {
                    "role": "user",
                    "content": text
                }
            ]
        });

        debug!("发送请求到: {}", url);
        debug!("请求体: {}", serde_json::to_string_pretty(&body).unwrap_or_default());

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?;

        debug!("收到响应: {:#?}", response);

        if !response.status().is_success() {
            let error_json = response.json::<serde_json::Value>().await?;
            debug!("响应内容: {}", serde_json::to_string_pretty(&error_json)?);
            return Err(anyhow::anyhow!("API 调用失败: {}", 
                error_json["error"]["message"].as_str().unwrap_or("未知错误")));
        }

        let result = response.json::<serde_json::Value>().await?;
        debug!("响应内容: {}", serde_json::to_string_pretty(&result)?);

        let translation = result["content"][0]["text"]
            .as_str()
            .unwrap_or_default();
        Ok(extract_translation(translation))
    }
}

#[async_trait]
impl Translator for CopilotTranslator {
    async fn translate(&self, text: &str) -> anyhow::Result<String> {
        debug!("使用 Copilot 进行翻译");
        let messages = vec![
            Message {
                role: "system".to_string(),
                content: get_translation_prompt(text).to_string(),
            },
            Message {
                role: "user".to_string(),
                content: text.to_string(),
            },
        ];
        let response = self.client.chat_completion(messages, self.model.clone()).await?;
        let translation = response.choices.get(0)
            .map(|choice| choice.message.content.clone())
            .unwrap_or_default();
        Ok(extract_translation(&translation))
    }
}

#[async_trait]
impl Translator for GeminiTranslator {
    async fn translate(&self, text: &str) -> anyhow::Result<String> {
        debug!("使用 Gemini 进行翻译，API Endpoint: {}", self.endpoint);
        let client = reqwest::Client::new();
        let url = format!("{}/models/{}:generateContent", self.endpoint, self.model);
        let body = serde_json::json!({
            "contents": [{
                "parts": [{
                    "text": get_translation_prompt(text)
                }]
            }]
        });

        debug!("发送请求到: {}", url);
        debug!("请求体: {}", serde_json::to_string_pretty(&body).unwrap_or_default());

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?;

        debug!("收到响应: {:#?}", response);

        if !response.status().is_success() {
            let error_json = response.json::<serde_json::Value>().await?;
            debug!("响应内容: {}", serde_json::to_string_pretty(&error_json)?);
            return Err(anyhow::anyhow!("API 调用失败: {}", 
                error_json["error"]["message"].as_str().unwrap_or("未知错误")));
        }

        let result = response.json::<serde_json::Value>().await?;
        debug!("响应内容: {}", serde_json::to_string_pretty(&result)?);

        let translation = result["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or_default();
        Ok(extract_translation(translation))
    }
}

#[async_trait]
impl Translator for GrokTranslator {
    async fn translate(&self, text: &str) -> anyhow::Result<String> {
        debug!("使用 Grok 进行翻译，API Endpoint: {}", self.endpoint);
        let client = reqwest::Client::new();
        let url = format!("{}/chat/completions", self.endpoint);
        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": get_translation_prompt(text)
                },
                {
                    "role": "user",
                    "content": text
                }
            ]
        });

        debug!("发送请求到: {}", url);
        debug!("请求体: {}", serde_json::to_string_pretty(&body).unwrap_or_default());

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?;

        debug!("收到响应: {:#?}", response);

        if !response.status().is_success() {
            let error_json = response.json::<serde_json::Value>().await?;
            debug!("响应内容: {}", serde_json::to_string_pretty(&error_json)?);
            return Err(anyhow::anyhow!("API 调用失败: {}", 
                error_json["error"]["message"].as_str().unwrap_or("未知错误")));
        }

        let result = response.json::<serde_json::Value>().await?;
        debug!("响应内容: {}", serde_json::to_string_pretty(&result)?);

        let translation = result["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or_default();
        Ok(extract_translation(translation))
    }
}

pub async fn create_translator(config: &Config) -> anyhow::Result<Box<dyn Translator>> {
    info!("创建 {:?} 翻译器", config.default_service);
    let service_config = config.services.iter()
        .find(|s| s.service == config.default_service)
        .ok_or_else(|| anyhow::anyhow!("找不到默认服务的配置"))?;
    create_translator_for_service(service_config).await
}

pub async fn translate_with_fallback(config: &Config, text: &str) -> anyhow::Result<String> {
    let mut tried_services = Vec::new();

    // 如果内容已经是双语的（带有 NO_TRANSLATE 标记），则直接返回
    if text.trim().starts_with("[NO_TRANSLATE]") {
        return Ok(text.trim().to_string());
    }

    debug!("尝试使用默认服务 {:?} 进行翻译", config.default_service);
    if let Some(result) = try_translate(&config.default_service, config, text).await {
        return result;
    }
    tried_services.push(config.default_service.clone());

    for service_config in &config.services {
        if tried_services.contains(&service_config.service) {
            continue;
        }

        debug!("尝试使用备选服务 {:?} 进行翻译", service_config.service);
        if let Some(result) = try_translate(&service_config.service, config, text).await {
            return result;
        }
        tried_services.push(service_config.service.clone());
    }

    while let Some(service) = select_retry_service(config, &tried_services)? {
        debug!("用户选择使用 {:?} 重试翻译", service);
        if let Some(result) = try_translate(&service, config, text).await {
            return result;
        }
        tried_services.push(service);
    }

    Err(anyhow::anyhow!("所有翻译服务均失败"))
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

pub async fn create_translator_for_service(config: &AIServiceConfig) -> anyhow::Result<Box<dyn Translator>> {
    Ok(match config.service {
        AIService::DeepSeek => Box::new(DeepSeekTranslator::new(config)),
        AIService::OpenAI => Box::new(OpenAITranslator::new(config)),
        AIService::Claude => Box::new(ClaudeTranslator::new(config)),
        AIService::Copilot => {
            let editor_version = "1.0.0".to_string();
            let client = CopilotClient::new_with_models(config.api_key.clone(), editor_version).await?;
            let model_id = config.model.clone().unwrap_or_else(|| "copilot-chat".to_string());
            Box::new(CopilotTranslator::new(client, model_id))
        },
        AIService::Gemini => Box::new(GeminiTranslator::new(config)),
        AIService::Grok => Box::new(GrokTranslator::new(config)),
    })
}
