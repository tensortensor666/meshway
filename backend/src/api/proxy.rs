use crate::{auth, error::ApiError, logger, provider, server::AppState};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, Uri},
    response::Response,
};
use serde_json::Value;
use std::sync::Arc;

pub async fn chat_completions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Result<Response, ApiError> {
    forward(State(state), headers, "/v1/chat/completions", payload).await
}

pub async fn responses(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Result<Response, ApiError> {
    forward(State(state), headers, "/v1/responses", payload).await
}

pub async fn messages(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Result<Response, ApiError> {
    forward(State(state), headers, "/v1/messages", payload).await
}

async fn forward(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    path: &str,
    payload: Value,
) -> Result<Response, ApiError> {
    let api_key_id = auth::authenticate(&headers, &state.database).await?;
    let model = payload
        .get("model")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let started = std::time::Instant::now();
    let result = provider::forward(&state.http_client, &state.database, path, payload).await?;
    let status = result.response.status().as_u16();
    logger::record(
        &state.database,
        &api_key_id,
        &result.provider,
        model.as_deref(),
        path,
        status,
        started,
    )
    .await;
    Ok(result.response)
}

#[allow(dead_code)]
fn _uri_is_used(_: Uri) {}
