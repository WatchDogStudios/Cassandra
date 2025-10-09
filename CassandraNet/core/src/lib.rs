//! Core foundational utilities: configuration, tracing init, shutdown signals.
use anyhow::Result;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub mod build_info;
pub use build_info::{build_info, BuildInfo};
pub mod platform;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub service_name: String,
    pub log_level: Option<String>,
    pub http: HttpConfig,
    #[cfg(feature = "db")]
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpConfig {
    pub bind_addr: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            service_name: "cassandra-gateway".into(),
            log_level: Some("info".into()),
            http: HttpConfig {
                bind_addr: "127.0.0.1:8080".into(),
            },
            #[cfg(feature = "db")]
            database: DatabaseConfig::default(),
        }
    }
}

#[cfg(feature = "db")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[cfg(feature = "db")]
impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://localhost:5432/cassandra".into(),
            max_connections: 5,
        }
    }
}

static GLOBAL_CONFIG: Lazy<AppConfig> = Lazy::new(|| load_config().unwrap_or_default());

pub fn config() -> &'static AppConfig {
    &GLOBAL_CONFIG
}

fn load_config() -> Result<AppConfig> {
    #[allow(unused_mut)]
    let mut builder = config::Config::builder()
        .set_default("service_name", "cassandra-gateway")?
        .set_default("http.bind_addr", "127.0.0.1:8080")?;
    #[cfg(feature = "db")]
    {
        builder = builder
            .set_default("database.url", "postgres://localhost:5432/cassandra")?
            .set_default("database.max_connections", 5)?;
    }
    let c = builder
        .add_source(config::Environment::with_prefix("CASS").separator("__"))
        .build()?;
    let cfg: AppConfig = c.try_deserialize()?;
    Ok(cfg)
}

pub fn init_tracing() {
    static START: Lazy<()> = Lazy::new(|| {
        let cfg = config();
        let level = cfg.log_level.clone().unwrap_or_else(|| "info".into());
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer())
            .init();
    });
    Lazy::force(&START);
}

pub async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut term = signal(SignalKind::terminate()).expect("install SIGTERM handler");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {},
            _ = term.recv() => {},
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
    tracing::info!("shutdown signal received");
}

// Database pool singleton (sqlx) behind feature flag
#[cfg(feature = "db")]
use once_cell::sync::OnceCell;
#[cfg(feature = "db")]
static DB: OnceCell<sqlx::Pool<sqlx::Postgres>> = OnceCell::new();

#[cfg(feature = "db")]
pub async fn db() -> Result<&'static sqlx::Pool<sqlx::Postgres>> {
    if let Some(p) = DB.get() {
        return Ok(p);
    }
    let cfg = &config().database;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(cfg.max_connections)
        .connect(&cfg.url)
        .await?;
    let _ = DB.set(pool);
    Ok(DB.get().unwrap())
}

#[cfg(feature = "db")]
pub async fn run_migrations() -> Result<()> {
    let pool = db().await?;
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}
