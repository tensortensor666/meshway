use crate::{database::Database, error::ApiError};
use axum::{body::Body, response::Response};
use reqwest::{Client, header};
use serde_json::Value;

pub struct ForwardResult {
    pub response: Response,
    pub provider: String,
}

pub async fn forward(
    client: &Client,
    database: &Database,
    path: &str,
    payload: Value,
) -> Result<ForwardResult, ApiError> {
    let provider_type = if path == "/v1/messages" {
        "anthropic"
    } else {
        "openai"
    };
    let (_, base_url, api_key) = database.get_provider(provider_type).await?;
    let endpoint = endpoint(&base_url, path)?;
    let mut request = client.post(endpoint).json(&payload);
    if provider_type == "anthropic" {
        request = request
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01");
    } else {
        request = request.header(header::AUTHORIZATION, format!("Bearer {api_key}"));
    }
    let upstream = request.send().await?;
    let status = upstream.status();
    let headers = upstream.headers().clone();
    let stream = upstream.bytes_stream();
    let mut response = Response::new(Body::from_stream(stream));
    *response.status_mut() = status;
    for (name, value) in &headers {
        if name == header::CONTENT_TYPE || name == header::CACHE_CONTROL {
            response.headers_mut().insert(name.clone(), value.clone());
        }
    }
    Ok(ForwardResult {
        response,
        provider: provider_type.into(),
    })
}

fn endpoint(base_url: &str, path: &str) -> Result<String, ApiError> {
    let base = base_url.trim_end_matches('/');
    let suffix = match path {
        "/v1/chat/completions" => "/chat/completions",
        "/v1/responses" => "/responses",
        "/v1/messages" => "/messages",
        _ => return Err(ApiError::BadRequest("unsupported API path".into())),
    };
    Ok(format!("{base}{suffix}"))
}
