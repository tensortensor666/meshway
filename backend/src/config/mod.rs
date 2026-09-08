use anyhow::{Context, Result};
use std::{env, net::SocketAddr};

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub admin_token: String,
    pub web_dir: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let host = env::var("MESHWAY_HOST").unwrap_or_else(|_| "127.0.0.1".into());
        let port = env::var("MESHWAY_PORT")
            .unwrap_or_else(|_| "8080".into())
            .parse()
            .context("MESHWAY_PORT must be a valid port")?;
        let database_url =
            env::var("MESHWAY_DATABASE_URL").unwrap_or_else(|_| "sqlite://data/meshway.db".into());
        let admin_token = env::var("MESHWAY_ADMIN_TOKEN").unwrap_or_else(|_| "change-me".into());
        let web_dir = env::var("MESHWAY_WEB_DIR").unwrap_or_else(|_| "frontend/dist".into());
        Ok(Self {
            host,
            port,
            database_url,
            admin_token,
            web_dir,
        })
    }

    pub fn address(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("validated host and port")
    }
}
