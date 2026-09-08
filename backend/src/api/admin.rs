use crate::{
    auth,
    database::{ApiKeyRecord, Database, LogRecord, ProviderRecord},
    error::ApiError,
    newapi::{self, NewApiInput, NewApiInspection},
    server::AppState,
};
use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ProviderInput {
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: String,
    pub base_url: String,
    pub api_key: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct ApiKeyInput {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct CreatedKey {
    pub id: String,
    pub name: String,
    pub key: String,
}

fn default_enabled() -> bool {
    true
}

pub async fn inspect_newapi(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(input): Json<NewApiInput>,
) -> Result<Json<NewApiInspection>, ApiError> {
    require_admin(&headers, &state)?;
    Ok(Json(newapi::inspect(&state.http_client, input).await?))
}

fn require_admin(headers: &HeaderMap, state: &AppState) -> Result<(), ApiError> {
    let token = auth::bearer_token(headers)
        .or_else(|| headers.get("x-admin-token").and_then(|v| v.to_str().ok()));
    if token == Some(state.config.admin_token.as_str()) {
        Ok(())
    } else {
        Err(ApiError::Unauthorized)
    }
}

pub async fn stats(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    require_admin(&headers, &state)?;
    Ok(Json(state.database.dashboard_stats().await?))
}

pub async fn providers(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<ProviderRecord>>, ApiError> {
    require_admin(&headers, &state)?;
    Ok(Json(state.database.list_providers().await?))
}

pub async fn save_provider(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(input): Json<ProviderInput>,
) -> Result<Json<Value>, ApiError> {
    require_admin(&headers, &state)?;
    if !["openai", "anthropic"].contains(&input.provider_type.as_str()) {
        return Err(ApiError::BadRequest(
            "type must be openai or anthropic".into(),
        ));
    }
    if input.name.trim().is_empty()
        || input.base_url.trim().is_empty()
        || input.api_key.trim().is_empty()
    {
        return Err(ApiError::BadRequest(
            "name, base_url and api_key are required".into(),
        ));
    }
    state
        .database
        .upsert_provider(
            &input.name,
            &input.provider_type,
            input.base_url.trim(),
            input.api_key.trim(),
            input.enabled,
        )
        .await?;
    Ok(Json(json!({ "status": "saved" })))
}

pub async fn api_keys(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<ApiKeyRecord>>, ApiError> {
    require_admin(&headers, &state)?;
    Ok(Json(state.database.list_api_keys().await?))
}

pub async fn create_api_key(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(input): Json<ApiKeyInput>,
) -> Result<Json<CreatedKey>, ApiError> {
    require_admin(&headers, &state)?;
    if input.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name is required".into()));
    }
    let key = format!("mw_{}", Uuid::new_v4().simple());
    let record = state
        .database
        .create_api_key(input.name.trim(), &auth::hash_key(&key))
        .await?;
    Ok(Json(CreatedKey {
        id: record.id,
        name: record.name,
        key,
    }))
}

pub async fn set_api_key(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    require_admin(&headers, &state)?;
    let enabled = input
        .get("enabled")
        .and_then(Value::as_bool)
        .ok_or_else(|| ApiError::BadRequest("enabled must be boolean".into()))?;
    state.database.set_api_key_enabled(&id, enabled).await?;
    Ok(Json(json!({ "status": "saved" })))
}

pub async fn logs(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<LogRecord>>, ApiError> {
    require_admin(&headers, &state)?;
    Ok(Json(state.database.list_logs().await?))
}

#[allow(dead_code)]
fn _database_is_used(_: &Database) {}
