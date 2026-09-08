use crate::error::ApiError;
use chrono::{DateTime, Utc};
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::{path::Path, str::FromStr};
use uuid::Uuid;

#[derive(Clone)]
pub struct Database {
    pub pool: SqlitePool,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct ProviderRecord {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: String,
    pub base_url: String,
    pub enabled: i64,
    pub created_at: String,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct ApiKeyRecord {
    pub id: String,
    pub name: String,
    pub enabled: i64,
    pub created_at: String,
    pub last_used: Option<String>,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct LogRecord {
    pub id: String,
    pub api_key_id: Option<String>,
    pub provider: String,
    pub model: Option<String>,
    pub path: String,
    pub status: i64,
    pub latency_ms: i64,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub created_at: String,
}

impl Database {
    pub async fn connect(url: &str) -> Result<Self, ApiError> {
        if let Some(path) = url.strip_prefix("sqlite://") {
            if path != ":memory:" {
                if let Some(parent) = Path::new(path).parent() {
                    tokio::fs::create_dir_all(parent)
                        .await
                        .map_err(|e| anyhow::anyhow!(e))?;
                }
            }
        }
        let options = SqliteConnectOptions::from_str(url)?.create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;
        sqlx::raw_sql(include_str!("../../../migrations/001_initial.sql"))
            .execute(&pool)
            .await?;
        Ok(Self { pool })
    }

    pub async fn list_providers(&self) -> Result<Vec<ProviderRecord>, ApiError> {
        Ok(sqlx::query_as::<_, ProviderRecord>("SELECT id, name, type AS provider_type, base_url, enabled, created_at FROM providers ORDER BY created_at DESC")
            .fetch_all(&self.pool).await?)
    }

    pub async fn get_provider(
        &self,
        provider_type: &str,
    ) -> Result<(String, String, String), ApiError> {
        sqlx::query_as::<_, (String, String, String)>("SELECT type, base_url, api_key FROM providers WHERE type = ? AND enabled = 1 ORDER BY created_at ASC LIMIT 1")
            .bind(provider_type)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| ApiError::NotFound(format!("enabled provider: {provider_type}")))
    }

    pub async fn upsert_provider(
        &self,
        name: &str,
        provider_type: &str,
        base_url: &str,
        api_key: &str,
        enabled: bool,
    ) -> Result<(), ApiError> {
        let now = Utc::now().to_rfc3339();
        sqlx::query("INSERT INTO providers (id, name, type, base_url, api_key, enabled, created_at) VALUES (?, ?, ?, ?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET name=excluded.name, type=excluded.type, base_url=excluded.base_url, api_key=excluded.api_key, enabled=excluded.enabled")
            .bind(Uuid::new_v4().to_string()).bind(name).bind(provider_type).bind(base_url).bind(api_key).bind(enabled as i64).bind(now)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn authenticate_key(&self, hash: &str) -> Result<Option<String>, ApiError> {
        let row = sqlx::query_as::<_, (String,)>(
            "SELECT id FROM api_keys WHERE key_hash = ? AND enabled = 1",
        )
        .bind(hash)
        .fetch_optional(&self.pool)
        .await?;
        if let Some((id,)) = row {
            sqlx::query("UPDATE api_keys SET last_used = ? WHERE id = ?")
                .bind(Utc::now().to_rfc3339())
                .bind(&id)
                .execute(&self.pool)
                .await?;
            Ok(Some(id))
        } else {
            Ok(None)
        }
    }

    pub async fn create_api_key(&self, name: &str, hash: &str) -> Result<ApiKeyRecord, ApiError> {
        let record = ApiKeyRecord {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            enabled: 1,
            created_at: Utc::now().to_rfc3339(),
            last_used: None,
        };
        sqlx::query(
            "INSERT INTO api_keys (id, name, key_hash, enabled, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&record.id)
        .bind(&record.name)
        .bind(hash)
        .bind(record.enabled)
        .bind(&record.created_at)
        .execute(&self.pool)
        .await?;
        Ok(record)
    }

    pub async fn list_api_keys(&self) -> Result<Vec<ApiKeyRecord>, ApiError> {
        Ok(sqlx::query_as::<_, ApiKeyRecord>("SELECT id, name, enabled, created_at, last_used FROM api_keys ORDER BY created_at DESC").fetch_all(&self.pool).await?)
    }

    pub async fn set_api_key_enabled(&self, id: &str, enabled: bool) -> Result<(), ApiError> {
        sqlx::query("UPDATE api_keys SET enabled = ? WHERE id = ?")
            .bind(enabled as i64)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn add_log(
        &self,
        api_key_id: Option<&str>,
        provider: &str,
        model: Option<&str>,
        path: &str,
        status: u16,
        latency_ms: i64,
    ) -> Result<(), ApiError> {
        sqlx::query("INSERT INTO request_logs (id, api_key_id, provider, model, path, status, latency_ms, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(Uuid::new_v4().to_string()).bind(api_key_id).bind(provider).bind(model).bind(path).bind(i64::from(status)).bind(latency_ms).bind(Utc::now().to_rfc3339()).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn list_logs(&self) -> Result<Vec<LogRecord>, ApiError> {
        Ok(sqlx::query_as::<_, LogRecord>("SELECT id, api_key_id, provider, model, path, status, latency_ms, input_tokens, output_tokens, created_at FROM request_logs ORDER BY created_at DESC LIMIT 200").fetch_all(&self.pool).await?)
    }

    pub async fn dashboard_stats(&self) -> Result<serde_json::Value, ApiError> {
        let (total,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM request_logs")
            .fetch_one(&self.pool)
            .await?;
        let (success,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM request_logs WHERE status BETWEEN 200 AND 299")
                .fetch_one(&self.pool)
                .await?;
        let (latency,): (f64,) =
            sqlx::query_as("SELECT COALESCE(AVG(latency_ms), 0) FROM request_logs")
                .fetch_one(&self.pool)
                .await?;
        Ok(
            serde_json::json!({ "total_requests": total, "successful_requests": success, "success_rate": if total == 0 { 0.0 } else { success as f64 / total as f64 }, "average_latency_ms": latency }),
        )
    }
}

#[allow(dead_code)]
fn _date_type_is_available(_: DateTime<Utc>) {}
