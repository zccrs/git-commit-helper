use async_trait::async_trait;
use dialoguer::{Confirm, Select};
use log::{debug, info, warn};
use crate::config::{AIService, Config, AIServiceConfig};
use crate::translator::{Translator, Message};
use copilot_client::{CopilotClient};

pub struct DeepSeekTranslator {
    api_key: String,
    endpoint: String,
    model: String,
    timeout_seconds: u64,
}

pub struct OpenAITranslator {
    api_key: String,
    endpoint: String,
    model: String,
    timeout_seconds: u64,
}

pub struct ClaudeTranslator {
    api_key: String,
    endpoint: String,
    model: String,
    timeout_seconds: u64,
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
}

pub struct GrokTranslator {
    api_key: String,
    endpoint: String,
    model: String,
    timeout_seconds: u64,
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
                .unwrap_or_else(|| "gemini-pro".into()),
            timeout_seconds: crate::config::Config::load()
                .map(|c| c.timeout_seconds)
                .unwrap_or(20),
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
            timeout_seconds: crate::config::Config::load()
                .map(|c| c.timeout_seconds)
                .unwrap_or(20),
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
    let prompt = if text.contains("diff --git") {
        // Git diff 内容的提示语
        String::from(
            "Please analyze the git diff content and generate a detailed bilingual commit message with:
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
            DO NOT end commit titles with any punctuation.")
    } else {
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
        wrap_chinese_text(text, 72))
    };

    debug!("生成的提示词:\n{}", prompt);
    prompt
}

fn extract_translation(response: &str) -> String {
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
        debug!("使用 DeepSeek，API Endpoint: {}", self.endpoint);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(self.timeout_seconds))
            .build()?;

        let url = format!("{}/chat/completions", self.endpoint);
        let messages = vec![
            serde_json::json!({
                "role": "system",
                "content": get_translation_prompt(text)
            }),
            serde_json::json!({
                "role": "user",
                "content": text
            })
        ];
        debug!("发送给 DeepSeek 的消息:\n{}", serde_json::to_string_pretty(&messages)?);
        let body = serde_json::json!({
            "model": self.model,
            "messages": messages
        });
        let api_key = self.api_key.clone();

        loop {
            match client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&body)
                .send()
                .await
            {
                Ok(response) => {
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
                    return Ok(extract_translation(translation));
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
impl Translator for OpenAITranslator {
    async fn translate(&self, text: &str) -> anyhow::Result<String> {
        debug!("使用 OpenAI，API Endpoint: {}", self.endpoint);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(self.timeout_seconds))
            .build()?;

        let url = format!("{}/chat/completions", self.endpoint);
        let messages = vec![
            serde_json::json!({
                "role": "system",
                "content": get_translation_prompt(text)
            }),
            serde_json::json!({
                "role": "user",
                "content": text
            })
        ];
        debug!("发送给 OpenAI 的消息:\n{}", serde_json::to_string_pretty(&messages)?);
        let body = serde_json::json!({
            "model": self.model,
            "messages": messages
        });
        let api_key = self.api_key.clone();

        loop {
            match client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&body)
                .send()
                .await
            {
                Ok(response) => {
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
                    return Ok(extract_translation(translation));
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
impl Translator for ClaudeTranslator {
    async fn translate(&self, text: &str) -> anyhow::Result<String> {
        debug!("使用 Claude，API Endpoint: {}", self.endpoint);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(self.timeout_seconds))
            .build()?;

        let url = format!("{}/messages", self.endpoint);
        let messages = vec![
            serde_json::json!({
                "role": "system",
                "content": get_translation_prompt(text)
            }),
            serde_json::json!({
                "role": "user",
                "content": text
            })
        ];
        debug!("发送给 Claude 的消息:\n{}", serde_json::to_string_pretty(&messages)?);
        let body = serde_json::json!({
            "model": self.model,
            "messages": messages
        });
        let api_key = self.api_key.clone();

        loop {
            match client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("anthropic-version", "2023-06-01")
                .json(&body)
                .send()
                .await
            {
                Ok(response) => {
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
                    return Ok(extract_translation(translation));
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
impl Translator for CopilotTranslator {
    async fn translate(&self, text: &str) -> anyhow::Result<String> {
        debug!("使用 Copilot");
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
        debug!("发送给 Copilot 的消息:\n{}", serde_json::to_string_pretty(&messages)?);
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
        debug!("使用 Gemini，API Endpoint: {}", self.endpoint);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(self.timeout_seconds))
            .build()?;

        let url = format!("{}/models/{}:generateContent?key={}", self.endpoint, self.model, self.api_key);
        let prompt = get_translation_prompt(text);
        debug!("发送给 Gemini 的内容:\n{}", prompt);
        let body = serde_json::json!({
            "contents": [{
                "parts": [{
                    "text": prompt
                }]
            }]
        });

        loop {
            match client
                .post(&url)
                .json(&body)
                .send()
                .await
            {
                Ok(response) => {
                    debug!("收到响应: {:#?}", response);

                    if !response.status().is_success() {
                        let error_json = response.json::<serde_json::Value>().await?;
                        debug!("响应内容: {}", serde_json::to_string_pretty(&error_json)?);
                        return Err(anyhow::anyhow!("API 调用失败: {}",
                            error_json["error"]["message"].as_str().unwrap_or("未知错误")));
                    }

                    let result = response.json::<serde_json::Value>().await?;
                    debug!("响应内容: {}", serde_json::to_string_pretty(&result)?);

                    let translation = result["candidates"][0]["output"]
                        .as_str()
                        .unwrap_or_default();
                    return Ok(extract_translation(translation));
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
impl Translator for GrokTranslator {
    async fn translate(&self, text: &str) -> anyhow::Result<String> {
        debug!("使用 Grok，API Endpoint: {}", self.endpoint);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(self.timeout_seconds))
            .build()?;

        let url = format!("{}/chat/completions", self.endpoint);
        let messages = vec![
            serde_json::json!({
                "role": "system",
                "content": get_translation_prompt(text)
            }),
            serde_json::json!({
                "role": "user",
                "content": text
            })
        ];
        debug!("发送给 Grok 的消息:\n{}", serde_json::to_string_pretty(&messages)?);
        let body = serde_json::json!({
            "model": self.model,
            "messages": messages
        });
        let api_key = self.api_key.clone();

        loop {
            match client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&body)
                .send()
                .await
            {
                Ok(response) => {
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
                    return Ok(extract_translation(translation));
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
    })
}
