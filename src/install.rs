use anyhow::{Context, Result};
use dialoguer::{Confirm, Select};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn install_git_hook(repo_path: Option<PathBuf>, force: bool) -> Result<()> {
    let repo_path = repo_path.unwrap_or_else(|| PathBuf::from("."));
    let git_dir = find_git_dir(&repo_path)?;
    let hooks_dir = git_dir.join("hooks");
    let commit_msg_hook = hooks_dir.join("commit-msg");

    // 处理已存在的 hook
    let (use_backup, run_before) = if commit_msg_hook.exists() {
        if force {
            let (keep_old, run_before) = handle_existing_hook(&commit_msg_hook)?;
            (keep_old, if keep_old { Some(run_before) } else { None })
        } else {
            return Err(anyhow::anyhow!(
                "Hook 文件已存在: {}。使用 --force 选项进行处理。",
                commit_msg_hook.display()
            ));
        }
    } else {
        (false, None)
    };

    // 获取当前二进制的路径
    let current_exe = std::env::current_exe()?;
    let binary_path = current_exe.canonicalize()?;

    // 创建新的 hook 内容
    let hook_content = create_hook_content(&binary_path, use_backup, run_before)?;

    // 确保 hooks 目录存在
    fs::create_dir_all(&hooks_dir)?;

    // 写入 hook 脚本
    fs::write(&commit_msg_hook, hook_content)?;

    // 设置可执行权限
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&commit_msg_hook, fs::Permissions::from_mode(0o755))?;
    }

    println!("Git hook 已安装到: {}", commit_msg_hook.display());
    Ok(())
}

fn handle_existing_hook(hook_path: &Path) -> Result<(bool, bool)> {
    println!("检测到已存在的 commit-msg hook");

    let keep_old = Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("是否保留已存在的 hook 功能？")
        .default(true)
        .interact()?;

    if !keep_old {
        fs::remove_file(hook_path)?;
        return Ok((false, false));
    }

    // 备份原有 hook 到同目录下
    let hooks_dir = hook_path.parent().unwrap();
    let backup_path = hooks_dir.join("commit-msg.old");
    fs::rename(hook_path, &backup_path)?;
    println!("已存在的 hook 已备份到: {}", backup_path.display());

    let options = vec![
        "先执行翻译程序，再执行原 hook",
        "先执行原 hook，再执行翻译程序"
    ];
    let selection = Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("请选择执行顺序")
        .items(&options)
        .default(0)
        .interact()?;

    Ok((true, selection == 0))
}

fn create_hook_content(binary_path: &Path, use_backup: bool, run_before: Option<bool>) -> Result<String> {
    if !use_backup {
        return Ok(format!(
            r#"#!/bin/sh
exec "{}" "$1"
"#,
            binary_path.display()
        ));
    }

    let git_hooks_dir = binary_path.parent().unwrap().parent().unwrap().join("hooks");
    let backup_path = git_hooks_dir.join("commit-msg.old");

    match run_before {
        Some(true) => Ok(format!(
            r#"#!/bin/sh
# 先运行当前程序，如果失败则中止
"{}" "$1" || exit $?

# 如果存在旧的 hook，则运行它
if [ -x "{}" ]; then
    exec "{}" "$1"
fi
"#,
            binary_path.display(),
            backup_path.display(),
            backup_path.display()
        )),
        Some(false) => Ok(format!(
            r#"#!/bin/sh
# 如果存在旧的 hook，先运行它
if [ -x "{}" ]; then
    "{}" "$1" || exit $?
fi

# 运行当前程序
exec "{}" "$1"
"#,
            backup_path.display(),
            backup_path.display(),
            binary_path.display()
        )),
        None => unreachable!("使用备份时必须指定运行顺序"),
    }
}

fn find_git_dir(start_path: &Path) -> Result<PathBuf> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--git-dir")
        .current_dir(start_path)
        .output()
        .context("执行 git 命令失败")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("当前目录不是 git 仓库"));
    }

    let git_dir = String::from_utf8(output.stdout)?;
    Ok(PathBuf::from(git_dir.trim()))
}
