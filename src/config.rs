use anyhow::{Context, Result};
use dialoguer::{Confirm, Input};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use copilot_client::CopilotClient;
use copilot_client::get_github_token;
use log::{debug, info, warn};
use dialoguer::console::Term;
use crate::ai_service;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub default_service: AIService,
    pub services: Vec<AIServiceConfig>,
    #[serde(default = "default_ai_review")]
    pub ai_review: bool,  // 添加 AI Review 开关
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,  // 添加请求超时时间设置
    #[serde(default)]
    pub gerrit: Option<GerritConfig>,  // Gerrit 配置
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GerritConfig {
    pub username: Option<String>,
    pub password: Option<String>,
    pub token: Option<String>,
}

// 添加默认值函数
fn default_ai_review() -> bool {
    true
}

// 添加默认超时时间函数
fn default_timeout() -> u64 {
    20
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
            ai_review: true,  // 默认开启
            timeout_seconds: default_timeout(),
            gerrit: None,
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

    pub async fn setup_gerrit(&mut self) -> Result<()> {
        println!("\nGerrit 认证配置");
        println!("选择认证方式：");
        println!("1) 用户名密码");
        println!("2) Token");
        println!("3) 跳过 (不配置)");

        let selection: usize = Input::new()
            .with_prompt("请选择认证方式")
            .default(3)
            .validate_with(|input: &usize| -> Result<(), &str> {
                if *input >= 1 && *input <= 3 {
                    Ok(())
                } else {
                    Err("请输入 1-3 之间的数字")
                }
            })
            .interact()?;

        let mut gerrit_config = GerritConfig::default();

        match selection {
            1 => {
                let username: String = Input::new()
                    .with_prompt("请输入 Gerrit 用户名")
                    .interact_text()?;

                let password: String = Input::new()
                    .with_prompt("请输入 Gerrit 密码")
                    .interact_text()?;

                gerrit_config.username = Some(username);
                gerrit_config.password = Some(password);
            }
            2 => {
                let token: String = Input::new()
                    .with_prompt("请输入 Gerrit Token")
                    .interact_text()?;

                gerrit_config.token = Some(token);
            }
            _ => {
                // 不配置
                self.gerrit = None;
                return Ok(());
            }
        }

        self.gerrit = Some(gerrit_config);
        self.save()?;

        println!("✅ Gerrit 认证信息已保存");
        Ok(())
    }

    pub async fn interactive_config_impl() -> Result<()> {
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

            let config = Config::input_service_config(service).await?;
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
            ai_review: true,  // 默认开启
            timeout_seconds: default_timeout(),
            gerrit: None,
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
            // 创建一个临时的 Config 对象，确保只测试默认服务
            let test_config = Config {
                default_service: config.default_service.clone(),
                services: vec![config.services[default_index - 1].clone()],
                ai_review: true,
                timeout_seconds: config.timeout_seconds,
                gerrit: None,
            };
            let translator = ai_service::create_translator(&test_config).await?;
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
                    println!("1. 重新修改配置");
                    println!("2. 强制保存配置");
                    println!("3. 退出");

                    let selection: usize = Input::new()
                        .with_prompt("请输入对应的数字")
                        .validate_with(|input: &usize| -> Result<(), &str> {
                            if *input >= 1 && *input <= 3 {
                                Ok(())
                            } else {
                                Err("请输入 1-3 之间的数字")
                            }
                        })
                        .interact()?;

                    match selection {
                        1 => {
                            // 重新获取当前服务的配置
                            let new_config = Config::input_service_config(config.default_service.clone()).await?;
                            config.services.pop(); // 移除失败的配置
                            config.services.push(new_config); // 添加新配置
                            // 重新保存配置
                            fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;
                            // 递归调用测试，使用 Box::pin
                            return Box::pin(Config::interactive_config_impl()).await;
                        },
                        2 => {
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
        Box::pin(self.add_service_impl(service)).await
    }

    async fn add_service_impl(&mut self, service: AIService) -> Result<()> {
        // 获取服务配置
        let config = match service {
            AIService::Copilot => {
                println!("Copilot 服务需要 GitHub 身份验证...");

                // 尝试获取 GitHub token
                match get_github_token() {
                    Ok(token) => {
                        println!("✅ 已成功获取 GitHub 令牌");
                        // 尝试连接 Copilot API 验证令牌
                        let editor_version = "1.0.0".to_string();
                        let client = CopilotClient::new_with_models(token.clone(), editor_version).await;
                        match client {
                            Ok(client) => {
                                println!("✅ GitHub Copilot 认证成功！");
                                // 获取可用模型
                                let models = client.get_models().await?;
                                if !models.is_empty() {
                                    println!("\n可用模型:");
                                    for (i, model) in models.iter().enumerate() {
                                        println!("  {}. {} ({})", i+1, model.name, model.id);
                                    }

                                    // 让用户选择模型
                                    let model_count = models.len();
                                    let selection = Input::<String>::new()
                                        .with_prompt("请选择要使用的模型编号 (留空使用默认)")
                                        .allow_empty(true)
                                        .validate_with(|input: &String| -> Result<(), &str> {
                                            if input.is_empty() {
                                                return Ok(());
                                            }
                                            match input.parse::<usize>() {
                                                Ok(n) if n >= 1 && n <= model_count => Ok(()),
                                                _ => Err("请输入有效的模型编号或留空")
                                            }
                                        })
                                        .interact()?;

                                    // 处理用户选择
                                    let model_id = if selection.is_empty() {
                                        "copilot-chat".to_string()
                                    } else {
                                        let idx = selection.parse::<usize>().unwrap() - 1;
                                        models[idx].id.clone()
                                    };

                                    // 返回配置，使用用户选择的模型
                                    AIServiceConfig {
                                        service: AIService::Copilot,
                                        api_key: token,
                                        api_endpoint: None,
                                        model: Some(model_id),
                                    }
                                } else {
                                    // 如果没有可用模型列表，使用默认模型
                                    AIServiceConfig {
                                        service: AIService::Copilot,
                                        api_key: token,
                                        api_endpoint: None,
                                        model: Some("copilot-chat".to_string()),
                                    }
                                }
                            },
                            Err(e) => {
                                println!("❌ Copilot API 连接失败: {}", e);
                                println!("请确保您已订阅 GitHub Copilot 服务并拥有有效权限。");
                                return Err(anyhow::anyhow!("Copilot 认证失败"));
                            }
                        }
                    },
                    Err(e) => {
                        println!("❌ 无法获取 GitHub 令牌: {}", e);
                        println!("\n请按照以下步骤获取 GitHub 令牌:");
                        println!("可使用QtCreator中的Copilot插件获取到copilot的token，或直接使用copilot.nvim在nvim中获取token：https://github.com/github/copilot.vim");
                        println!("\n按回车键继续...");
                        Term::stdout().read_line()?;
                        return Err(anyhow::anyhow!("无法获取 GitHub 令牌"));
                    }
                }
            },
            _ => Config::input_service_config_with_default(&AIServiceConfig {
                service: service.clone(),
                api_key: String::new(),
                api_endpoint: None,
                model: None,
            }).await?,
        };

        // 添加服务
        if self.services.is_empty() {
            self.default_service = config.service.clone();
        }
        self.services.push(config.clone());

        // 提供测试选项
        if Confirm::new()
            .with_prompt("是否要测试该服务？")
            .default(true)
            .interact()?
        {
            println!("正在测试 {:?} 服务...", config.service);
            // 创建一个临时的 Config 对象，只包含要测试的新服务
            let test_config = Config {
                default_service: config.service.clone(),
                services: vec![config.clone()],
                ai_review: true,
                timeout_seconds: self.timeout_seconds,
                gerrit: None,
            };
            let translator = ai_service::create_translator(&test_config).await?;
            let text = "这是一个测试消息，用于验证翻译功能是否正常。";
            debug!("开始发送翻译请求");
            match translator.translate(text).await {
                Ok(result) => {
                    debug!("收到翻译响应");
                    println!("\n测试结果:");
                    println!("原文: {}", text);
                    if result.is_empty() {
                        println!("警告: 收到空的翻译结果！");
                    }
                    println!("译文: {}", result);
                    println!("\n✅ 测试成功！服务已添加并可正常使用。");
                    self.save()?;
                },
                Err(e) => {
                    println!("\n❌ 测试失败！错误信息:");
                    println!("{}", e);
                    println!("\n请检查:");
                    println!("1. API Key 是否正确");
                    println!("2. API Endpoint 是否可访问");
                    println!("3. 网络连接是否正常");
                    println!("4. 查看日志获取详细信息（设置 RUST_LOG=debug）");

                    println!("\n请选择操作:");
                    println!("1. 重新配置服务");
                    println!("2. 强制保存配置");
                    println!("3. 放弃添加");

                    let selection: usize = Input::new()
                        .with_prompt("请输入对应的数字")
                        .validate_with(|input: &usize| -> Result<(), &str> {
                            if *input >= 1 && *input <= 3 {
                                Ok(())
                            } else {
                                Err("请输入 1-3 之间的数字")
                            }
                        })
                        .interact()?;

                    match selection {
                        1 => {
                            // 移除刚添加的服务
                            self.services.pop();
                            // 使用 Box::pin 包装递归调用
                            return Box::pin(self.add_service_impl(service)).await;
                        },
                        2 => {
                            println!("配置已强制保存，但服务可能无法正常工作。");
                            self.save()?;
                        },
                        _ => {
                            self.services.pop(); // 移除失败的服务
                            return Err(anyhow::anyhow!("已取消添加服务"));
                        }
                    }
                }
            }
        } else {
            self.save()?;
            println!("✅ {:?} 服务已添加（未测试）", service);
        }

        info!("AI 服务已添加");
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

        // 不进行测试，直接更新服务
        self.services[selection - 1] = new_config;
        self.save()?;

        println!("✅ 服务配置已更新。请稍后使用 'git-commit-helper test' 命令测试该服务。");
        info!("AI 服务已修改（未测试）");

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
        // 对于除 Copilot 以外的服务，使用默认逻辑
        Config::input_service_config_with_default(&AIServiceConfig {
            service,
            api_key: String::new(),
            api_endpoint: None,
            model: None,
        }).await
    }

    pub async fn input_service_config_with_default(default: &AIServiceConfig) -> Result<AIServiceConfig> {
        // 如果是 Copilot 服务，使用特殊处理
        if default.service == AIService::Copilot {
            // 为已存在的 Copilot 配置，只询问模型
            if !default.api_key.is_empty() {
                // 尝试连接 Copilot API 获取可用模型
                let editor_version = "1.0.0".to_string();
                match CopilotClient::new_with_models(default.api_key.clone(), editor_version).await {
                    Ok(client) => {
                        let models = client.get_models().await?;
                        if !models.is_empty() {
                            println!("\n可用模型:");
                            for (i, model) in models.iter().enumerate() {
                                println!("  {}. {} ({})", i+1, model.name, model.id);
                            }

                            // 显示当前选择的模型
                            let current_model = default.model.as_deref().unwrap_or("copilot-chat");
                            println!("\n当前选择的模型: {}", current_model);

                            // 让用户选择模型
                            let model_count = models.len();
                            let selection = Input::<String>::new()
                                .with_prompt("请选择要使用的模型编号 (留空保持当前选择)")
                                .allow_empty(true)
                                .validate_with(|input: &String| -> Result<(), &str> {
                                    if input.is_empty() {
                                        return Ok(());
                                    }
                                    match input.parse::<usize>() {
                                        Ok(n) if n >= 1 && n <= model_count => Ok(()),
                                        _ => Err("请输入有效的模型编号或留空")
                                    }
                                })
                                .interact()?;

                            // 处理用户选择
                            let model_id = if selection.is_empty() {
                                default.model.clone().unwrap_or_else(|| "copilot-chat".to_string())
                            } else {
                                let idx = selection.parse::<usize>().unwrap() - 1;
                                models[idx].id.clone()
                            };

                            return Ok(AIServiceConfig {
                                service: default.service.clone(),
                                api_key: default.api_key.clone(),
                                api_endpoint: None,
                                model: Some(model_id),
                            });
                        }
                    },
                    Err(e) => {
                        println!("⚠️ 无法获取模型列表: {}", e);
                        println!("将使用之前配置的模型或默认模型。");
                    }
                }

                let model: String = Input::new()
                    .with_prompt("请输入模型名称 (可选，直接回车使用默认值) [copilot-chat]")
                    .with_initial_text(default.model.as_deref().unwrap_or("copilot-chat"))
                    .allow_empty(true)
                    .interact_text()?;

                return Ok(AIServiceConfig {
                    service: default.service.clone(),
                    api_key: default.api_key.clone(),  // 保留原有 token
                    api_endpoint: None,
                    model: if model.is_empty() { Some("copilot-chat".to_string()) } else { Some(model) },
                });
            } else {
                // 如果没有 API key，需要重新验证
                // 使用 Box::pin 解决递归调用问题
                return Box::pin(Config::input_service_config(AIService::Copilot)).await;
            }
        }

        // 非 Copilot 服务需要 API Key
        let api_key: String = Input::new()
            .with_prompt("请输入 API Key")
            .with_initial_text(&default.api_key)
            .interact_text()?;

        let default_endpoint = match default.service {
            AIService::DeepSeek => "https://api.deepseek.com/v1",
            AIService::OpenAI => "https://api.openai.com/v1",
            AIService::Claude => "https://api.anthropic.com/v1",
            AIService::Copilot => "",  // Copilot 不需要 endpoint
            AIService::Gemini => "https://generativelanguage.googleapis.com/v1beta",
            AIService::Grok => "https://api.grok.x.ai/v1",
        };
        let api_endpoint: String = Input::new()
            .with_prompt(format!("请输入 API Endpoint (可选，直接回车使用默认值) [{}]", default_endpoint))
            .with_initial_text(default.api_endpoint.as_deref().unwrap_or(""))
            .allow_empty(true)
            .interact_text()?;

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
            .with_initial_text(default.model.as_deref().unwrap_or(""))
            .allow_empty(true)
            .interact_text()?;

        Ok(AIServiceConfig {
            service: default.service.clone(),
            api_key,
            api_endpoint: if api_endpoint.is_empty() { None } else { Some(api_endpoint) },
            model: if model.is_empty() { None } else { Some(model) },
        })
    }

    pub fn get_default_service(&self) -> Result<&AIServiceConfig> {
        if self.services.is_empty() {
            return Err(anyhow::anyhow!("没有配置任何 AI 服务"));
        }

        // 查找默认服务
        if let Some(service) = self.services.iter().find(|s| s.service == self.default_service) {
            return Ok(service);
        }

        // 如果没有设置默认服务或默认服务不存在，返回第一个服务
        Ok(&self.services[0])
    }

    pub fn save(&self) -> Result<()> {
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
