use anyhow::Result;
use clap::{Parser, Subcommand};
use dialoguer::{Confirm, Select, Input};
use log::debug;
use std::path::PathBuf;
use crate::config::AIService;
mod terminal_format;
use terminal_format::Style;

mod config;
mod git;
mod github;
mod gerrit;
mod install;
mod commit;
mod review;
mod ai_service;

#[derive(Parser)]
#[command(name = "git-commit-helper")]
#[command(author, version, about = "Git commit message helper", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Git commit message file path 或 GitHub/Gerrit URL 或 commit id
    #[arg(help = "Git commit message file path or GitHub/Gerrit URL or commit id (7-40 chars)")]
    input: Option<String>,

    /// 禁用代码审查功能
    #[arg(long, global = true)]
    no_review: bool,
}

#[derive(Subcommand, PartialEq)]
enum Commands {
    /// 配置 AI 服务
    Config {
        /// 设置默认是否只使用中文提交信息，true: 仅中文，false: 中英双语
        #[arg(long = "set-only-chinese", help = "设置是否默认只使用中文提交信息，true: 仅中文，false: 中英双语")]
        only_chinese: Option<bool>,
        /// 设置默认是否只使用英文提交信息，true: 仅英文，false: 中英双语
        #[arg(long = "set-only-english", help = "设置是否默认只使用英文提交信息，true: 仅英文，false: 中英双语")]
        only_english: Option<bool>,
    },
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
        /// 要翻译的内容
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
        /// 不翻译提交信息
        #[arg(long)]
        no_translate: bool,
        /// 仅保留中文提交信息
        #[arg(long = "only-chinese")]
        only_chinese: bool,
        /// 仅保留英文提交信息
        #[arg(long = "only-english")]
        only_english: bool,
        /// 禁用测试建议
        #[arg(long)]
        no_test_suggestions: bool,
        /// 关联的GitHub issue或PMS链接
        #[arg(long, value_delimiter = ' ', num_args = 0..)]
        issues: Vec<String>,
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
    #[command(name = "set-default")]
    SetDefault,
    /// 设置网络请求超时时间
    #[command(name = "set-timeout")]
    SetTimeout {
        /// 超时时间（单位：秒）
        #[arg(short, long)]
        seconds: u64,
    },
    /// 列出所有 AI 服务
    List,
    /// 测试指定的 AI 服务
    Test {
        /// 测试用的中文文本
        #[arg(short, long)]
        text: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Add dynamic completion support
    clap_complete::CompleteEnv::with_factory(<Cli as clap::CommandFactory>::command)
        .complete();

    let default_level = if cfg!(debug_assertions) { "debug" } else { "warn" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default_level))
        .format_timestamp(None)
        .format_module_path(false)
        .init();

    debug!("正在启动 git-commit-helper...");
    let cli = Cli::parse();

    // 检查当前命令是否需要 Gerrit 认证
    let needs_gerrit = match &cli.input {
        Some(input) if input.contains("/+/") => true,
        _ => false,
    };

    // 加载配置文件
    let _config = match config::Config::load() {
        Ok(mut config) => {
            // 如果是 Gerrit URL 且没有配置认证信息，提示用户配置
            if needs_gerrit && config.gerrit.is_none() {
                println!("检测到 Gerrit URL，但未配置 Gerrit 认证信息。");
                if Confirm::new()
                    .with_prompt("是否现在配置 Gerrit 认证？")
                    .default(true)
                    .interact()?
                {
                    config.setup_gerrit().await?;
                }
            }
            config
        }
        Err(e) => {
            // 检查当前命令是否为 config，如果是则允许继续
            if let Some(Commands::Config { .. }) = cli.command {
                config::Config::new()
            } else {
                println!("{}", Style::red(&format!("错误: {}", e)));
                println!("{}", Style::yellow("未检测到有效的 AI 配置，需要先进行配置"));
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
    };

    match cli.command {
        Some(Commands::Config { only_chinese, only_english }) => {
            if let Some(only_chinese) = only_chinese {
                let mut config = config::Config::load().unwrap_or_else(|_| config::Config::new());
                config.only_chinese = only_chinese;
                if only_chinese {
                    config.only_english = false; // 如果设置为仅中文，则清除仅英文标志
                }
                config.save()?;
                let language_mode = if config.only_chinese {
                    "仅中文"
                } else if config.only_english {
                    "仅英文"
                } else {
                    "中英双语"
                };
                println!("{}", Style::green(&format!("已将默认提交信息语言设置为: {}", language_mode)));
                Ok(())
            } else if let Some(only_english) = only_english {
                let mut config = config::Config::load().unwrap_or_else(|_| config::Config::new());
                config.only_english = only_english;
                if only_english {
                    config.only_chinese = false; // 如果设置为仅英文，则清除仅中文标志
                }
                config.save()?;
                let language_mode = if config.only_chinese {
                    "仅中文"
                } else if config.only_english {
                    "仅英文"
                } else {
                    "中英双语"
                };
                println!("{}", Style::green(&format!("已将默认提交信息语言设置为: {}", language_mode)));
                Ok(())
            } else {
                config::Config::interactive_config().await?;
                Ok(())
            }
        }
        Some(Commands::Show) => {
            let config = config::Config::load()?;
            let config_path = config::Config::config_path()?;
            println!("{}", Style::title(&format!("配置文件路径: {}", config_path.display())));
            println!("{}", Style::separator());
            println!("{}", Style::title("当前配置内容:"));
            println!("{}", Style::plain(&format!("默认 AI 服务: {:?}", config.default_service)));
            println!("{}", Style::title("已配置的服务:"));
            for (i, service) in config.services.iter().enumerate() {
                println!("{}", Style::plain(&format!("{}. {:?}", i + 1, service.service)));
                println!("{}", Style::plain(&format!("   API Key: {}", service.api_key)));
                if let Some(endpoint) = &service.api_endpoint {
                    println!("{}", Style::plain(&format!("   API Endpoint: {}", endpoint)));
                }
                if let Some(model) = &service.model {
                    println!("{}", Style::plain(&format!("   Model: {}", model)));
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
                        println!("7) Qwen");

                        let selection = Input::<String>::new()
                            .with_prompt("请输入对应的数字")
                            .report(true)
                            .validate_with(|input: &String| -> Result<(), &str> {
                                match input.parse::<usize>() {
                                    Ok(n) if n >= 1 && n <= 7 => Ok(()),
                                    _ => Err("请输入 1-7 之间的数字")
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
                            7 => AIService::Qwen,
                            _ => unreachable!(),
                        }
                    };
                    config.add_service(selected_service).await
                }
                ServiceCommands::Edit => config.edit_service().await,
                ServiceCommands::Remove => config.remove_service().await,
                ServiceCommands::SetDefault => config.set_default_service().await,
                ServiceCommands::SetTimeout { seconds } => {
                    config.timeout_seconds = seconds;
                    config.save()?;
                    println!("{}", Style::green(&format!("已将网络请求超时时间设置为 {} 秒", seconds)));
                    Ok(())
                }
                ServiceCommands::List => {
                    let config = config::Config::load()?;
                    println!("{}", Style::title("已配置的 AI 服务列表:"));
                    for (i, service) in config.services.iter().enumerate() {
                        println!("{}", Style::plain(&format!("[{}] {:?}{}", i + 1, service.service, if service.service == config.default_service { " (默认)" } else { "" })));
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
                    println!("{}", Style::title(&format!("正在测试 {:?} 服务...", service.service)));

                    let translator = ai_service::create_translator_for_service(service).await?;
                    let test_text = text.unwrap_or_else(|| "这是一个测试消息，用于验证翻译功能是否正常。".to_string());
                    debug!("开始发送翻译请求");
                    match translator.translate(&test_text).await {
                        Ok(result) => {
                            debug!("收到翻译响应");
                            println!("{}", Style::separator());
                            println!("{}", Style::title("测试结果:"));
                            println!("{}", Style::plain(&format!("原文: {}", test_text)));
                            if result.is_empty() {
                                println!("{}", Style::yellow("警告: 收到空的翻译结果！"));
                            }
                            println!("{}", Style::green(&format!("译文: {}", result)));
                            println!("{}", Style::green("测试成功！"));
                            Ok(())
                        }
                        Err(e) => {
                            println!("{}", Style::separator());
                            println!("{}", Style::red("测试失败！错误信息:"));
                            println!("{}", Style::red(&format!("{}", e)));
                            println!("{}", Style::yellow("请检查:"));
                            println!("{}", Style::plain("1. API Key 是否正确"));
                            println!("{}", Style::plain("2. API Endpoint 是否可访问"));
                            println!("{}", Style::plain("3. 网络连接是否正常"));
                            println!("{}", Style::plain("4. 查看日志获取详细信息（设置 RUST_LOG=debug）"));
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
            println!("{}", Style::title(&format!("正在使用 {:?} 服务进行翻译...", service.service)));

            let translator = ai_service::create_translator_for_service(service).await?;
            match translator.translate(&content).await {
                Ok(result) => {
                    println!("{}", Style::separator());
                    println!("{}", Style::title("翻译结果:"));
                    println!("{}", Style::plain(&format!("原文: {}", content)));
                    println!("{}", Style::green(&format!("译文: {}", result)));
                    Ok(())
                }
                Err(e) => Err(e)
            }
        }
        Some(Commands::Commit { r#type, message, all, no_translate, only_chinese, only_english, no_test_suggestions, issues }) => {
            let issues_str = if issues.is_empty() {
                None
            } else {
                Some(issues.join(" "))
            };
            commit::generate_commit_message(r#type, message, all, cli.no_review, no_translate, only_chinese, only_english, no_test_suggestions, issues_str).await
        }
        Some(Commands::AIReview { enable, disable, status }) => {
            let mut config = config::Config::load()?;
            if status {
                let language_mode = if config.only_chinese {
                    "仅中文"
                } else if config.only_english {
                    "仅英文"
                } else {
                    "中英双语"
                };
                println!("{}", Style::title(&format!("AI 代码审查功能当前状态: {}", if config.ai_review { "已启用" } else { "已禁用" })));
                println!("{}", Style::plain(&format!("默认提交信息语言: {}", language_mode)));
                return Ok(());
            }
            if enable {
                config.ai_review = true;
                config.save()?;
                println!("{}", Style::green("已启用 AI 代码审查功能"));
            } else if disable {
                config.ai_review = false;
                config.save()?;
                println!("{}", Style::yellow("已禁用 AI 代码审查功能"));
            }
            Ok(())
        }
        None => {
            match cli.input {
                Some(input) if input.starts_with("http") => {
                    // 处理远程代码审查链接
                    let config = config::Config::load()?;
                    if config.services.is_empty() {
                        return Err(anyhow::anyhow!("没有配置任何 AI 服务，请先添加服务"));
                    }

                    match review::review_remote_changes(&config, &input).await {
                        Ok(review) => {
                            println!("\n{}\n", review);
                            Ok(())
                        }
                        Err(e) => Err(e)
                    }
                }
                Some(input) if input.len() >= 7 && input.len() <= 40 && input.chars().all(|c| c.is_ascii_hexdigit()) => {
                    // 处理Git commit id
                    let config = config::Config::load()?;
                    if config.services.is_empty() {
                        return Err(anyhow::anyhow!("没有配置任何 AI 服务，请先添加服务"));
                    }

                    match review::review_local_commit(&config, &input).await {
                        Ok(review) => {
                            println!("\n{}\n", review);
                            Ok(())
                        }
                        Err(e) => Err(e)
                    }
                }
                Some(path) => {
                    // 处理Git commit message文件
                    let path = PathBuf::from(path);
                    let no_review = cli.no_review;
                    git::process_commit_msg(&path, no_review).await
                }
                None => {
                    Err(anyhow::anyhow!("Missing input: expected commit message file path or GitHub/Gerrit URL or commit id (7-40 chars)"))
                }
            }
        }
    }
}
