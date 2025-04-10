use anyhow::{Context, Result};
use dialoguer::{Confirm, Input, Select};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use crate::translator::ai_service;
use log::{debug, info, warn};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub default_service: AIService,
    pub services: Vec<AIServiceConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AIServiceConfig {
    pub service: AIService,
    pub api_key: String,
    pub api_endpoint: Option<String>,
    pub model: Option<String>,  // 新增字段
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AIService {
    DeepSeek,
    OpenAI,  // Changed from ChatGPT
    Claude,
    Copilot,
    Gemini,  // 新增
    Grok,    // 新增
}

impl Config {
    pub fn new() -> Self {
        Self {
            default_service: AIService::OpenAI, // Changed from ChatGPT
            services: Vec::new(),
        }
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        debug!("尝试加载配置文件: {}", config_path.display());
        
        if !config_path.exists() {  // 移除多余的括号
            warn!("配置文件不存在: {}", config_path.display());
            return Err(anyhow::anyhow!("配置文件不存在，请先运行 'git-commit-helper config' 进行配置"));
        }
        
        let config_str = fs::read_to_string(&config_path)
            .context("读取配置文件失败")?;
        let config: Config = serde_json::from_str(&config_str)
            .context("解析配置文件失败")?;
        
        info!("已加载配置，使用 {:?} 服务", config.default_service);
        
        Ok(config)
    }

    pub async fn interactive_config() -> Result<()> {
        Box::pin(Self::interactive_config_impl()).await
    }

    async fn interactive_config_impl() -> Result<()> {
        info!("开始交互式配置...");
        // 询问配置文件存放位置
        let default_path = Self::default_config_path()?;
        println!("\n配置文件存放位置选项:");
        println!("1) 系统默认位置: {}", default_path.display());
        println!("2) 自定义路径");

        let selection: usize = Input::new()
            .with_prompt("请选择配置文件存放位置")
            .validate_with(|input: &usize| -> Result<(), &str> {
                if *input >= 1 && *input <= 2 {
                    Ok(())
                } else {
                    Err("请输入 1-2 之间的数字")
                }
            })
            .interact()?;

        let config_path = if selection == 1 {
            default_path
        } else {
            let custom_path: String = Input::new()
                .with_prompt("请输入配置文件路径 (相对路径将基于可执行文件所在目录)")
                .interact_text()?;
            
            let path = PathBuf::from(&custom_path);
            if path.is_relative() {
                let exe_dir = std::env::current_exe()?
                    .parent()
                    .ok_or_else(|| anyhow::anyhow!("无法获取可执行文件目录"))?
                    .to_path_buf();
                exe_dir.join(path)
            } else {
                path
            }
        };

        // 设置环境变量，用于后续加载配置
        std::env::set_var("GIT_COMMIT_HELPER_CONFIG", config_path.to_string_lossy().to_string());

        let mut services: Vec<AIServiceConfig> = Vec::new();
        
        loop {
            println!("\n当前已配置的 AI 服务:");
            for (i, s) in services.iter().enumerate() {
                println!("{}. {:?}", i + 1, s.service);
            }

            if !Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
                .with_prompt("是否继续添加 AI 服务？")
                .default(services.is_empty())
                .interact()?
            {
                break;
            }

            println!("\n请选择要添加的 AI 服务:");
            println!("1) DeepSeek");
            println!("2) OpenAI");
            println!("3) Claude");
            println!("4) Copilot");

            let selection = Input::<String>::new()
                .with_prompt("请输入对应的数字")
                .report(true)
                .validate_with(|input: &String| -> Result<(), &str> {
                    match input.parse::<usize>() {
                        Ok(n) if n >= 1 && n <= 4 => Ok(()),
                        _ => Err("请输入 1-4 之间的数字")
                    }
                })
                .interact()?
                .parse::<usize>()?;

            let service = match selection {
                1 => AIService::DeepSeek,
                2 => AIService::OpenAI,
                3 => AIService::Claude,
                4 => AIService::Copilot,
                _ => unreachable!(),
            };

            let api_key: String = Input::new()
                .with_prompt("请输入 API Key")
                .interact_text()?;

            let api_endpoint: String = Input::new()
                .with_prompt("请输入 API Endpoint (可选，直接回车使用默认值)")
                .allow_empty(true)
                .interact_text()?;

            let config = AIServiceConfig {
                service: service.clone(),
                api_key,
                api_endpoint: if api_endpoint.is_empty() { None } else { Some(api_endpoint) },
                model: None,
            };

            services.push(config);
        }

        if services.is_empty() {
            return Err(anyhow::anyhow!("至少需要配置一个 AI 服务"));
        }

        println!("\n请选择默认的 AI 服务:");
        for (i, s) in services.iter().enumerate() {
            println!("{}. {:?}", i + 1, s.service);
        }

        let services_len = services.len();
        let default_index: usize = Input::new()
            .with_prompt("请输入对应的数字")
            .validate_with(|input: &usize| -> Result<(), &str> {
                if *input >= 1 && *input <= services_len {
                    Ok(())
                } else {
                    Err("输入的数字超出范围")
                }
            })
            .interact()?;

        let mut config = Config {
            default_service: services[default_index - 1].service.clone(),
            services,
        };

        // 确保配置目录存在
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // 保存配置
        fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;
        info!("配置已保存: {}", config_path.display());
        println!("配置已保存到: {}", config_path.display());

        // 询问是否进行测试
        if Confirm::new()
            .with_prompt("是否要测试翻译功能？")
            .default(true)
            .interact()? 
        {
            println!("正在测试翻译功能...");
            let translator = ai_service::create_translator(&config)?;
            match translator.translate("这是一个测试消息，用于验证翻译功能是否正常。").await {
                Ok(result) => {
                    println!("\n测试结果:");
                    println!("原文: 这是一个测试消息，用于验证翻译功能是否正常。");
                    println!("译文: {}\n", result);
                    println!("测试成功！配置已完成。");
                },
                Err(e) => {
                    println!("\n测试失败！错误信息:");
                    println!("{}", e);
                    println!("\n请检查以下内容:");
                    println!("1. API Key 是否正确");
                    println!("2. API Endpoint 是否可访问");
                    println!("3. 网络连接是否正常");

                    println!("\n请选择操作:");
                    let options = vec!["重新修改配置", "强制保存配置", "退出"];
                    match Select::new()
                        .with_prompt("选择操作")
                        .items(&options)
                        .interact()? 
                    {
                        0 => {
                            // 重新获取当前服务的配置
                            let new_config = Config::input_service_config(config.default_service.clone()).await?;
                            config.services.pop(); // 移除失败的配置
                            config.services.push(new_config); // 添加新配置
                            // 重新保存配置
                            fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;
                            // 递归调用测试，使用 Box::pin
                            return Box::pin(Config::interactive_config_impl()).await;
                        },
                        1 => {
                            println!("配置已强制保存，但可能无法正常工作。");
                            return Ok(());
                        },
                        _ => return Err(e),
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn add_service(&mut self, service: AIService) -> Result<()> {
        let config = Config::input_service_config(service).await?;
        if self.services.len() == 1 {
            self.default_service = config.service.clone();
        }
        self.services.push(config);
        
        self.save()?;
        info!("AI 服务添加成功");
        Ok(())
    }

    pub async fn edit_service(&mut self) -> Result<()> {
        if self.services.is_empty() {
            return Err(anyhow::anyhow!("没有可编辑的 AI 服务"));
        }

        println!("\n已配置的 AI 服务:");
        for (i, s) in self.services.iter().enumerate() {
            println!("{}. {:?}", i + 1, s.service);
        }

        let selection = Input::<String>::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("请输入要编辑的服务编号")
            .report(true)
            .interact()?
            .parse::<usize>()?;

        // 验证选择的服务编号是否有效
        if selection < 1 || selection > self.services.len() {
            return Err(anyhow::anyhow!("无效的服务编号"));
        }

        let old_config = &self.services[selection - 1];
        let new_config = Config::input_service_config_with_default(old_config).await?;
        self.services[selection - 1] = new_config;
        self.save()?;
        info!("AI 服务修改成功");
        Ok(())
    }

    pub async fn remove_service(&mut self) -> Result<()> {
        if self.services.is_empty() {
            return Err(anyhow::anyhow!("没有可删除的 AI 服务"));
        }

        println!("\n已配置的 AI 服务:");
        for (i, s) in self.services.iter().enumerate() {
            println!("{}. {:?}", i + 1, s.service);
        }

        let services_len = self.services.len();
        let selection = Input::<String>::new()
            .with_prompt("请输入要删除的服务编号")
            .report(true)
            .validate_with(|input: &String| -> Result<(), &str> {
                match input.parse::<usize>() {
                    Ok(n) if n >= 1 && n <= services_len => Ok(()),
                    _ => Err("输入的数字超出范围")
                }
            })
            .interact()?
            .parse::<usize>()?;

        let removed = self.services.remove(selection - 1);
        
        if removed.service == self.default_service && !self.services.is_empty() {
            self.default_service = self.services[0].service.clone();
        }

        self.save()?;
        info!("AI 服务删除成功");
        Ok(())
    }

    pub async fn set_default_service(&mut self) -> Result<()> {
        if self.services.is_empty() {
            return Err(anyhow::anyhow!("没有可选择的 AI 服务"));
        }

        println!("\n已配置的 AI 服务:");
        for (i, s) in self.services.iter().enumerate() {
            println!("{}. {:?}", i + 1, s.service);
        }

        let services_len = self.services.len();
        let selection = Input::<String>::new()
            .with_prompt("请输入要设为默认的服务编号")
            .report(true)
            .validate_with(|input: &String| -> Result<(), &str> {
                match input.parse::<usize>() {
                    Ok(n) if n >= 1 && n <= services_len => Ok(()),
                    _ => Err("输入的数字超出范围")
                }
            })
            .interact()?
            .parse::<usize>()?;

        self.default_service = self.services[selection - 1].service.clone();
        self.save()?;
        info!("默认 AI 服务设置成功");
        Ok(())
    }

    pub async fn input_service_config(service: AIService) -> Result<AIServiceConfig> {
        Config::input_service_config_with_default(&AIServiceConfig {
            service,
            api_key: String::new(),
            api_endpoint: None,
            model: None,
        }).await
    }

    pub async fn input_service_config_with_default(default: &AIServiceConfig) -> Result<AIServiceConfig> {
        let default_key = &default.api_key;
        let api_key: String = Input::new()
            .with_prompt("请输入 API Key")
            .with_initial_text(default_key)
            .interact_text()?;

        let default_endpoint = default.api_endpoint.as_deref().unwrap_or("");
        let api_endpoint: String = Input::new()
            .with_prompt("请输入 API Endpoint (可选，直接回车使用默认值)")
            .with_initial_text(default_endpoint)
            .allow_empty(true)
            .interact_text()?;

        let default_model = default.model.as_deref().unwrap_or("");
        let default_model_name = match default.service {
            AIService::DeepSeek => "deepseek-chat",
            AIService::OpenAI => "gpt-3.5-turbo",
            AIService::Claude => "claude-3-sonnet-20240229",
            AIService::Copilot => "copilot-chat",
            AIService::Gemini => "gemini-pro",
            AIService::Grok => "grok-1",
        };
        let model: String = Input::new()
            .with_prompt(format!("请输入模型名称 (可选，直接回车使用默认值) [{}]", default_model_name))
            .with_initial_text(default_model)
            .allow_empty(true)
            .interact_text()?;

        Ok(AIServiceConfig {
            service: default.service.clone(),
            api_key,
            api_endpoint: if api_endpoint.is_empty() { None } else { Some(api_endpoint) },
            model: if model.is_empty() { None } else { Some(model) },
        })
    }

    fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&config_path, serde_json::to_string_pretty(&self)?)?;
        Ok(())
    }

    pub fn config_path() -> Result<PathBuf> {
        if let Ok(path) = std::env::var("GIT_COMMIT_HELPER_CONFIG") {
            return Ok(PathBuf::from(path));
        }
        Self::default_config_path()
    }

    fn default_config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "githelper", "git-commit-helper")
            .context("无法确定配置文件路径")?;
        Ok(proj_dirs.config_dir().join("config.json"))
    }
}
