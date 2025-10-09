use anyhow::Result;
use cncore::{config, init_tracing};
use cnproto::{agent_control_client::AgentControlClient, HeartbeatRequest, RegisterAgentRequest};
use sysinfo::{CpuExt, System, SystemExt};
use tonic::transport::Channel;
use tracing::{error, info};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    info!("agent.start", config=?config(), "Starting CassandraNet Agent prototype");
    // Connect to gateway gRPC (assumes default http bind +1 port for grpc as implemented)
    let http_addr = &config().http.bind_addr;
    let mut parts = http_addr.split(':').collect::<Vec<_>>();
    let port: u16 = parts.pop().unwrap_or("0").parse().unwrap_or(8080);
    let host = parts.join(":");
    let grpc_addr = format!("http://{}:{}", host, port + 1);
    let channel = loop {
        match Channel::from_shared(grpc_addr.clone())?.connect().await {
            Ok(ch) => break ch,
            Err(e) => {
                error!(error=%e, "waiting for grpc server");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    };
    let mut client = AgentControlClient::new(channel);
    let mut sys = System::new_all();
    sys.refresh_all();
    let node_id = Uuid::new_v4().to_string();
    let req = RegisterAgentRequest {
        node_id: node_id.clone(),
        hostname: sys.host_name().unwrap_or_else(|| "unknown".into()),
        os: std::env::consts::OS.into(),
        arch: std::env::consts::ARCH.into(),
        cpu_cores: sys.cpus().len() as u32,
        memory_bytes: sys.total_memory() * 1024,
        secret: "bootstrap-placeholder".into(),
    };
    let resp = client.register_agent(req).await?.into_inner();
    info!(assigned_id=%resp.assigned_id, interval=resp.heartbeat_interval_seconds, "agent registered");
    let assigned = resp.assigned_id;
    let interval = std::time::Duration::from_secs(resp.heartbeat_interval_seconds as u64);
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => { info!("agent.shutdown", "Shutting down agent"); break; }
            _ = tokio::time::sleep(interval) => {
                sys.refresh_cpu();
                sys.refresh_memory();
                let hb = HeartbeatRequest {
                    assigned_id: assigned.clone(),
                    cpu_percent: sys.global_cpu_info().cpu_usage() as f64,
                    memory_used_bytes: sys.used_memory() * 1024,
                    network_rx_bytes: 0,
                    network_tx_bytes: 0,
                    timestamp_unix_ms: (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()) as u64,
                };
                match client.heartbeat(hb).await {
                    Ok(r) => { let r = r.into_inner(); if r.rotate_credentials { info!("rotate.creds", "server requested credential rotation"); } }
                    Err(e) => error!(error=%e, "heartbeat failed"),
                }
            }
        }
    }
    Ok(())
}
