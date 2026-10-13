//! Agent management commands

use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use tabled::{Table, Tabled};

use super::{create_client, delete, get_json};
use crate::{Cli, OutputFormat};

#[derive(Subcommand, Debug)]
pub enum AgentCommands {
    /// List all agents
    List {
        /// Filter by tenant ID
        #[arg(short, long)]
        tenant_id: Option<String>,

        /// Filter by project ID
        #[arg(short, long)]
        project_id: Option<String>,
    },

    /// Get a specific agent
    Get {
        /// Agent ID
        id: String,
    },

    /// Get agent status/health
    Status {
        /// Agent ID
        id: String,
    },

    /// Disconnect an agent
    Disconnect {
        /// Agent ID
        id: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Tabled)]
pub struct AgentResponse {
    pub id: String,
    pub node_id: String,
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub status: String,
    pub last_heartbeat: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentStatusResponse {
    pub id: String,
    pub status: String,
    pub cpu_percent: f64,
    pub memory_used_bytes: u64,
    pub last_heartbeat: String,
    pub uptime_seconds: u64,
}

pub async fn execute(cli: &Cli, cmd: &AgentCommands) -> Result<()> {
    let client = create_client(cli.api_key.as_deref())?;
    let base_url = format!("{}/api/v1/agents", cli.gateway);

    match cmd {
        AgentCommands::List {
            tenant_id,
            project_id,
        } => {
            let mut url = base_url.clone();
            let mut params = vec![];
            if let Some(tid) = tenant_id {
                params.push(format!("tenant_id={}", tid));
            }
            if let Some(pid) = project_id {
                params.push(format!("project_id={}", pid));
            }
            if !params.is_empty() {
                url = format!("{}?{}", url, params.join("&"));
            }
            let agents: Vec<AgentResponse> = get_json(&client, &url).await?;
            output_list(&agents, cli.format);
        }
        AgentCommands::Get { id } => {
            let url = format!("{}/{}", base_url, id);
            let agent: AgentResponse = get_json(&client, &url).await?;
            output_single(&agent, cli.format);
        }
        AgentCommands::Status { id } => {
            let url = format!("{}/{}/status", base_url, id);
            let status: AgentStatusResponse = get_json(&client, &url).await?;
            match cli.format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&status).unwrap());
                }
                OutputFormat::Table => {
                    println!("Agent: {}", status.id);
                    println!("Status: {}", status.status);
                    println!("CPU: {:.1}%", status.cpu_percent);
                    println!("Memory: {} bytes", status.memory_used_bytes);
                    println!("Last Heartbeat: {}", status.last_heartbeat);
                    println!("Uptime: {} seconds", status.uptime_seconds);
                }
            }
        }
        AgentCommands::Disconnect { id } => {
            let url = format!("{}/{}", base_url, id);
            delete(&client, &url).await?;
            println!("Disconnected agent: {}", id);
        }
    }

    Ok(())
}

fn output_list(agents: &[AgentResponse], format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(agents).unwrap());
        }
        OutputFormat::Table => {
            if agents.is_empty() {
                println!("No agents found.");
            } else {
                println!("{}", Table::new(agents));
            }
        }
    }
}

fn output_single(agent: &AgentResponse, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(agent).unwrap());
        }
        OutputFormat::Table => {
            println!("{}", Table::new(std::iter::once(agent)));
        }
    }
}
