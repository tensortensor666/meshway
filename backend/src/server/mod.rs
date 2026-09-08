use crate::{
    api::{admin, health, proxy},
    config::Config,
    database::Database,
};
use axum::{
    Router,
    routing::{get, post, put},
};
use reqwest::Client;
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub database: Database,
    pub http_client: Client,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .route("/v1/chat/completions", post(proxy::chat_completions))
        .route("/v1/responses", post(proxy::responses))
        .route("/v1/messages", post(proxy::messages))
        .route("/api/admin/stats", get(admin::stats))
        .route(
            "/api/admin/providers",
            get(admin::providers).post(admin::save_provider),
        )
        .route(
            "/api/admin/api-keys",
            get(admin::api_keys).post(admin::create_api_key),
        )
        .route("/api/admin/api-keys/{id}", put(admin::set_api_key))
        .route("/api/admin/logs", get(admin::logs))
        .with_state(Arc::new(state))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}
