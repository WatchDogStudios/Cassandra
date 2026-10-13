//! Tenant management commands

use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use tabled::{Table, Tabled};

use super::{create_client, delete, get_json, post_json};
use crate::{Cli, OutputFormat};

#[derive(Subcommand, Debug)]
pub enum TenantCommands {
    /// List all tenants
    List,

    /// Get a specific tenant
    Get {
        /// Tenant ID
        id: String,
    },

    /// Create a new tenant
    Create {
        /// Tenant name
        #[arg(short, long)]
        name: String,

        /// Tenant plan (free, pro, enterprise)
        #[arg(short, long, default_value = "free")]
        plan: String,
    },

    /// Delete a tenant
    Delete {
        /// Tenant ID
        id: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTenantRequest {
    pub name: String,
    pub plan: String,
}

#[derive(Debug, Serialize, Deserialize, Tabled)]
pub struct TenantResponse {
    pub id: String,
    pub name: String,
    pub plan: String,
    pub created_at: String,
}

pub async fn execute(cli: &Cli, cmd: &TenantCommands) -> Result<()> {
    let client = create_client(cli.api_key.as_deref())?;
    let base_url = format!("{}/api/v1/tenants", cli.gateway);

    match cmd {
        TenantCommands::List => {
            let tenants: Vec<TenantResponse> = get_json(&client, &base_url).await?;
            output_list(&tenants, cli.format);
        }
        TenantCommands::Get { id } => {
            let url = format!("{}/{}", base_url, id);
            let tenant: TenantResponse = get_json(&client, &url).await?;
            output_single(&tenant, cli.format);
        }
        TenantCommands::Create { name, plan } => {
            let req = CreateTenantRequest {
                name: name.clone(),
                plan: plan.clone(),
            };
            let tenant: TenantResponse = post_json(&client, &base_url, &req).await?;
            println!("Created tenant: {}", tenant.id);
            output_single(&tenant, cli.format);
        }
        TenantCommands::Delete { id } => {
            let url = format!("{}/{}", base_url, id);
            delete(&client, &url).await?;
            println!("Deleted tenant: {}", id);
        }
    }

    Ok(())
}

fn output_list(tenants: &[TenantResponse], format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(tenants).unwrap());
        }
        OutputFormat::Table => {
            if tenants.is_empty() {
                println!("No tenants found.");
            } else {
                println!("{}", Table::new(tenants));
            }
        }
    }
}

fn output_single(tenant: &TenantResponse, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(tenant).unwrap());
        }
        OutputFormat::Table => {
            println!("{}", Table::new(std::iter::once(tenant)));
        }
    }
}
