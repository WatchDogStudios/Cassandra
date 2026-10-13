//! Runs the jobs the gateway hands out on heartbeat and builds their results.
//!
//! `exec` and `script` run arbitrary code and are refused unless the agent was
//! started with `--allow-exec`; `deploy_build` needs `--deploy-dir`.
use anyhow::{anyhow, bail, Context, Result};
use cncore::platform::{clamp_output, MAX_JOB_OUTPUT_BYTES};
use cnproto::{
    agent_control_client::AgentControlClient, AgentJob, DownloadBuildRequest,
    ReportJobResultRequest,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::io::Write;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tonic::transport::Channel;
use tracing_subscriber::fmt::MakeWriter;

/// What the agent process should do once a job's result has been reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AfterReport {
    Continue,
    /// Exit with `RESTART_EXIT_CODE` so a supervisor starts the agent again.
    Restart,
    Shutdown,
}

/// EX_TEMPFAIL: tells systemd (`Restart=on-failure`) or Docker
/// (`restart: on-failure`) to start the agent again.
pub const RESTART_EXIT_CODE: i32 = 75;

#[derive(Clone)]
pub struct JobRunner {
    pub allow_exec: bool,
    pub deploy_dir: Option<PathBuf>,
    pub logs: LogBuffer,
}

#[derive(Debug, Default)]
pub struct Outcome {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub error: Option<String>,
}

impl Outcome {
    fn ok(stdout: impl Into<String>) -> Self {
        Self {
            success: true,
            stdout: stdout.into(),
            ..Default::default()
        }
    }

    fn failed(error: impl std::fmt::Display) -> Self {
        Self {
            error: Some(error.to_string()),
            ..Default::default()
        }
    }

    pub fn into_report(self, assigned_id: &str, job_id: &str) -> ReportJobResultRequest {
        ReportJobResultRequest {
            assigned_id: assigned_id.to_string(),
            job_id: job_id.to_string(),
            success: self.success,
            exit_code: self.exit_code.unwrap_or_default(),
            has_exit_code: self.exit_code.is_some(),
            stdout: clamp_output(self.stdout),
            stderr: clamp_output(self.stderr),
            error: self.error.unwrap_or_default(),
        }
    }
}

impl JobRunner {
    pub async fn run(
        &self,
        client: &mut AgentControlClient<Channel>,
        api_key: &str,
        job: &AgentJob,
    ) -> (Outcome, AfterReport) {
        let args: Value = serde_json::from_str(&job.args_json).unwrap_or(Value::Null);
        let timeout = Duration::from_secs(u64::from(job.timeout_seconds.max(1)));
        tracing::info!(job_id = %job.job_id, kind = %job.kind, "job.started");
        let result = match job.kind.as_str() {
            "restart" => return (Outcome::ok("restarting agent"), AfterReport::Restart),
            "shutdown" => return (Outcome::ok("shutting down agent"), AfterReport::Shutdown),
            "collect_logs" => Ok(Outcome::ok(tail(
                self.logs.contents(),
                MAX_JOB_OUTPUT_BYTES,
            ))),
            "exec" => self.exec(&args, timeout).await,
            "script" => self.script(&job.job_id, &args, timeout).await,
            "deploy_build" => self.deploy(client, api_key, &args).await,
            other => Err(anyhow!("unsupported job kind: {other}")),
        };
        let outcome = result.unwrap_or_else(Outcome::failed);
        tracing::info!(
            job_id = %job.job_id,
            success = outcome.success,
            exit_code = ?outcome.exit_code,
            error = ?outcome.error,
            "job.finished"
        );
        (outcome, AfterReport::Continue)
    }

    fn require_exec(&self) -> Result<()> {
        if !self.allow_exec {
            bail!("remote execution is disabled on this agent (start it with --allow-exec)");
        }
        Ok(())
    }

    async fn exec(&self, args: &Value, timeout: Duration) -> Result<Outcome> {
        self.require_exec()?;
        let command = args
            .get("command")
            .and_then(Value::as_str)
            .filter(|c| !c.trim().is_empty())
            .context("exec requires args.command")?;
        let mut cmd = if cfg!(windows) {
            let mut c = tokio::process::Command::new("cmd");
            c.args(["/C", command]);
            c
        } else {
            let mut c = tokio::process::Command::new("sh");
            c.args(["-c", command]);
            c
        };
        run_process(&mut cmd, timeout).await
    }

    async fn script(&self, job_id: &str, args: &Value, timeout: Duration) -> Result<Outcome> {
        self.require_exec()?;
        let language = args.get("language").and_then(Value::as_str).unwrap_or("");
        let content = args
            .get("content")
            .and_then(Value::as_str)
            .context("script job has no content")?;
        let (program, flags, extension) =
            interpreter(language).with_context(|| format!("unsupported language: {language}"))?;

        let path = std::env::temp_dir().join(format!("cass-job-{job_id}.{extension}"));
        tokio::fs::write(&path, content)
            .await
            .with_context(|| format!("writing {}", path.display()))?;
        let mut cmd = tokio::process::Command::new(program);
        cmd.args(flags).arg(&path);
        if let Some(Value::Object(values)) = args.get("args") {
            for (name, value) in values {
                let value = match value {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                cmd.env(arg_env_name(name), value);
            }
        }
        let outcome = run_process(&mut cmd, timeout).await;
        let _ = tokio::fs::remove_file(&path).await;
        outcome
    }

    async fn deploy(
        &self,
        client: &mut AgentControlClient<Channel>,
        api_key: &str,
        args: &Value,
    ) -> Result<Outcome> {
        let deploy_dir = self
            .deploy_dir
            .as_ref()
            .context("deployments are disabled on this agent (start it with --deploy-dir)")?;
        let build_id = args
            .get("build_id")
            .and_then(Value::as_str)
            .and_then(|s| uuid::Uuid::parse_str(s).ok())
            .context("deploy_build requires args.build_id")?
            .to_string();

        let mut request = tonic::Request::new(DownloadBuildRequest {
            build_id: build_id.clone(),
        });
        request.metadata_mut().insert(
            "x-api-key",
            api_key.parse().context("invalid agent secret")?,
        );
        let mut stream = client
            .download_build(request)
            .await
            .context("download request failed")?
            .into_inner();
        let meta = stream
            .message()
            .await
            .context("download failed")?
            .context("empty download")?;

        let target_dir = deploy_dir.join(&build_id);
        tokio::fs::create_dir_all(&target_dir)
            .await
            .with_context(|| format!("creating {}", target_dir.display()))?;
        let filename = safe_filename(&meta.filename);
        let final_path = target_dir.join(&filename);
        let part_path = target_dir.join(format!("{filename}.part"));

        let written = async {
            let mut file = tokio::fs::File::create(&part_path).await?;
            let mut hasher = Sha256::new();
            let mut size = 0u64;
            while let Some(chunk) = stream.message().await.context("download interrupted")? {
                hasher.update(&chunk.data);
                size += chunk.data.len() as u64;
                file.write_all(&chunk.data).await?;
            }
            file.flush().await?;
            anyhow::Ok((size, hex::encode(hasher.finalize())))
        }
        .await;
        let (size, checksum) = match written {
            Ok(v) => v,
            Err(err) => {
                let _ = tokio::fs::remove_file(&part_path).await;
                return Err(err);
            }
        };
        if size != meta.size_bytes || checksum != meta.checksum_sha256 {
            let _ = tokio::fs::remove_file(&part_path).await;
            bail!(
                "integrity check failed: got {size} bytes / sha256 {checksum}, expected {} / {}",
                meta.size_bytes,
                meta.checksum_sha256
            );
        }
        tokio::fs::rename(&part_path, &final_path).await?;
        Ok(Outcome::ok(format!(
            "deployed {} ({size} bytes, sha256 {checksum})",
            final_path.display()
        )))
    }
}

/// Run to completion or until `timeout`, capturing output. On timeout the
/// child is killed (direct child only; descendants it spawned may outlive it).
async fn run_process(cmd: &mut tokio::process::Command, timeout: Duration) -> Result<Outcome> {
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let child = cmd.spawn().context("failed to start process")?;
    match tokio::time::timeout(timeout, child.wait_with_output()).await {
        Ok(output) => {
            let output = output.context("failed to wait for process")?;
            Ok(Outcome {
                success: output.status.success(),
                exit_code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                error: None,
            })
        }
        Err(_) => bail!("timed out after {}s", timeout.as_secs()),
    }
}

/// (program, flags before the script path, file extension)
fn interpreter(language: &str) -> Option<(&'static str, &'static [&'static str], &'static str)> {
    Some(match language {
        "bash" => ("bash", &[], "sh"),
        "sh" => ("sh", &[], "sh"),
        "powershell" if cfg!(windows) => (
            "powershell",
            &[
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
            ],
            "ps1",
        ),
        "powershell" => ("pwsh", &["-NoProfile", "-NonInteractive", "-File"], "ps1"),
        "python" if cfg!(windows) => ("python", &[], "py"),
        "python" => ("python3", &[], "py"),
        "javascript" => ("node", &[], "js"),
        "lua" => ("lua", &[], "lua"),
        _ => return None,
    })
}

/// `max-players` -> `CASS_ARG_MAX_PLAYERS`
fn arg_env_name(name: &str) -> String {
    let upper: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect();
    format!("CASS_ARG_{upper}")
}

/// The gateway already sanitizes names; never trust that on the receiving end.
fn safe_filename(raw: &str) -> String {
    let base = raw.rsplit(['/', '\\']).next().unwrap_or_default();
    let cleaned: String = base
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
        .take(128)
        .collect();
    match cleaned.trim_matches('.') {
        "" => "build.bin".to_string(),
        name => name.to_string(),
    }
}

/// Keep the last `max` bytes (on a char boundary): for logs the end matters.
fn tail(text: String, max: usize) -> String {
    if text.len() <= max {
        return text;
    }
    let mut start = text.len() - max;
    while !text.is_char_boundary(start) {
        start += 1;
    }
    text[start..].to_string()
}

/// Recent log output for `collect_logs`, filled by teeing the log writer.
#[derive(Clone, Default)]
pub struct LogBuffer(Arc<Mutex<VecDeque<String>>>);

const MAX_LOG_LINES: usize = 1000;

impl LogBuffer {
    fn push(&self, line: String) {
        let mut lines = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if lines.len() == MAX_LOG_LINES {
            lines.pop_front();
        }
        lines.push_back(line);
    }

    pub fn contents(&self) -> String {
        let lines = self.0.lock().unwrap_or_else(|e| e.into_inner());
        lines.iter().map(String::as_str).collect()
    }
}

/// Writes each formatted log event to stdout and keeps a copy.
pub struct TeeWriter(LogBuffer);

impl Write for TeeWriter {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        std::io::stdout().write_all(data)?;
        self.0.push(String::from_utf8_lossy(data).into_owned());
        Ok(data.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        std::io::stdout().flush()
    }
}

impl<'a> MakeWriter<'a> for LogBuffer {
    type Writer = TeeWriter;

    fn make_writer(&'a self) -> TeeWriter {
        TeeWriter(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn runner(allow_exec: bool) -> JobRunner {
        JobRunner {
            allow_exec,
            deploy_dir: None,
            logs: LogBuffer::default(),
        }
    }

    #[tokio::test]
    async fn exec_is_refused_unless_enabled() {
        let err = runner(false)
            .exec(&json!({"command": "echo hi"}), Duration::from_secs(5))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("--allow-exec"));
    }

    #[tokio::test]
    async fn exec_captures_output_and_exit_code() {
        let outcome = runner(true)
            .exec(&json!({"command": "echo hello"}), Duration::from_secs(10))
            .await
            .unwrap();
        assert!(outcome.success);
        assert_eq!(outcome.exit_code, Some(0));
        assert_eq!(outcome.stdout.trim(), "hello");

        let failing = runner(true)
            .exec(&json!({"command": "exit 3"}), Duration::from_secs(10))
            .await
            .unwrap();
        assert!(!failing.success);
        assert_eq!(failing.exit_code, Some(3));
    }

    #[tokio::test]
    async fn exec_times_out() {
        let command = if cfg!(windows) {
            "ping -n 3 127.0.0.1 >NUL"
        } else {
            "sleep 3"
        };
        let err = runner(true)
            .exec(&json!({"command": command}), Duration::from_millis(500))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("timed out"));
    }

    #[test]
    fn deployed_filenames_stay_in_the_target_dir() {
        assert_eq!(safe_filename("../../etc/cron.d/x"), "x");
        assert_eq!(safe_filename("..\\..\\evil.exe"), "evil.exe");
        assert_eq!(safe_filename(".."), "build.bin");
        assert_eq!(safe_filename(""), "build.bin");
    }

    #[test]
    fn script_args_become_env_vars() {
        assert_eq!(arg_env_name("max-players"), "CASS_ARG_MAX_PLAYERS");
        assert_eq!(arg_env_name("map"), "CASS_ARG_MAP");
    }

    #[test]
    fn tail_keeps_the_end_on_a_char_boundary() {
        assert_eq!(tail("abcdef".into(), 3), "def");
        assert_eq!(tail("éé".into(), 3), "é");
    }
}
