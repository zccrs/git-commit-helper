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
    let run_before = if commit_msg_hook.exists() {
        if force {
            handle_existing_hook(&commit_msg_hook)?
        } else {
            return Err(anyhow::anyhow!(
                "Hook 文件已存在: {}。使用 --force 选项进行处理。",
                commit_msg_hook.display()
            ));
        }
    } else {
        false
    };

    // 获取当前二进制的路径
    let current_exe = std::env::current_exe()?;
    let binary_path = current_exe.canonicalize()?;

    // 创建新的 hook 内容
    let hook_content = create_hook_content(&binary_path, run_before)?;

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

fn handle_existing_hook(hook_path: &Path) -> Result<bool> {
    println!("检测到已存在的 commit-msg hook");
    
    if !Confirm::new()
        .with_prompt("是否保留现有的 hook 功能？")
        .default(true)
        .interact()? {
        return Ok(false);
    }

    // 备份原有 hook
    let backup_path = hook_path.with_extension("old");
    fs::rename(hook_path, &backup_path)?;
    println!("原有 hook 已备份到: {}", backup_path.display());

    let options = vec!["在现有 hook 之前运行", "在现有 hook 之后运行"];
    let selection = Select::new()
        .with_prompt("请选择运行顺序")
        .items(&options)
        .default(0)
        .interact()?;

    Ok(selection == 0)  // 返回是否在现有 hook 之前运行
}

fn create_hook_content(binary_path: &Path, run_before: bool) -> Result<String> {
    let backup_path = binary_path.parent().unwrap().join("commit-msg.old");
    
    if backup_path.exists() {
        if run_before {
            Ok(format!(
                r#"#!/bin/sh
if [ -x "{}" ]; then
    "{}" "$1" || exit $?
fi
exec {} "$1"
"#,
                backup_path.display(),
                backup_path.display(),
                binary_path.display()
            ))
        } else {
            Ok(format!(
                r#"#!/bin/sh
exec {} "$1" || exit $?
if [ -x "{}" ]; then
    exec "{}" "$1"
fi
"#,
                binary_path.display(),
                backup_path.display(),
                backup_path.display()
            ))
        }
    } else {
        // 如果没有备份的 hook，则只包含当前程序
        Ok(format!(
            r#"#!/bin/sh
exec {} "$1"
"#,
            binary_path.display()
        ))
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
