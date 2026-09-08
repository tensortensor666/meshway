use anyhow::{Context, Result};
use std::{env, net::SocketAddr, path::PathBuf};

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
            env::var("MESHWAY_DATABASE_URL").unwrap_or_else(|_| default_database_url());
        let admin_token = env::var("MESHWAY_ADMIN_TOKEN").unwrap_or_else(|_| "change-me".into());
        let web_dir = env::var("MESHWAY_WEB_DIR").unwrap_or_else(|_| default_web_dir());
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

fn default_database_url() -> String {
    #[cfg(target_os = "windows")]
    if let Ok(local_app_data) = env::var("LOCALAPPDATA") {
        let path = PathBuf::from(local_app_data)
            .join("Meshway")
            .join("data")
            .join("meshway.db");
        return format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"));
    }
    "sqlite://data/meshway.db".into()
}

fn default_web_dir() -> String {
    if let Ok(executable) = env::current_exe() {
        if let Some(parent) = executable.parent() {
            let bundled = parent.join("frontend").join("dist");
            if bundled.is_dir() {
                return bundled.to_string_lossy().into_owned();
            }
        }
    }
    #[cfg(target_os = "linux")]
    return "/usr/share/meshway/frontend/dist".into();
    #[cfg(not(target_os = "linux"))]
    "frontend/dist".into()
}
