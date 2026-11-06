use std::net::SocketAddr;

use anyhow::{Context, bail};

pub struct Config {
    pub database_url: String,
    pub bind_addr: SocketAddr,
    pub db_max_connections: u32,
    /// E-posta kodlarını HMAC ile özetlemek için sunucu sırrı (en az 32 bayt).
    pub code_secret: Vec<u8>,
    /// Verilmezse e-postalar gönderilmez, kodlar günlüğe yazılır (yalnızca geliştirme).
    pub resend_api_key: Option<String>,
    pub mail_from: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let code_secret = std::env::var("CODE_SECRET")
            .context("CODE_SECRET tanımlı olmalı")?
            .into_bytes();
        if code_secret.len() < 32 {
            bail!("CODE_SECRET en az 32 karakter olmalı");
        }

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
            code_secret,
            resend_api_key: std::env::var("RESEND_API_KEY")
                .ok()
                .filter(|v| !v.is_empty()),
            mail_from: std::env::var("MAIL_FROM")
                .unwrap_or_else(|_| "Playthru <onboarding@resend.dev>".into()),
        })
    }
}
