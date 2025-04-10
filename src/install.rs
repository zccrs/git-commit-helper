use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn install_git_hook(repo_path: Option<PathBuf>, force: bool) -> Result<()> {
    let repo_path = repo_path.unwrap_or_else(|| PathBuf::from("."));
    let git_dir = find_git_dir(&repo_path)?;
    let hooks_dir = git_dir.join("hooks");
    let commit_msg_hook = hooks_dir.join("commit-msg");

    // 检查 hook 文件是否已存在
    if commit_msg_hook.exists() && !force {
        return Err(anyhow::anyhow!(
            "Hook 文件已存在: {}。使用 --force 选项强制安装。",
            commit_msg_hook.display()
        ));
    }

    // 获取当前二进制的路径
    let current_exe = std::env::current_exe()?;
    let binary_path = current_exe.canonicalize()?;

    // 创建 commit-msg hook 脚本
    let hook_content = format!(
        r#"#!/bin/sh
exec {} "$1"
"#,
        binary_path.display()
    );

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

    println!("Git hook 已{}安装到: {}", 
        if force { "强制" } else { "" },
        commit_msg_hook.display());
    Ok(())
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
