use async_trait::async_trait;
use dialoguer::{Confirm, Select};
use log::{debug, info, warn};
use crate::config::{AIService, Config, AIServiceConfig};
use crate::translator::Translator;

pub struct DeepSeekTranslator {
    api_key: String,
    endpoint: String,
}

pub struct ChatGPTTranslator {
    api_key: String,
    endpoint: String,
}

pub struct ClaudeTranslator {
    api_key: String,
    endpoint: String,
}

pub struct CopilotTranslator {
    api_key: String,
    endpoint: String,
}

impl DeepSeekTranslator {
    pub fn new(config: &AIServiceConfig) -> Self {
        Self {
            api_key: config.api_key.clone(),
            endpoint: config.api_endpoint.clone()
                .unwrap_or_else(|| "https://api.deepseek.com/v1".into()),
        }
    }
}

impl ChatGPTTranslator {
    pub fn new(config: &AIServiceConfig) -> Self {
        Self {
            api_key: config.api_key.clone(),
            endpoint: config.api_endpoint.clone()
                .unwrap_or_else(|| "https://api.openai.com/v1".into()),
        }
    }
}

impl ClaudeTranslator {
    pub fn new(config: &AIServiceConfig) -> Self {
        Self {
            api_key: config.api_key.clone(),
            endpoint: config.api_endpoint.clone()
                .unwrap_or_else(|| "https://api.anthropic.com/v1".into()),
        }
    }
}

impl CopilotTranslator {
    pub fn new(config: &AIServiceConfig) -> Self {
        Self {
            api_key: config.api_key.clone(),
            endpoint: config.api_endpoint.clone()
                .unwrap_or_else(|| "https://api.github.com/copilot/v1".into()),
        }
    }
}

#[async_trait]
impl Translator for DeepSeekTranslator {
    async fn translate(&self, text: &str) -> anyhow::Result<String> {
        debug!("使用 DeepSeek 进行翻译，API Endpoint: {}", self.endpoint);
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/chat/completions", self.endpoint))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "messages": [
                    {
                        "role": "system",
                        "content": "You are a professional translator. Translate the following Chinese text to English. Keep the translation accurate and natural."
                    },
                    {
                        "role": "user",
                        "content": text
                    }
                ]
            }))
            .send()
            .await?;

        let result = response.json::<serde_json::Value>().await?;
        Ok(result["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or_default()
            .to_string())
    }
}

#[async_trait]
impl Translator for ChatGPTTranslator {
    async fn translate(&self, text: &str) -> anyhow::Result<String> {
        debug!("使用 ChatGPT 进行翻译，API Endpoint: {}", self.endpoint);
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/chat/completions", self.endpoint))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": "gpt-3.5-turbo",
                "messages": [
                    {
                        "role": "system",
                        "content": "You are a professional translator. Translate the following Chinese text to English. Keep the translation accurate and natural."
                    },
                    {
                        "role": "user",
                        "content": text
                    }
                ]
            }))
            .send()
            .await?;

        let result = response.json::<serde_json::Value>().await?;
        Ok(result["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or_default()
            .to_string())
    }
}

#[async_trait]
impl Translator for ClaudeTranslator {
    async fn translate(&self, text: &str) -> anyhow::Result<String> {
        debug!("使用 Claude 进行翻译，API Endpoint: {}", self.endpoint);
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/messages", self.endpoint))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("anthropic-version", "2023-06-01")
            .json(&serde_json::json!({
                "model": "claude-2",
                "messages": [
                    {
                        "role": "user",
                        "content": format!("Translate the following Chinese text to English: {}", text)
                    }
                ]
            }))
            .send()
            .await?;

        let result = response.json::<serde_json::Value>().await?;
        Ok(result["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .to_string())
    }
}

#[async_trait]
impl Translator for CopilotTranslator {
    async fn translate(&self, text: &str) -> anyhow::Result<String> {
        debug!("使用 Copilot 进行翻译，API Endpoint: {}", self.endpoint);
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/chat", self.endpoint))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "messages": [
                    {
                        "role": "system",
                        "content": "You are a professional translator. Translate the following Chinese text to English. Keep the translation accurate and natural."
                    },
                    {
                        "role": "user",
                        "content": text
                    }
                ]
            }))
            .send()
            .await?;

        let result = response.json::<serde_json::Value>().await?;
        Ok(result["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or_default()
            .to_string())
    }
}

pub fn create_translator(config: &Config) -> anyhow::Result<Box<dyn Translator>> {
    info!("创建 {:?} 翻译器", config.default_service);
    let service_config = config.services.iter()
        .find(|s| s.service == config.default_service)
        .ok_or_else(|| anyhow::anyhow!("找不到默认服务的配置"))?;
    create_translator_for_service(service_config)
}

pub async fn translate_with_fallback(config: &Config, text: &str) -> anyhow::Result<String> {
    let mut tried_services = Vec::new();

    debug!("尝试使用默认服务 {:?} 进行翻译", config.default_service);
    if let Some(result) = try_translate(&config.default_service, config, text).await {
        return result;
    }
    tried_services.push(config.default_service.clone());

    // 依次尝试其他服务
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

    // 所有自动尝试都失败了，询问用户选择重试
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

    let translator = create_translator_for_service(service_config).ok()?;
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
    if !Confirm::new().default(true).interact()? {
        return Ok(None);
    }

    let selection = Select::new()
        .with_prompt("请选择要使用的服务")
        .items(&options)
        .default(0)
        .interact()?;

    Ok(Some(available_services[selection].service.clone()))
}

pub fn create_translator_for_service(config: &AIServiceConfig) -> anyhow::Result<Box<dyn Translator>> {
    Ok(match config.service {
        AIService::DeepSeek => Box::new(DeepSeekTranslator::new(config)),
        AIService::ChatGPT => Box::new(ChatGPTTranslator::new(config)),
        AIService::Claude => Box::new(ClaudeTranslator::new(config)),
        AIService::Copilot => Box::new(CopilotTranslator::new(config)),
    })
}
