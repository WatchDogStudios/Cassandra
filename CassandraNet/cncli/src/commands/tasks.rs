//! Task management commands

use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use tabled::{Table, Tabled};

use super::{create_client, delete, get_json, post_json};
use crate::{Cli, OutputFormat};

#[derive(Subcommand, Debug)]
pub enum TaskCommands {
    /// List all tasks
    List {
        /// Filter by project ID
        #[arg(short, long)]
        project_id: Option<String>,

        /// Filter by status
        #[arg(short, long)]
        status: Option<String>,
    },

    /// Get a specific task
    Get {
        /// Task ID
        id: String,
    },

    /// Create a new task
    Create {
        /// Project ID
        #[arg(short, long)]
        project_id: String,

        /// Task name
        #[arg(short, long)]
        name: String,

        /// Task type (e.g., build, deploy, test)
        #[arg(short = 't', long, default_value = "generic")]
        task_type: String,

        /// Task payload (JSON string)
        #[arg(long)]
        payload: Option<String>,
    },

    /// Cancel a task
    Cancel {
        /// Task ID
        id: String,
    },

    /// Delete a task
    Delete {
        /// Task ID
        id: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub project_id: String,
    pub name: String,
    pub task_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Tabled)]
pub struct TaskResponse {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub task_type: String,
    pub status: String,
    pub created_at: String,
}

pub async fn execute(cli: &Cli, cmd: &TaskCommands) -> Result<()> {
    let client = create_client(cli.api_key.as_deref())?;
    let base_url = format!("{}/api/v1/tasks", cli.gateway);

    match cmd {
        TaskCommands::List { project_id, status } => {
            let mut url = base_url.clone();
            let mut params = vec![];
            if let Some(pid) = project_id {
                params.push(format!("project_id={}", pid));
            }
            if let Some(s) = status {
                params.push(format!("status={}", s));
            }
            if !params.is_empty() {
                url = format!("{}?{}", url, params.join("&"));
            }
            let tasks: Vec<TaskResponse> = get_json(&client, &url).await?;
            output_list(&tasks, cli.format);
        }
        TaskCommands::Get { id } => {
            let url = format!("{}/{}", base_url, id);
            let task: TaskResponse = get_json(&client, &url).await?;
            output_single(&task, cli.format);
        }
        TaskCommands::Create {
            project_id,
            name,
            task_type,
            payload,
        } => {
            let payload_value = if let Some(p) = payload {
                Some(serde_json::from_str(p)?)
            } else {
                None
            };
            let req = CreateTaskRequest {
                project_id: project_id.clone(),
                name: name.clone(),
                task_type: task_type.clone(),
                payload: payload_value,
            };
            let task: TaskResponse = post_json(&client, &base_url, &req).await?;
            println!("Created task: {}", task.id);
            output_single(&task, cli.format);
        }
        TaskCommands::Cancel { id } => {
            let url = format!("{}/{}/cancel", base_url, id);
            let _: serde_json::Value = post_json(&client, &url, &serde_json::json!({})).await?;
            println!("Cancelled task: {}", id);
        }
        TaskCommands::Delete { id } => {
            let url = format!("{}/{}", base_url, id);
            delete(&client, &url).await?;
            println!("Deleted task: {}", id);
        }
    }

    Ok(())
}

fn output_list(tasks: &[TaskResponse], format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(tasks).unwrap());
        }
        OutputFormat::Table => {
            if tasks.is_empty() {
                println!("No tasks found.");
            } else {
                println!("{}", Table::new(tasks));
            }
        }
    }
}

fn output_single(task: &TaskResponse, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(task).unwrap());
        }
        OutputFormat::Table => {
            println!("{}", Table::new(std::iter::once(task)));
        }
    }
}
