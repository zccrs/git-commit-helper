use log::debug;
use serde_json::Value;

pub fn log_request_info(url: &str, body: &Value) {
    debug!("发送请求到: {}", url);
    debug!("请求体: {}", serde_json::to_string_pretty(body).unwrap_or_default());
}

pub fn log_response_info(response: &str) {
    debug!("收到响应: {}", response);
}
