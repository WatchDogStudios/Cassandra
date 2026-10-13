mod auth;
mod auth_handlers;
mod cli;
mod grpc;
mod http;
mod messaging_handlers;
mod metrics;
mod portal_handlers;
mod redis_store;
mod state;
mod ugc_handlers;

#[cfg(test)]
mod tests;

use crate::metrics::MetricsLayer;
use crate::state::AppState;
use clap::Parser;
use cncore::{config, shutdown_signal};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tonic::transport::Server;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use utoipa::OpenApi;

fn init_telemetry() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let fmt_layer = tracing_subscriber::fmt::layer();

    let registry = tracing_subscriber::registry().with(filter).with(fmt_layer);

    if std::env::var("COLLECTOR_OTLP_ENABLED").is_ok() {
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(opentelemetry_otlp::new_exporter().tonic())
            .with_trace_config(opentelemetry_sdk::trace::config().with_resource(
                opentelemetry_sdk::Resource::new(vec![opentelemetry::KeyValue::new(
                    "service.name",
                    "cassandra-gateway",
                )]),
            ))
            .install_batch(opentelemetry_sdk::runtime::Tokio)
            .expect("failed to install otlp tracer");

        let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
        registry.with(otel_layer).init();
    } else {
        registry.init();
    }
}

/// Scopes the seeded admin@localhost needs to use every console page.
/// `docs:read` stays opt-in (see migration 0012); `agent:execute` is for agents.
const ADMIN_SCOPES: &[&str] = &[
    "admin:read",
    "admin:write",
    "agent:read",
    "agent:write",
    "apikey:manage",
    "build:read",
    "build:write",
    "build:deploy",
    "script:read",
    "script:write",
    "script:execute",
    "ugc:read",
    "ugc:write",
    "messaging:read",
    "messaging:write",
    "observability:read",
];

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_telemetry();
    let cli::CliArgs {
        print_config,
        command,
    } = cli::CliArgs::parse();

    if print_config {
        println!("{}", serde_json::to_string_pretty(config())?);
        return Ok(());
    }

    if let Some(cmd) = command {
        match cmd {
            cli::CliCommand::GenToken { sub } => {
                println!("{}", auth::hs256_generate(&sub)?);
                return Ok(());
            }
            cli::CliCommand::Version { json } => {
                let info = cncore::build_info();
                let payload = http::VersionResponse {
                    service: config().service_name.clone(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    git_sha: info.git_sha.to_string(),
                    git_tag: info.git_tag.to_string(),
                    build_ts: info.build_timestamp.to_string(),
                };
                if json {
                    println!("{}", serde_json::to_string_pretty(&payload)?);
                } else {
                    println!(
                        "{} v{} (git: {}, tag: {}, built: {})",
                        payload.service,
                        payload.version,
                        payload.git_sha,
                        payload.git_tag,
                        payload.build_ts
                    );
                }
                return Ok(());
            }
        }
    }

    // Fail closed: without a secret no token could be verified, and silently
    // running would leave every bearer-authenticated route unusable.
    let Some(jwt_secret) = auth::jwt_secret() else {
        anyhow::bail!("CASS_JWT_SECRET must be set (use at least 32 random bytes)");
    };
    if jwt_secret.len() < 32 {
        tracing::warn!(
            target: "security",
            len = jwt_secret.len(),
            "CASS_JWT_SECRET is shorter than 32 bytes; use a longer random value in production"
        );
    }

    let cfg = config().clone();

    #[cfg(feature = "db")]
    {
        if let Err(e) = cncore::run_migrations().await {
            tracing::error!(error = %e, "migrations failed");
        } else {
            tracing::info!("database migrations applied");
        }

        // Seed/update admin user password from environment
        if let Ok(admin_password) = std::env::var("CASS_ADMIN_PASSWORD") {
            if !admin_password.is_empty() {
                match seed_admin_user(&admin_password).await {
                    Ok(_) => tracing::info!("admin user seeded/updated"),
                    Err(e) => tracing::warn!(error = %e, "failed to seed admin user"),
                }
            }
        }
    }

    let mut openapi = http::ApiDoc::openapi();
    {
        use utoipa::openapi::security::{
            ApiKey, ApiKeyValue, Http, HttpAuthScheme, SecurityScheme,
        };
        let mut comps = openapi.components.unwrap_or_default();
        comps.add_security_scheme(
            "ApiKey",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("x-api-key"))),
        );
        comps.add_security_scheme(
            "BearerAuth",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );
        openapi.components = Some(comps);
    }
    // Expose the OpenAPI spec at a stable path that works well behind the `/api` reverse proxy.
    // Keep the legacy `/api-docs/openapi.json` path for compatibility during migration.
    let swagger = utoipa_swagger_ui::SwaggerUi::new("/docs")
        .url("/api-doc/openapi.json", openapi.clone())
        .url("/api-docs/openapi.json", openapi);

    let state = {
        let redis_store = if let Ok(url) = std::env::var("CASS_REDIS_URL") {
            match crate::redis_store::RedisAgentStore::new(&url) {
                Ok(store) => {
                    tracing::info!("connected to redis/dragonfly");
                    Some(store)
                }
                Err(e) => {
                    tracing::warn!(error = %e, "failed to connect to redis; falling back to in-memory registry");
                    None
                }
            }
        } else {
            None
        };

        #[cfg(feature = "db")]
        {
            use cncore::platform::persistence::{
                AsyncProjectStore, AsyncTaskStore, AsyncTenantStore, AsyncWorkflowStore,
                PostgresAgentStore, PostgresApiKeyStore, PostgresContentStore,
                PostgresMessagingStore, PostgresModerationStore, PostgresOrchestrationStore,
                PostgresProjectStore, PostgresTaskStore, PostgresTenantStore, PostgresUserStore,
                PostgresWorkflowStore,
            };
            let pool = cncore::db().await?.clone();
            let content_store: Arc<dyn cncore::platform::persistence::ContentStore> =
                Arc::new(PostgresContentStore::new(pool.clone()));
            let orchestration_store: Arc<dyn cncore::platform::persistence::OrchestrationStore> =
                Arc::new(PostgresOrchestrationStore::new(pool.clone()));
            let moderation_store: Arc<dyn cncore::platform::persistence::ModerationStore> =
                Arc::new(PostgresModerationStore::new(pool.clone()));
            let messaging_store: Arc<dyn cncore::platform::persistence::MessagingStore> =
                Arc::new(PostgresMessagingStore::new(pool.clone()));
            let user_store: Arc<dyn cncore::platform::persistence::UserStore> =
                Arc::new(PostgresUserStore::new(pool.clone()));
            let tenant_store: Arc<dyn AsyncTenantStore> =
                Arc::new(PostgresTenantStore::new(pool.clone()));
            let project_store: Arc<dyn AsyncProjectStore> =
                Arc::new(PostgresProjectStore::new(pool.clone()));
            let task_store: Arc<dyn AsyncTaskStore> =
                Arc::new(PostgresTaskStore::new(pool.clone()));
            let workflow_store: Arc<dyn AsyncWorkflowStore> =
                Arc::new(PostgresWorkflowStore::new(pool.clone()));
            let api_key_store: Arc<dyn cncore::platform::persistence::AsyncApiKeyStore> =
                Arc::new(PostgresApiKeyStore::new(pool.clone()));
            let base = AppState::default();
            let audit = state::audit::AuditSink::new(Arc::new(
                state::audit::PostgresAuditEmitter::new(pool.clone())
                    .with_fallback(Arc::new(base.audit.clone())),
            ));
            let portal_store: Arc<dyn cncore::platform::PortalStore> =
                Arc::new(cncore::platform::PostgresPortalStore::new(pool.clone()));
            let agent_store = Arc::new(PostgresAgentStore::new(pool));
            AppState {
                audit,
                portal_store,
                redis_store,
                content_store,
                orchestration_store,
                moderation_store,
                messaging_store,
                user_store,
                tenant_store: Some(tenant_store),
                project_store: Some(project_store),
                task_store: Some(task_store),
                workflow_store: Some(workflow_store),
                agent_store: Some(agent_store),
                api_key_store: Some(api_key_store),
                ..base
            }
        }
        #[cfg(not(feature = "db"))]
        {
            use cncore::platform::persistence::InMemoryPersistence;
            // One InMemoryPersistence instance backs every store.
            let persistence = Arc::new(InMemoryPersistence::new());
            AppState {
                redis_store,
                content_store: persistence.clone(),
                orchestration_store: persistence.clone(),
                moderation_store: persistence.clone(),
                messaging_store: persistence.clone(),
                user_store: persistence,
                ..AppState::default()
            }
        }
    };

    #[cfg(feature = "db")]
    {
        if let Some(store) = state.api_key_store.as_ref() {
            match store.list_all_active_api_keys().await {
                Ok(keys) => {
                    let records = keys
                        .into_iter()
                        .map(|k| {
                            (
                                k.id,
                                k.token_prefix,
                                k.token_hash,
                                k.scopes.iter().map(|s| s.as_str().to_string()).collect(),
                            )
                        })
                        .collect();
                    auth::replace_db_api_key_cache(records);
                    tracing::info!("db api key cache primed");
                }
                Err(err) => {
                    tracing::warn!(error = %err, "failed to prime db api key cache");
                }
            }
        }
    }

    // Seed admin user for in-memory mode
    #[cfg(not(feature = "db"))]
    {
        // No well-known default: generate a one-off password for this
        // (ephemeral, in-memory) instance and print it once.
        let admin_password = match std::env::var("CASS_ADMIN_PASSWORD") {
            Ok(p) if !p.is_empty() => p,
            _ => {
                let generated = uuid::Uuid::new_v4().simple().to_string();
                tracing::warn!(
                    target: "security",
                    "CASS_ADMIN_PASSWORD not set; generated admin password for admin@localhost: {generated}"
                );
                generated
            }
        };
        if let Err(e) = seed_admin_user_inmem(&state, &admin_password).await {
            tracing::warn!(error = %e, "failed to seed in-memory admin user");
        } else {
            tracing::info!("in-memory admin user seeded");
        }
    }

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
        ])
        .allow_headers(Any);

    let app = http::router()
        .with_state(state.clone())
        .layer(MetricsLayer)
        .layer(cors)
        .merge(swagger);
    let make_service = app.into_make_service_with_connect_info::<SocketAddr>();

    let addr: SocketAddr = cfg.http.bind_addr.parse()?;
    let listener = TcpListener::bind(addr).await?;
    tracing::info!(%addr, "gateway listening (http + grpc on same port via hyper)");

    std::fs::create_dir_all(&*state.build_dir).map_err(|err| {
        anyhow::anyhow!(
            "cannot create build storage dir {}: {err}",
            state.build_dir.display()
        )
    })?;

    let grpc_service = {
        #[cfg(feature = "db")]
        {
            grpc::InMemoryAgentControl::with_store(
                state.registry.clone(),
                state.redis_store.clone(),
                state.audit.clone(),
                state.portal_store.clone(),
                state.build_dir.clone(),
                state.agent_store.clone(),
            )
        }
        #[cfg(not(feature = "db"))]
        {
            grpc::InMemoryAgentControl::new(
                state.registry.clone(),
                state.redis_store.clone(),
                state.audit.clone(),
                state.portal_store.clone(),
                state.build_dir.clone(),
            )
        }
    }
    .into_server();

    let ugc_service = grpc::UgcServiceImpl::new(state.moderation_store.clone()).into_server();
    let messaging_service =
        grpc::MessagingServiceImpl::new(state.messaging_store.clone()).into_server();

    let mut grpc_addr = addr;
    grpc_addr.set_port(grpc_addr.port() + 1);
    let grpc = Server::builder()
        .add_service(grpc_service)
        .add_service(ugc_service)
        .add_service(messaging_service)
        .serve(grpc_addr);
    tracing::info!(%grpc_addr, "grpc listening");
    tokio::spawn(async move {
        if let Err(e) = grpc.await {
            tracing::error!(error = %e, "grpc server error");
        }
    });

    axum::serve(listener, make_service)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

#[cfg(not(feature = "db"))]
async fn seed_admin_user_inmem(state: &AppState, password: &str) -> anyhow::Result<()> {
    use chrono::Utc;
    use cncommon::auth::Scope;
    use cncore::platform::User;
    use uuid::Uuid;

    let admin_id: Uuid = "00000000-0000-0000-0000-000000000001".parse()?;
    let password_hash = auth_handlers::hash_password(password)
        .map_err(|_| anyhow::anyhow!("failed to hash password"))?;

    let admin_user = User {
        id: admin_id,
        email: "admin@localhost".to_string(),
        password_hash,
        display_name: Some("Administrator".to_string()),
        tenant_id: None,
        scopes: ADMIN_SCOPES.iter().map(|s| Scope::from_name(s)).collect(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_login_at: None,
        is_active: true,
    };

    state.user_store.create_user(admin_user).await?;
    Ok(())
}

#[cfg(feature = "db")]
async fn seed_admin_user(password: &str) -> anyhow::Result<()> {
    use chrono::Utc;
    use uuid::Uuid;

    let pool = cncore::db().await?.clone();
    let admin_id: Uuid = "00000000-0000-0000-0000-000000000001".parse()?;
    let password_hash = auth_handlers::hash_password(password)
        .map_err(|_| anyhow::anyhow!("failed to hash password"))?;

    // Upsert the admin user with the hashed password
    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, display_name, scopes, is_active, created_at, updated_at)
        VALUES ($1, 'admin@localhost', $2, 'Administrator', $4, TRUE, $3, $3)
        ON CONFLICT (id) DO UPDATE SET
            password_hash = EXCLUDED.password_hash,
            scopes = ARRAY(SELECT DISTINCT unnest(users.scopes || EXCLUDED.scopes)),
            updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(admin_id)
    .bind(&password_hash)
    .bind(Utc::now())
    .bind(ADMIN_SCOPES)
    .execute(&pool)
    .await?;

    Ok(())
}
