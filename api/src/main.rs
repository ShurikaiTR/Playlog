mod auth;
mod catalog;
mod config;
mod error;
mod mail;
mod oauth;
mod ratelimit;
mod routes;
mod state;
mod turnstile;

use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

use crate::{
    auth::{password, role},
    config::Config,
    mail::Mailer,
    oauth::OAuth,
    state::AppState,
    turnstile::Turnstile,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Geliştirmede .env dosyasını yükler; üretimde değişkenler ortamdan gelir ve dosya yoktur.
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = Config::from_env()?;

    let pool = PgPoolOptions::new()
        .max_connections(config.db_max_connections)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&config.database_url)
        .await?;

    // Migration dosyaları derleme anında binary'ye gömülür; açılışta uygulanır.
    sqlx::migrate!("./migrations").run(&pool).await?;

    if let Some(email) = &config.admin_email {
        role::bootstrap_admin(&pool, email).await?;
    }

    let app = routes::router()
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(15),
        ))
        .with_state(AppState {
            db: pool,
            mailer: Mailer::new(config.resend_api_key, config.mail_from)?,
            code_secret: std::sync::Arc::from(config.code_secret.as_slice()),
            dummy_hash: password::dummy_hash().await?.into(),
            limiter: Default::default(),
            turnstile: config
                .turnstile_secret
                .map(|secret| Turnstile::new(secret, config.turnstile_hostnames))
                .transpose()?
                .map(std::sync::Arc::new),
            oauth: OAuth::new(
                config.public_url,
                config.google,
                config.discord,
                config.apple,
                config.steam_api_key,
                &config.code_secret,
            )?
            .into(),
        });

    let listener = TcpListener::bind(config.bind_addr).await?;
    tracing::info!("listening on {}", listener.local_addr()?);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Ctrl+C dinleyicisi kurulamadı");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("SIGTERM dinleyicisi kurulamadı")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
