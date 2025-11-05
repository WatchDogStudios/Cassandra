mod auth;
mod cli;
mod grpc;
mod http;
mod metrics;
mod state;

#[cfg(test)]
mod tests;

use crate::metrics::MetricsLayer;
use crate::state::AppState;
use clap::Parser;
use cncore::{config, init_tracing, shutdown_signal};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tonic::transport::Server;
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
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

    let cfg = config().clone();

    #[cfg(feature = "db")]
    {
        if let Err(e) = cncore::run_migrations().await {
            tracing::error!(error = %e, "migrations failed");
        } else {
            tracing::info!("database migrations applied");
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
    let swagger = utoipa_swagger_ui::SwaggerUi::new("/docs").url("/api-docs/openapi.json", openapi);

    let state = {
        #[cfg(feature = "db")]
        {
            use cncore::platform::persistence::{
                PostgresAgentStore, PostgresContentStore, PostgresMessagingStore,
                PostgresModerationStore, PostgresOrchestrationStore,
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
            let agent_store = Arc::new(PostgresAgentStore::new(pool));
            let mut state = AppState::with_content_store(content_store);
            state.orchestration_store = orchestration_store;
            state.moderation_store = moderation_store;
            state.messaging_store = messaging_store;
            state.agent_store = Some(agent_store);
            state
        }
        #[cfg(not(feature = "db"))]
        {
            use cncore::platform::persistence::InMemoryPersistence;
            let store: Arc<dyn cncore::platform::persistence::ContentStore> =
                Arc::new(InMemoryPersistence::new());
            AppState::with_content_store(store)
        }
    };

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

    let grpc_service = {
        #[cfg(feature = "db")]
        {
            grpc::InMemoryAgentControl::with_store(
                state.registry.clone(),
                state.agent_store.clone(),
            )
        }
        #[cfg(not(feature = "db"))]
        {
            grpc::InMemoryAgentControl::new(state.registry.clone())
        }
    }
    .into_server();
    let mut grpc_addr = addr;
    grpc_addr.set_port(grpc_addr.port() + 1);
    let grpc = Server::builder().add_service(grpc_service).serve(grpc_addr);
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
