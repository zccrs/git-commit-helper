use anyhow::Result;
use clap::{Parser, Subcommand};
use dialoguer::{Confirm, Select, Input};
use log::debug;
use std::path::{Path, PathBuf};
use crate::config::AIService;

mod config;
mod git;
mod translator;
mod install;
mod commit;
mod review;

#[derive(Parser)]
#[command(name = "git-commit-helper")]
#[command(author, version, about = "Git commit message helper", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(help = "Git commit message file path")]
    commit_msg_file: Option<PathBuf>,

    /// 禁用代码审查功能
    #[arg(long, global = true)]
    no_review: bool,
}

#[derive(Subcommand, PartialEq)]
enum Commands {
    /// 配置 AI 服务
    Config,
    /// 显示当前配置信息
    Show,
    /// 将工具安装到当前 git 仓库
    Install {
        /// 指定 git 仓库路径，默认为当前目录
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// 强制安装
        #[arg(short, long)]
        force: bool,
    },
    /// 管理 AI 服务配置
    #[command(name = "ai")]
    AI {
        #[command(subcommand)]
        command: ServiceCommands,
    },
    /// 翻译中文内容为英文
    Translate {
        /// 要翻译的文件路径
        #[arg(short, long)]
        file: Option<PathBuf>,

        /// 要翻译的文本内容
        #[arg(short, long)]
        text: Option<String>,

        /// 要翻译的内容，如果是一个存在的文件路径则作为文件处理，否则作为文本内容处理
        content: Option<String>,
    },
    /// 生成提交信息
    #[command(name = "commit")]
    Commit {
        /// 提交消息的类型
        #[arg(short, long)]
        r#type: Option<String>,

        /// 用户对改动的描述
        #[arg(short, long)]
        message: Option<String>,

        /// 自动添加所有已修改但未暂存的文件
        #[arg(short, long)]
        all: bool,
    },
    /// 管理 AI 代码审查功能
    #[command(name = "ai-review")]
    AIReview {
        /// 启用 AI 代码审查
        #[arg(long, group = "review_action")]
        enable: bool,

        /// 禁用 AI 代码审查
        #[arg(long, group = "review_action")]
        disable: bool,

        /// 显示当前 AI 代码审查状态
        #[arg(long, group = "review_action")]
        status: bool,
    },
}

#[derive(Subcommand, PartialEq)]
enum ServiceCommands {
    /// 添加新的 AI 服务
    Add,
    /// 修改已有的 AI 服务配置
    Edit,
    /// 删除 AI 服务
    Remove,
    /// 设置默认 AI 服务
    SetDefault,
    /// 列出所有AI服务
    List,
    /// 测试指定的AI服务
    Test {
        /// 测试用的中文文本
        #[arg(short, long, default_value = "这是一个测试消息。")]
        text: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // 根据编译模式设置默认日志级别
    let default_level = if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
    };

    // 初始化日志系统
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default_level))
        .format_timestamp(None)
        .format_module_path(false)
        .init();

    debug!("正在启动 git-commit-helper...");
    let cli = Cli::parse();

    // 检查配置文件
    if let Err(e) = config::Config::load() {
        if cli.command != Some(Commands::Config) {
            println!("错误: {}", e);
            println!("未检测到有效的 AI 配置，需要先进行配置");
            if Confirm::new()
                .with_prompt("是否现在进行配置？")
                .default(true)
                .interact()?
            {
                return config::Config::interactive_config().await;
            }
            return Err(anyhow::anyhow!("请先运行 'git-commit-helper config' 进行配置"));
        }
    }

    match cli.command {
        Some(Commands::Config) => {
            config::Config::interactive_config().await?;
            Ok(())
        }
        Some(Commands::Show) => {
            let config = config::Config::load()?;
            let config_path = config::Config::config_path()?;
            println!("配置文件路径: {}", config_path.display());
            println!("\n当前配置内容:");
            println!("默认 AI 服务: {:?}", config.default_service);
            println!("\n已配置的服务:");
            for (i, service) in config.services.iter().enumerate() {
                println!("{}. {:?}", i + 1, service.service);
                println!("   API Key: {}", service.api_key);
                if let Some(endpoint) = &service.api_endpoint {
                    println!("   API Endpoint: {}", endpoint);
                }
                if let Some(model) = &service.model {
                    println!("   Model: {}", model);
                }
            }
            Ok(())
        }
        Some(Commands::Install { path, force }) => {
            install::install_git_hook(path, force)?;
            Ok(())
        }
        Some(Commands::AI { command }) => {
            let mut config = config::Config::load().unwrap_or_else(|_| config::Config::new());
            match command {
                ServiceCommands::Add => {
                    let selected_service = {
                        println!("\n请选择要添加的 AI 服务:");
                        println!("1) DeepSeek");
                        println!("2) OpenAI");
                        println!("3) Claude");
                        println!("4) Copilot");
                        println!("5) Gemini");
                        println!("6) Grok");

                        let selection = Input::<String>::new()
                            .with_prompt("请输入对应的数字")
                            .report(true)
                            .validate_with(|input: &String| -> Result<(), &str> {
                                match input.parse::<usize>() {
                                    Ok(n) if n >= 1 && n <= 6 => Ok(()),
                                    _ => Err("请输入 1-6 之间的数字")
                                }
                            })
                            .interact()?
                            .parse::<usize>()?;

                        match selection {
                            1 => AIService::DeepSeek,
                            2 => AIService::OpenAI,
                            3 => AIService::Claude,
                            4 => AIService::Copilot,
                            5 => AIService::Gemini,
                            6 => AIService::Grok,
                            _ => unreachable!(),
                        }
                    };
                    config.add_service(selected_service).await
                }
                ServiceCommands::Edit => config.edit_service().await,
                ServiceCommands::Remove => config.remove_service().await,
                ServiceCommands::SetDefault => config.set_default_service().await,
                ServiceCommands::List => {
                    let config = config::Config::load()?;
                    println!("已配置的 AI 服务列表:");
                    for (i, service) in config.services.iter().enumerate() {
                        println!("[{}] {:?}{}",
                            i + 1,
                            service.service,
                            if service.service == config.default_service { " (默认)" } else { "" }
                        );
                    }
                    Ok(())
                }
                ServiceCommands::Test { text } => {
                    let config = config::Config::load()?;
                    if config.services.is_empty() {
                        return Err(anyhow::anyhow!("没有配置任何 AI 服务，请先添加服务"));
                    }

                    let service_names: Vec<String> = config.services
                        .iter()
                        .enumerate()
                        .map(|(i, s)| format!("[{}] {:?}{}",
                            i + 1,
                            s.service,
                            if s.service == config.default_service { " (默认)" } else { "" }
                        ))
                        .collect();

                    let selection = Select::new()
                        .with_prompt("请选择要测试的 AI 服务")
                        .items(&service_names)
                        .default(0)
                        .interact()?;

                    let service = &config.services[selection];
                    println!("正在测试 {:?} 服务...", service.service);

                    let translator = translator::ai_service::create_translator_for_service(service).await?;
                    debug!("开始发送翻译请求");
                    match translator.translate(&text).await {
                        Ok(result) => {
                            debug!("收到翻译响应");
                            println!("\n测试结果:");
                            println!("原文: {}", text);
                            if result.is_empty() {
                                println!("警告: 收到空的翻译结果！");
                            }
                            println!("译文: {}", result);
                            println!("\n测试成功！");
                            Ok(())
                        }
                        Err(e) => {
                            println!("\n测试失败！错误信息:");
                            println!("{}", e);
                            println!("\n请检查:");
                            println!("1. API Key 是否正确");
                            println!("2. API Endpoint 是否可访问");
                            println!("3. 网络连接是否正常");
                            println!("4. 查看日志获取详细信息（设置 RUST_LOG=debug）");
                            Err(e)
                        }
                    }
                }
            }
        }
        Some(Commands::Translate { file, text, content }) => {
            let config = config::Config::load()?;
            if config.services.is_empty() {
                return Err(anyhow::anyhow!("没有配置任何 AI 服务，请先添加服务"));
            }

            let content = if let Some(file_path) = file {
                std::fs::read_to_string(file_path)?
            } else if let Some(text) = text {
                text
            } else if let Some(content) = content {
                // 检查内容是否为文件路径
                let path = PathBuf::from(&content);
                if path.exists() && path.is_file() {
                    std::fs::read_to_string(path)?
                } else {
                    content
                }
            } else {
                return Err(anyhow::anyhow!("请提供要翻译的内容"));
            };

            let service = config.get_default_service()?;
            println!("正在使用 {:?} 服务进行翻译...", service.service);

            let translator = translator::ai_service::create_translator_for_service(service).await?;
            match translator.translate(&content).await {
                Ok(result) => {
                    println!("\n翻译结果:");
                    println!("原文: {}", content);
                    println!("译文: {}", result);
                    Ok(())
                }
                Err(e) => Err(e)
            }
        }
        Some(Commands::Commit { r#type, message, all }) => {
            commit::generate_commit_message(r#type, message, all).await
        }
        Some(Commands::AIReview { enable, disable, status }) => {
            let mut config = config::Config::load()?;

            if status {
                println!("AI 代码审查功能当前状态: {}",
                    if config.ai_review { "已启用" } else { "已禁用" });
                return Ok(());
            }

            if enable {
                config.ai_review = true;
                config.save()?;
                println!("已启用 AI 代码审查功能");
            } else if disable {
                config.ai_review = false;
                config.save()?;
                println!("已禁用 AI 代码审查功能");
            }

            Ok(())
        }
        None => {
            let commit_msg_path = cli.commit_msg_file.ok_or_else(|| {
                anyhow::anyhow!("Missing commit message file path")
            })?;
            let no_review = cli.no_review;
            git::process_commit_msg(&commit_msg_path, no_review).await
        }
    }
}

async fn process_commit_msg(args: &[String]) -> Result<()> {
    if args.len() != 2 {
        return Err(anyhow::anyhow!("参数数量不正确"));
    }

    let no_review = args.iter().any(|arg| arg == "--no-review");
    let commit_msg_file = Path::new(&args[1]);
    git::process_commit_msg(commit_msg_file, no_review).await?;
    Ok(())
}
