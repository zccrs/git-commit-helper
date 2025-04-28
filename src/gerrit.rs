use anyhow::Result;
use reqwest::Client;
use log::debug;
use base64::Engine;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ChangeInfo {
    subject: String,
}

pub async fn get_change_info(url: &str) -> Result<String> {
    debug!("开始获取 Gerrit 改动信息: {}", url);

    // 解析 Gerrit URL
    // 示例: https://gerrit.uniontech.com/c/udcp/udcp-uim/+/179042
    let parts: Vec<&str> = url.split("/+/").collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("无效的 Gerrit URL"));
    }

    // 从 URL 中提取项目名和改动 ID
    let parts: Vec<&str> = url.split("/c/").collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("无效的 Gerrit URL"));
    }

    let base_url = parts[0];
    let mut path_parts = parts[1].split("/+/");

    let _project = path_parts.next()
        .ok_or_else(|| anyhow::anyhow!("无法解析项目路径"))?
        .trim_end_matches('/');

    let change_id = path_parts.next()
        .ok_or_else(|| anyhow::anyhow!("无法解析改动ID"))?;

    // 构建 API URL
    let api_url = format!(
        "{}/a/changes/{}",
        base_url,
        change_id
    );

    // 创建 HTTP 客户端并发送请求
    let mut request = Client::new()
        .get(&api_url)
        .header("Accept", "application/json");

    // 添加认证信息
    request = add_auth(request);

    let response = request.send().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "获取 Gerrit 改动信息失败: HTTP {}",
            response.status()
        ));
    }

    let json_text = response.text().await?;
    // Gerrit API 返回的 JSON 数据前面会有一个防止 XSS 的 )]}'
    let json = json_text.trim_start_matches(")]}'\n");

    let info: ChangeInfo = serde_json::from_str(json)?;
    Ok(info.subject)
}

pub async fn get_change_diff(url: &str) -> Result<String> {
    debug!("开始获取 Gerrit 改动内容: {}", url);

    // 解析 Gerrit URL
    // 示例: https://gerrit.uniontech.com/c/udcp/udcp-uim/+/179042
    let parts: Vec<&str> = url.split("/+/").collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("无效的 Gerrit URL"));
    }

    // 从 URL 中提取项目名和改动 ID
    let parts: Vec<&str> = url.split("/c/").collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("无效的 Gerrit URL"));
    }

    let base_url = parts[0];
    let mut path_parts = parts[1].split("/+/");

    let project = path_parts.next()
        .ok_or_else(|| anyhow::anyhow!("无法解析项目路径"))?
        .trim_end_matches('/');

    let change_id = path_parts.next()
        .ok_or_else(|| anyhow::anyhow!("无法解析改动ID"))?;

    // 构造 API URL
    // Gerrit API 文档: https://gerrit-review.googlesource.com/Documentation/rest-api.html
    let api_url = format!(
        "{}/a/changes/{}~{}/revisions/current/patch",  // 添加 /a/ 表示需要认证的 API
        base_url,
        project.replace("/", "%2F"),
        change_id
    );

    debug!("Gerrit API URL: {}", api_url);

    // 创建 HTTP 客户端并准备请求
    let mut request = Client::new()
        .get(&api_url)
        .header("Accept", "text/plain");

    request = add_auth(request);

    let response = request.send().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "获取 Gerrit 改动失败: HTTP {}",
            response.status()
        ));
    }

    let encoded_diff = response.text().await?;
    if encoded_diff.trim().is_empty() {
        return Err(anyhow::anyhow!("未发现任何代码改动"));
    }

    // base64 解码 patch 内容
    let diff = match base64::engine::general_purpose::STANDARD.decode(encoded_diff.trim()) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        Err(e) => return Err(anyhow::anyhow!("解析 patch 内容失败: {}", e)),
    };

    debug!("获取到的 Gerrit patch 内容:\n{}", diff);

    Ok(diff)
}

// 添加认证信息到请求
fn add_auth(mut request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    // 优先使用配置文件中的认证信息
    let mut auth_added = false;
    if let Ok(config) = crate::config::Config::load() {
        if let Some(gerrit_config) = config.gerrit {
            if let Some(token) = gerrit_config.token {
                debug!("使用配置文件中的 Token 认证");
                request = request.bearer_auth(&token);
                auth_added = true;
            } else if let Some(username) = &gerrit_config.username {
                if let Some(password) = &gerrit_config.password {
                    debug!("使用配置文件中的用户名密码认证: {}", username);
                    request = request.basic_auth(username, Some(password));
                    auth_added = true;
                }
            }
        }
    }

    // 如果配置文件中没有认证信息，尝试使用环境变量
    if !auth_added {
        if let Ok(username) = std::env::var("GERRIT_USERNAME") {
            if let Ok(password) = std::env::var("GERRIT_PASSWORD") {
                debug!("使用环境变量中的用户名认证: {}", username);
                request = request.basic_auth(&username, Some(&password));
                auth_added = true;
            }
        } else if let Ok(token) = std::env::var("GERRIT_TOKEN") {
            debug!("使用环境变量中的 Token 认证");
            request = request.bearer_auth(&token);
            auth_added = true;
        }
    }

    if !auth_added {
        debug!("未找到任何认证信息，尝试匿名访问");
    }

    request
}
