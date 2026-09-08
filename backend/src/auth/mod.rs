use crate::{database::Database, error::ApiError};
use axum::http::HeaderMap;
use sha2::{Digest, Sha256};

pub fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}

pub fn hash_key(key: &str) -> String {
    hex::encode(Sha256::digest(key.as_bytes()))
}

pub async fn authenticate(headers: &HeaderMap, database: &Database) -> Result<String, ApiError> {
    let token = bearer_token(headers).ok_or(ApiError::Unauthorized)?;
    database
        .authenticate_key(&hash_key(token))
        .await?
        .ok_or(ApiError::Unauthorized)
}
