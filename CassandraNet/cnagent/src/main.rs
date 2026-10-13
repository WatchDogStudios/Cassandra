//! CassandraNet Agent - Hardened production-ready agent binary
//!
//! Connects to a CassandraNet gateway, registers itself, and sends periodic heartbeats
//! with system metrics. Supports graceful shutdown, exponential backoff retry, and
//! credential rotation.

mod jobs;

use anyhow::{Context, Result};
use backoff::{future::retry, ExponentialBackoff};
use clap::Parser;
use cnproto::{
    agent_control_client::AgentControlClient, HeartbeatRequest, HeartbeatResponse,
    RegisterAgentRequest,
};
use jobs::{AfterReport, JobRunner, LogBuffer, RESTART_EXIT_CODE};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use sysinfo::System;
use tonic::transport::Channel;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// CassandraNet Agent - connects to gateway and reports system metrics
#[derive(Parser, Debug, Clone)]
#[command(name = "cnagent", version, about, long_about = None)]
pub struct Args {
    /// Gateway gRPC endpoint (e.g., http://localhost:8081)
    #[arg(
        short,
        long,
        env = "CASS_GATEWAY_ADDR",
        default_value = "http://127.0.0.1:8081"
    )]
    pub gateway: String,

    /// Tenant ID for this agent
    #[arg(short, long, env = "CASS_AGENT_TENANT_ID")]
    pub tenant_id: Option<String>,

    /// Project ID for this agent
    #[arg(short, long, env = "CASS_AGENT_PROJECT_ID")]
    pub project_id: Option<String>,

    /// API key with the `agent:execute` scope, sent as `x-api-key` on every call
    #[arg(short, long, env = "CASS_AGENT_SECRET", hide_env_values = true)]
    pub secret: String,

    /// Run `exec` commands and scripts sent by operators. Off by default:
    /// enabling it lets anyone holding an `agent:write` or `script:execute`
    /// credential run code on this host.
    #[arg(long, env = "CASS_AGENT_ALLOW_EXEC")]
    pub allow_exec: bool,

    /// Directory builds are deployed into (`<dir>/<build_id>/<file>`).
    /// Deployments are refused when unset.
    #[arg(long, env = "CASS_AGENT_DEPLOY_DIR")]
    pub deploy_dir: Option<PathBuf>,

    /// Node ID (a UUID). Defaults to one derived from the hostname, so the
    /// agent keeps its identity, jobs and history across restarts.
    #[arg(long, env = "CASS_AGENT_NODE_ID")]
    pub node_id: Option<Uuid>,

    /// Maximum consecutive heartbeat failures before attempting reconnection
    #[arg(long, env = "CASS_AGENT_MAX_FAILURES", default_value = "5")]
    pub max_failures: u32,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, env = "CASS_LOG_LEVEL", default_value = "info")]
    pub log_level: String,
}

/// Connect to the gateway with exponential backoff retry
async fn connect_with_retry(gateway: &str) -> Result<Channel> {
    let backoff = ExponentialBackoff {
        max_elapsed_time: Some(Duration::from_secs(300)), // 5 minutes max
        initial_interval: Duration::from_secs(1),
        max_interval: Duration::from_secs(30),
        ..Default::default()
    };

    let gateway = gateway.to_string();
    let channel = retry(backoff, || async {
        info!(gateway = %gateway, "Attempting to connect to gateway");
        Channel::from_shared(gateway.clone())
            .map_err(|e| backoff::Error::permanent(anyhow::anyhow!("invalid URI: {}", e)))?
            .connect()
            .await
            .map_err(|e| {
                warn!(error = %e, "Connection failed, retrying...");
                backoff::Error::transient(anyhow::anyhow!("connection failed: {}", e))
            })
    })
    .await?;

    info!("Successfully connected to gateway");
    Ok(channel)
}

/// Stable per host: a reconnect or restart must not turn the agent into a new
/// one (its queued jobs and history are keyed by this id). Hosts sharing a
/// hostname need an explicit `--node-id`.
fn default_node_id() -> Uuid {
    let host = System::host_name().unwrap_or_else(|| "unknown".into());
    Uuid::new_v5(
        &Uuid::NAMESPACE_DNS,
        format!("{host}.agents.cassandranet").as_bytes(),
    )
}

/// Attach the agent's credentials to an outgoing call.
fn authed<T>(message: T, api_key: &str) -> Result<tonic::Request<T>> {
    let mut request = tonic::Request::new(message);
    let value = api_key
        .parse()
        .context("CASS_AGENT_SECRET is not a valid header value")?;
    request.metadata_mut().insert("x-api-key", value);
    Ok(request)
}

/// Register this agent with the gateway
async fn register_agent(
    client: &mut AgentControlClient<Channel>,
    args: &Args,
    sys: &System,
) -> Result<(String, Duration)> {
    let node_id = args.node_id.unwrap_or_else(default_node_id).to_string();

    let req = RegisterAgentRequest {
        node_id: node_id.clone(),
        hostname: System::host_name().unwrap_or_else(|| "unknown".into()),
        os: std::env::consts::OS.into(),
        arch: std::env::consts::ARCH.into(),
        cpu_cores: sys.cpus().len() as u32,
        memory_bytes: sys.total_memory(),
        secret: args.secret.clone(),
        tenant_id: args.tenant_id.clone().unwrap_or_default(),
        project_id: args.project_id.clone().unwrap_or_default(),
    };

    info!(
        node_id = %node_id,
        hostname = %req.hostname,
        os = %req.os,
        arch = %req.arch,
        "Registering agent"
    );

    let resp = client
        .register_agent(authed(req, &args.secret)?)
        .await
        .context("Failed to register agent")?
        .into_inner();

    let interval = Duration::from_secs(resp.heartbeat_interval_seconds as u64);
    info!(
        assigned_id = %resp.assigned_id,
        heartbeat_interval_secs = resp.heartbeat_interval_seconds,
        "Agent registered successfully"
    );

    Ok((resp.assigned_id, interval))
}

/// Send a heartbeat and handle the response
async fn send_heartbeat(
    client: &mut AgentControlClient<Channel>,
    api_key: &str,
    assigned_id: &str,
    sys: &mut System,
) -> Result<HeartbeatResponse> {
    sys.refresh_cpu();
    sys.refresh_memory();

    let hb = HeartbeatRequest {
        assigned_id: assigned_id.to_string(),
        cpu_percent: sys.global_cpu_info().cpu_usage() as f64,
        memory_used_bytes: sys.used_memory(),
        network_rx_bytes: 0,
        network_tx_bytes: 0,
        timestamp_unix_ms: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
    };

    debug!(
        cpu = hb.cpu_percent,
        memory = hb.memory_used_bytes,
        "Sending heartbeat"
    );

    let resp = client
        .heartbeat(authed(hb, api_key)?)
        .await
        .context("Heartbeat failed")?
        .into_inner();

    if resp.rotate_credentials {
        info!(
            event = "credential_rotation",
            "Server requested credential rotation"
        );
    }

    Ok(resp)
}

/// Run one job and report its result, retrying the report briefly so a
/// network blip doesn't strand the job as "dispatched".
async fn run_job(
    runner: JobRunner,
    mut client: AgentControlClient<Channel>,
    api_key: String,
    assigned_id: String,
    job: cnproto::AgentJob,
    shutdown: Arc<AtomicBool>,
    restart: Arc<AtomicBool>,
) {
    let (outcome, after) = runner.run(&mut client, &api_key, &job).await;
    let report = outcome.into_report(&assigned_id, &job.job_id);
    for attempt in 1..=3 {
        match authed(report.clone(), &api_key) {
            Ok(request) => match client.report_job_result(request).await {
                Ok(_) => break,
                Err(err) => warn!(job_id = %job.job_id, attempt, error = %err, "job.report_failed"),
            },
            Err(err) => {
                error!(error = %err, "job.report_unauthorizable");
                break;
            }
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    match after {
        AfterReport::Continue => {}
        AfterReport::Restart => {
            restart.store(true, Ordering::Relaxed);
            shutdown.store(true, Ordering::Relaxed);
        }
        AfterReport::Shutdown => shutdown.store(true, Ordering::Relaxed),
    }
}

/// Main agent loop with reconnection support
async fn run_agent(
    args: Args,
    runner: JobRunner,
    shutdown: Arc<AtomicBool>,
    restart: Arc<AtomicBool>,
) -> Result<()> {
    let mut sys = System::new_all();
    sys.refresh_all();

    loop {
        if shutdown.load(Ordering::Relaxed) {
            info!("Shutdown requested, exiting agent loop");
            break;
        }

        // Connect with retry
        let channel = match connect_with_retry(&args.gateway).await {
            Ok(ch) => ch,
            Err(e) => {
                error!(error = %e, "Failed to connect after retries");
                if shutdown.load(Ordering::Relaxed) {
                    break;
                }
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }
        };

        let mut client = AgentControlClient::new(channel);

        // Register
        let (assigned_id, mut interval) = match register_agent(&mut client, &args, &sys).await {
            Ok(r) => r,
            Err(e) => {
                error!(error = %e, "Failed to register agent");
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }
        };

        // Heartbeat loop
        let mut consecutive_failures = 0u32;
        loop {
            if shutdown.load(Ordering::Relaxed) {
                info!("Shutdown requested during heartbeat loop");
                break;
            }

            tokio::time::sleep(interval).await;

            match send_heartbeat(&mut client, &args.secret, &assigned_id, &mut sys).await {
                Ok(resp) => {
                    consecutive_failures = 0;
                    if resp.heartbeat_interval_seconds > 0 {
                        let wanted = Duration::from_secs(resp.heartbeat_interval_seconds.into());
                        if wanted != interval {
                            info!(
                                seconds = resp.heartbeat_interval_seconds,
                                "heartbeat interval updated"
                            );
                            interval = wanted;
                        }
                    }
                    for job in resp.jobs {
                        tokio::spawn(run_job(
                            runner.clone(),
                            client.clone(),
                            args.secret.clone(),
                            assigned_id.clone(),
                            job,
                            shutdown.clone(),
                            restart.clone(),
                        ));
                    }
                }
                Err(e) => {
                    consecutive_failures += 1;
                    error!(
                        error = %e,
                        failures = consecutive_failures,
                        max = args.max_failures,
                        "Heartbeat failed"
                    );

                    if consecutive_failures >= args.max_failures {
                        warn!("Max consecutive failures reached, reconnecting...");
                        break;
                    }
                }
            }
        }

        if shutdown.load(Ordering::Relaxed) {
            break;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing based on log level
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&args.log_level));
    // Logs are also kept in memory for the `collect_logs` command.
    let logs = LogBuffer::default();
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(false)
        .with_writer(logs.clone())
        .init();

    info!(
        version = env!("CARGO_PKG_VERSION"),
        gateway = %args.gateway,
        tenant_id = ?args.tenant_id,
        project_id = ?args.project_id,
        allow_exec = args.allow_exec,
        deploy_dir = ?args.deploy_dir,
        "Starting CassandraNet Agent"
    );
    if args.allow_exec {
        warn!("remote execution enabled: operators can run commands and scripts on this host");
    }

    // Set up graceful shutdown
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = shutdown.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for ctrl+c");
        info!(event = "shutdown", "Received shutdown signal");
        shutdown_clone.store(true, Ordering::Relaxed);
    });

    let runner = JobRunner {
        allow_exec: args.allow_exec,
        deploy_dir: args.deploy_dir.clone(),
        logs,
    };
    let restart = Arc::new(AtomicBool::new(false));
    run_agent(args, runner, shutdown, restart.clone()).await?;
    if restart.load(Ordering::Relaxed) {
        info!(
            code = RESTART_EXIT_CODE,
            "exiting so the supervisor restarts the agent"
        );
        std::process::exit(RESTART_EXIT_CODE);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_defaults() {
        let args = Args::parse_from(["cnagent", "--secret", "k"]);
        assert_eq!(args.gateway, "http://127.0.0.1:8081");
        assert_eq!(args.max_failures, 5);
        assert!(!args.allow_exec, "remote execution must be opt-in");
        assert!(args.deploy_dir.is_none(), "deployments must be opt-in");
        assert!(args.tenant_id.is_none());
        assert!(args.project_id.is_none());
    }

    #[test]
    fn default_node_id_is_stable_per_host() {
        assert_eq!(default_node_id(), default_node_id());
        assert_eq!(default_node_id().get_version_num(), 5);
        assert!(
            Args::try_parse_from(["cnagent", "--secret", "k", "--node-id", "not-a-uuid"]).is_err()
        );
    }

    #[test]
    fn test_args_custom() {
        let args = Args::parse_from([
            "cnagent",
            "--gateway",
            "http://gateway.example.com:9000",
            "--tenant-id",
            "tenant-123",
            "--project-id",
            "project-456",
            "--secret",
            "my-secret",
            "--max-failures",
            "10",
        ]);
        assert_eq!(args.gateway, "http://gateway.example.com:9000");
        assert_eq!(args.tenant_id, Some("tenant-123".to_string()));
        assert_eq!(args.project_id, Some("project-456".to_string()));
        assert_eq!(args.secret, "my-secret");
        assert_eq!(args.max_failures, 10);
    }
}
