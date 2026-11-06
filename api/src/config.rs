use std::net::SocketAddr;

use anyhow::Context;

pub struct Config {
    pub database_url: String,
    pub bind_addr: SocketAddr,
    pub db_max_connections: u32,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL").context("DATABASE_URL tanımlı olmalı")?,
            bind_addr: std::env::var("BIND_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8080".into())
                .parse()
                .context("BIND_ADDR geçerli bir adres olmalı (örn. 0.0.0.0:8080)")?,
            db_max_connections: std::env::var("DB_MAX_CONNECTIONS")
                .ok()
                .map(|v| v.parse())
                .transpose()
                .context("DB_MAX_CONNECTIONS sayı olmalı")?
                .unwrap_or(10),
        })
    }
}
