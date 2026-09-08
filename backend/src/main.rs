#![cfg_attr(windows, windows_subsystem = "windows")]

mod api;
mod auth;
mod config;
mod database;
mod error;
mod logger;
mod newapi;
mod provider;
mod server;
#[cfg(windows)]
mod windows_tray;

use anyhow::Result;
use config::Config;
use database::Database;
use reqwest::Client;
use server::AppState;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;
    let database = Database::connect(&config.database_url).await?;
    let state = AppState {
        config: config.clone(),
        database,
        http_client: Client::builder()
            .user_agent(concat!("meshway/", env!("CARGO_PKG_VERSION")))
            .build()?,
    };
    let app = server::router(state);
    let listener = tokio::net::TcpListener::bind(config.address()).await?;
    tracing::info!(address = %config.address(), "Meshway listening");

    #[cfg(windows)]
    {
        let url = format!("http://{}", config.address());
        let server = tokio::spawn(async move { axum::serve(listener, app).await });
        windows_tray::run(url)?;
        server.abort();
        let _ = server.await;
    }

    #[cfg(not(windows))]
    axum::serve(listener, app).await?;

    Ok(())
}
