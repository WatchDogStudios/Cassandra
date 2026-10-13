//! Project management commands

use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use tabled::{Table, Tabled};

use super::{create_client, delete, get_json, post_json};
use crate::{Cli, OutputFormat};

#[derive(Subcommand, Debug)]
pub enum ProjectCommands {
    /// List all projects
    List {
        /// Filter by tenant ID
        #[arg(short, long)]
        tenant_id: Option<String>,
    },

    /// Get a specific project
    Get {
        /// Project ID
        id: String,
    },

    /// Create a new project
    Create {
        /// Tenant ID
        #[arg(short, long)]
        tenant_id: String,

        /// Project name
        #[arg(short, long)]
        name: String,

        /// Project description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Delete a project
    Delete {
        /// Project ID
        id: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub tenant_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Tabled)]
pub struct ProjectResponse {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    #[tabled(display_with = "display_option")]
    pub description: Option<String>,
    pub created_at: String,
}

fn display_option(opt: &Option<String>) -> String {
    opt.clone().unwrap_or_else(|| "-".to_string())
}

pub async fn execute(cli: &Cli, cmd: &ProjectCommands) -> Result<()> {
    let client = create_client(cli.api_key.as_deref())?;
    let base_url = format!("{}/api/v1/projects", cli.gateway);

    match cmd {
        ProjectCommands::List { tenant_id } => {
            let url = if let Some(tid) = tenant_id {
                format!("{}?tenant_id={}", base_url, tid)
            } else {
                base_url.clone()
            };
            let projects: Vec<ProjectResponse> = get_json(&client, &url).await?;
            output_list(&projects, cli.format);
        }
        ProjectCommands::Get { id } => {
            let url = format!("{}/{}", base_url, id);
            let project: ProjectResponse = get_json(&client, &url).await?;
            output_single(&project, cli.format);
        }
        ProjectCommands::Create {
            tenant_id,
            name,
            description,
        } => {
            let req = CreateProjectRequest {
                tenant_id: tenant_id.clone(),
                name: name.clone(),
                description: description.clone(),
            };
            let project: ProjectResponse = post_json(&client, &base_url, &req).await?;
            println!("Created project: {}", project.id);
            output_single(&project, cli.format);
        }
        ProjectCommands::Delete { id } => {
            let url = format!("{}/{}", base_url, id);
            delete(&client, &url).await?;
            println!("Deleted project: {}", id);
        }
    }

    Ok(())
}

fn output_list(projects: &[ProjectResponse], format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(projects).unwrap());
        }
        OutputFormat::Table => {
            if projects.is_empty() {
                println!("No projects found.");
            } else {
                println!("{}", Table::new(projects));
            }
        }
    }
}

fn output_single(project: &ProjectResponse, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(project).unwrap());
        }
        OutputFormat::Table => {
            println!("{}", Table::new(std::iter::once(project)));
        }
    }
}
