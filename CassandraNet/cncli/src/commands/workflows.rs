//! Workflow management commands

use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use tabled::{Table, Tabled};

use super::{create_client, delete, get_json, post_json};
use crate::{Cli, OutputFormat};

#[derive(Subcommand, Debug)]
pub enum WorkflowCommands {
    /// List all workflows
    List {
        /// Filter by project ID
        #[arg(short, long)]
        project_id: Option<String>,
    },

    /// Get a specific workflow
    Get {
        /// Workflow ID
        id: String,
    },

    /// Create a new workflow
    Create {
        /// Project ID
        #[arg(short, long)]
        project_id: String,

        /// Workflow name
        #[arg(short, long)]
        name: String,

        /// Workflow definition (JSON string)
        #[arg(long)]
        definition: Option<String>,
    },

    /// Start a workflow
    Start {
        /// Workflow ID
        id: String,
    },

    /// Pause a workflow
    Pause {
        /// Workflow ID
        id: String,
    },

    /// Resume a workflow
    Resume {
        /// Workflow ID
        id: String,
    },

    /// Delete a workflow
    Delete {
        /// Workflow ID
        id: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateWorkflowRequest {
    pub project_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Tabled)]
pub struct WorkflowResponse {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub status: String,
    pub created_at: String,
}

pub async fn execute(cli: &Cli, cmd: &WorkflowCommands) -> Result<()> {
    let client = create_client(cli.api_key.as_deref())?;
    let base_url = format!("{}/api/v1/workflows", cli.gateway);

    match cmd {
        WorkflowCommands::List { project_id } => {
            let url = if let Some(pid) = project_id {
                format!("{}?project_id={}", base_url, pid)
            } else {
                base_url.clone()
            };
            let workflows: Vec<WorkflowResponse> = get_json(&client, &url).await?;
            output_list(&workflows, cli.format);
        }
        WorkflowCommands::Get { id } => {
            let url = format!("{}/{}", base_url, id);
            let workflow: WorkflowResponse = get_json(&client, &url).await?;
            output_single(&workflow, cli.format);
        }
        WorkflowCommands::Create {
            project_id,
            name,
            definition,
        } => {
            let def_value = if let Some(d) = definition {
                Some(serde_json::from_str(d)?)
            } else {
                None
            };
            let req = CreateWorkflowRequest {
                project_id: project_id.clone(),
                name: name.clone(),
                definition: def_value,
            };
            let workflow: WorkflowResponse = post_json(&client, &base_url, &req).await?;
            println!("Created workflow: {}", workflow.id);
            output_single(&workflow, cli.format);
        }
        WorkflowCommands::Start { id } => {
            let url = format!("{}/{}/start", base_url, id);
            let _: serde_json::Value = post_json(&client, &url, &serde_json::json!({})).await?;
            println!("Started workflow: {}", id);
        }
        WorkflowCommands::Pause { id } => {
            let url = format!("{}/{}/pause", base_url, id);
            let _: serde_json::Value = post_json(&client, &url, &serde_json::json!({})).await?;
            println!("Paused workflow: {}", id);
        }
        WorkflowCommands::Resume { id } => {
            let url = format!("{}/{}/resume", base_url, id);
            let _: serde_json::Value = post_json(&client, &url, &serde_json::json!({})).await?;
            println!("Resumed workflow: {}", id);
        }
        WorkflowCommands::Delete { id } => {
            let url = format!("{}/{}", base_url, id);
            delete(&client, &url).await?;
            println!("Deleted workflow: {}", id);
        }
    }

    Ok(())
}

fn output_list(workflows: &[WorkflowResponse], format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(workflows).unwrap());
        }
        OutputFormat::Table => {
            if workflows.is_empty() {
                println!("No workflows found.");
            } else {
                println!("{}", Table::new(workflows));
            }
        }
    }
}

fn output_single(workflow: &WorkflowResponse, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(workflow).unwrap());
        }
        OutputFormat::Table => {
            println!("{}", Table::new(std::iter::once(workflow)));
        }
    }
}
