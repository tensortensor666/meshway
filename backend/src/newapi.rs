use crate::error::ApiError;
use reqwest::{Client, StatusCode, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

const MAX_MODELS: usize = 100;

#[derive(Debug, Deserialize)]
pub struct NewApiInput {
    pub source: String,
    #[serde(default)]
    pub key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct NewApiInspection {
    pub provider_type: String,
    pub suggested_name: String,
    pub base_url: String,
    pub key_masked: String,
    pub models: Vec<NewApiModel>,
    pub model_count: usize,
    pub models_status: u16,
    pub capabilities: Vec<NewApiCapability>,
}

#[derive(Debug, Serialize)]
pub struct NewApiModel {
    pub id: String,
    pub owned_by: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct NewApiCapability {
    pub name: String,
    pub path: String,
    pub status: Option<u16>,
    pub supported: bool,
    pub detail: String,
}

pub async fn inspect(client: &Client, input: NewApiInput) -> Result<NewApiInspection, ApiError> {
    let (base_url, api_key) = parse_source(&input.source, input.key.as_deref())?;
    let key_masked = mask_key(&api_key);
    let suggested_name = format!("NewAPI · {}", base_url_host(&base_url));
    let models_url = format!("{base_url}/models");

    let models_response = tokio::time::timeout(
        Duration::from_secs(10),
        client.get(&models_url).bearer_auth(&api_key).send(),
    )
    .await
    .ok()
    .and_then(Result::ok);

    let (models_status, models) = match models_response {
        Some(response) => {
            let status = response.status().as_u16();
            let body = tokio::time::timeout(Duration::from_secs(5), response.json::<Value>())
                .await
                .ok()
                .and_then(Result::ok)
                .unwrap_or(Value::Null);
            (status, parse_models(&body))
        }
        None => (0, Vec::new()),
    };

    let capabilities = vec![
        probe_capability(
            client,
            &base_url,
            &api_key,
            "OpenAI Chat Completions",
            "/chat/completions",
            false,
        )
        .await,
        probe_capability(
            client,
            &base_url,
            &api_key,
            "OpenAI Responses",
            "/responses",
            false,
        )
        .await,
        probe_capability(
            client,
            &base_url,
            &api_key,
            "Anthropic Messages",
            "/messages",
            true,
        )
        .await,
    ];

    Ok(NewApiInspection {
        provider_type: "openai".into(),
        suggested_name,
        base_url,
        key_masked,
        model_count: models.len(),
        models,
        models_status,
        capabilities,
    })
}

async fn probe_capability(
    client: &Client,
    base_url: &str,
    api_key: &str,
    name: &str,
    path: &str,
    anthropic: bool,
) -> NewApiCapability {
    let endpoint = format!("{base_url}{path}");
    let mut request = client.get(&endpoint).bearer_auth(api_key);
    if anthropic {
        request = request
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01");
    }
    match tokio::time::timeout(Duration::from_secs(10), request.send()).await {
        Ok(Ok(response)) => {
            let status = response.status();
            NewApiCapability {
                name: name.into(),
                path: path.into(),
                status: Some(status.as_u16()),
                supported: status != StatusCode::NOT_FOUND && !status.is_server_error(),
                detail: status_detail(status),
            }
        }
        _ => NewApiCapability {
            name: name.into(),
            path: path.into(),
            status: None,
            supported: false,
            detail: "网络请求失败或超时".into(),
        },
    }
}

fn parse_source(source: &str, explicit_key: Option<&str>) -> Result<(String, String), ApiError> {
    let source = source.trim();
    if source.is_empty() {
        return Err(ApiError::BadRequest("NewAPI 配置不能为空".into()));
    }

    let (source_url, source_key) = if let Ok(value) = serde_json::from_str::<Value>(source) {
        (
            value.get("url").and_then(Value::as_str).map(str::to_owned),
            value.get("key").and_then(Value::as_str).map(str::to_owned),
        )
    } else {
        (extract_url(source), None)
    };

    let url = source_url
        .or_else(|| extract_url(source))
        .ok_or_else(|| ApiError::BadRequest("没有找到有效的 NewAPI URL".into()))?;
    let key = explicit_key
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .or(source_key)
        .ok_or_else(|| {
            ApiError::BadRequest(
                "没有找到 NewAPI Key，请在配置 JSON 中提供 key 或单独填写 Key".into(),
            )
        })?;

    let base_url = normalize_base_url(&url)?;
    if key.len() < 8 {
        return Err(ApiError::BadRequest("NewAPI Key 长度过短".into()));
    }
    Ok((base_url, key))
}

fn normalize_base_url(raw: &str) -> Result<String, ApiError> {
    let mut url = Url::parse(raw.trim().trim_matches(|c| "\"'<>[](){}".contains(c)))
        .map_err(|_| ApiError::BadRequest("NewAPI URL 格式无效".into()))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(ApiError::BadRequest(
            "NewAPI URL 必须使用 http 或 https".into(),
        ));
    }
    url.set_query(None);
    url.set_fragment(None);
    let path = url.path().trim_end_matches('/');
    let path = if path.is_empty() {
        "/v1".to_owned()
    } else if path.ends_with("/v1") {
        path.to_owned()
    } else {
        format!("{path}/v1")
    };
    url.set_path(&path);
    Ok(url.to_string().trim_end_matches('/').to_owned())
}

fn extract_url(source: &str) -> Option<String> {
    let start = source.find("https://").or_else(|| source.find("http://"))?;
    let raw = &source[start..];
    let end = raw
        .char_indices()
        .find(|(_, c)| matches!(c, '"' | '\'' | '}' | ']' | ')' | '>' | ' ' | '\n' | '\r'))
        .map(|(index, _)| index)
        .unwrap_or(raw.len());
    Some(raw[..end].trim_end_matches('/').to_owned())
}

fn parse_models(value: &Value) -> Vec<NewApiModel> {
    value
        .get("data")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let id = item.get("id").and_then(Value::as_str)?.to_owned();
                    Some(NewApiModel {
                        id,
                        owned_by: item
                            .get("owned_by")
                            .and_then(Value::as_str)
                            .map(str::to_owned),
                    })
                })
                .take(MAX_MODELS)
                .collect()
        })
        .unwrap_or_default()
}

fn mask_key(key: &str) -> String {
    if key.len() <= 8 {
        return "••••••••".into();
    }
    format!("{}••••{}", &key[..4], &key[key.len() - 4..])
}

fn base_url_host(base_url: &str) -> String {
    Url::parse(base_url)
        .ok()
        .and_then(|url| url.host_str().map(str::to_owned))
        .unwrap_or_else(|| "endpoint".into())
}

fn status_detail(status: StatusCode) -> String {
    match status {
        StatusCode::METHOD_NOT_ALLOWED => "接口存在，但不接受 GET 探测请求".into(),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => "接口存在，但 Key 未通过认证".into(),
        StatusCode::NOT_FOUND => "接口不存在".into(),
        status if status.is_success() => "接口可访问".into(),
        _ => format!("接口返回 HTTP {}", status.as_u16()),
    }
}
