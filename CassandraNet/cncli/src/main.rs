//! CassandraNet CLI - Command-line interface for CassandraNet platform management
//!
//! Provides subcommands for managing tenants, projects, agents, tasks, and workflows.

mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand};
use commands::{agents, projects, tasks, tenants, workflows};

/// CassandraNet CLI - Manage your CassandraNet platform
#[derive(Parser, Debug)]
#[command(name = "cncli", version, about, long_about = None)]
pub struct Cli {
    /// Gateway HTTP endpoint
    #[arg(
        short,
        long,
        env = "CASS_GATEWAY_URL",
        default_value = "http://127.0.0.1:8080"
    )]
    pub gateway: String,

    /// API key for authentication
    #[arg(short, long, env = "CASS_API_KEY")]
    pub api_key: Option<String>,

    /// Output format (json, table)
    #[arg(short, long, default_value = "table")]
    pub format: OutputFormat,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    Json,
    Table,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage tenants
    #[command(subcommand)]
    Tenant(tenants::TenantCommands),

    /// Manage projects
    #[command(subcommand)]
    Project(projects::ProjectCommands),

    /// Manage agents
    #[command(subcommand)]
    Agent(agents::AgentCommands),

    /// Manage tasks
    #[command(subcommand)]
    Task(tasks::TaskCommands),

    /// Manage workflows
    #[command(subcommand)]
    Workflow(workflows::WorkflowCommands),

    /// Show CLI version and gateway info
    Info,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Tenant(cmd) => tenants::execute(&cli, cmd).await?,
        Commands::Project(cmd) => projects::execute(&cli, cmd).await?,
        Commands::Agent(cmd) => agents::execute(&cli, cmd).await?,
        Commands::Task(cmd) => tasks::execute(&cli, cmd).await?,
        Commands::Workflow(cmd) => workflows::execute(&cli, cmd).await?,
        Commands::Info => {
            println!("CassandraNet CLI v{}", env!("CARGO_PKG_VERSION"));
            println!("Gateway: {}", cli.gateway);
            println!(
                "API Key: {}",
                if cli.api_key.is_some() {
                    "configured"
                } else {
                    "not set"
                }
            );
        }
    }

    Ok(())
}
