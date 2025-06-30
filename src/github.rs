use anyhow::Result;
use reqwest;
use serde::Deserialize;
use log::debug;
#[derive(Debug, Deserialize)]
struct Commit {
    commit: CommitDetails,
}

#[derive(Debug, Deserialize)]
struct CommitDetails {
    message: String,
}

#[derive(Debug, Deserialize)]
struct PullRequest {
    diff_url: String,
    title: String,
    body: Option<String>,
}

use crate::terminal_format::print_progress;

pub async fn get_pr_info(pr_url: &str) -> Result<String> {
    debug!("从GitHub获取PR信息: {}", pr_url);

    // 解析PR URL
    // 例如: https://github.com/owner/repo/pull/123
    let parts: Vec<&str> = pr_url.split('/').collect();
    if parts.len() < 7 {
        return Err(anyhow::anyhow!("无效的GitHub PR URL"));
    }

    let owner = parts[3];
    let repo = parts[4];
    let pr_number = parts[6];

    // 构建API URL
    let api_url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}",
        owner, repo, pr_number
    );

    // 进度提示
    print_progress("正在请求 github.com 获取PR内容", None);

    // 发送请求
    let client = reqwest::Client::new();
    let pr: PullRequest = client
        .get(&api_url)
        .header("User-Agent", "git-commit-helper")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?
        .json()
        .await?;

    print_progress("正在请求 github.com 获取PR内容", Some(100));

    let mut info = format!("标题：{}\n", pr.title);
    if let Some(body) = pr.body {
        if !body.trim().is_empty() {
            info.push_str(&format!("\n描述：\n{}", body));
        }
    }

    Ok(info)
}

pub async fn get_commit_info(commit_url: &str) -> Result<String> {
    debug!("从GitHub获取commit信息: {}", commit_url);

    // 解析commit URL
    // 例如: https://github.com/owner/repo/commit/hash
    let parts: Vec<&str> = commit_url.split('/').collect();
    if parts.len() < 7 {
        return Err(anyhow::anyhow!("无效的GitHub commit URL"));
    }

    let owner = parts[3];
    let repo = parts[4];
    let commit_hash = parts[6];

    // 构建API URL
    let api_url = format!(
        "https://api.github.com/repos/{}/{}/commits/{}",
        owner, repo, commit_hash
    );

    // 发送请求获取commit信息
    let client = reqwest::Client::new();
    let commit: Commit = client
        .get(&api_url)
        .header("User-Agent", "git-commit-helper")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?
        .json()
        .await?;

    Ok(commit.commit.message)
}

pub async fn get_pr_diff(pr_url: &str) -> Result<String> {
    debug!("从GitHub获取PR差异内容: {}", pr_url);

    // 解析PR URL
    // 例如: https://github.com/owner/repo/pull/123
    let parts: Vec<&str> = pr_url.split('/').collect();
    if parts.len() < 7 {
        return Err(anyhow::anyhow!("无效的GitHub PR URL"));
    }

    let owner = parts[3];
    let repo = parts[4];
    let pr_number = parts[6];

    // 构建API URL
    let api_url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}",
        owner, repo, pr_number
    );

    // 进度提示
    print_progress("正在请求 github.com 获取PR内容", None);

    // 发送请求
    let client = reqwest::Client::new();
    let pr: PullRequest = client
        .get(&api_url)
        .header("User-Agent", "git-commit-helper")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?
        .json()
        .await?;

    print_progress("正在请求 github.com 获取PR内容", Some(60));

    // 获取diff内容
    print_progress("正在请求 github.com 获取PR差异内容", None);
    let diff = client
        .get(&pr.diff_url)
        .header("User-Agent", "git-commit-helper")
        .send()
        .await?
        .text()
        .await?;

    print_progress("正在请求 github.com 获取PR差异内容", Some(100));

    Ok(diff)
}

pub async fn get_commit_diff(commit_url: &str) -> Result<String> {
    debug!("从GitHub获取commit差异内容: {}", commit_url);

    // 解析commit URL
    // 例如: https://github.com/owner/repo/commit/hash
    let parts: Vec<&str> = commit_url.split('/').collect();
    if parts.len() < 7 {
        return Err(anyhow::anyhow!("无效的GitHub commit URL"));
    }

    let owner = parts[3];
    let repo = parts[4];
    let commit_hash = parts[6];

    // 构建API URL
    let api_url = format!(
        "https://api.github.com/repos/{}/{}/commits/{}",
        owner, repo, commit_hash
    );

    // 发送请求获取diff
    let client = reqwest::Client::new();
    let diff = client
        .get(&api_url)
        .header("User-Agent", "git-commit-helper")
        .header("Accept", "application/vnd.github.v3.diff")
        .send()
        .await?
        .text()
        .await?;

    Ok(diff)
}
