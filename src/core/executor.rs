use anyhow::Result;
use serde_json::json;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::broadcast;

pub struct ExecutorOutput {
    pub exit_code: i32,
    pub full_output: String,
}

pub async fn run_playbook(
    playbook_path: &str,
    inventory_path: &str,
    extra_vars: Option<&serde_json::Value>,
    job_id: &str,
    log_tx: &broadcast::Sender<String>,
) -> Result<ExecutorOutput> {
    let mut cmd = Command::new("ansible-playbook");
    cmd.arg(playbook_path)
        .arg("-i")
        .arg(inventory_path)
        .arg("--diff")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(vars) = extra_vars {
        cmd.arg("-e").arg(serde_json::to_string(vars)?);
    }

    // Set ANSIBLE_STDOUT_CALLBACK to json for structured output
    cmd.env("ANSIBLE_STDOUT_CALLBACK", "json");
    cmd.env("ANSIBLE_FORCE_COLOR", "false");

    let mut child = cmd.spawn()?;
    let mut full_output = String::new();

    // Stream stdout
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        let tx = log_tx.clone();
        let jid = job_id.to_string();

        while let Ok(Some(line)) = lines.next_line().await {
            full_output.push_str(&line);
            full_output.push('\n');

            let log_msg = json!({
                "job_id": jid,
                "type": "stdout",
                "line": line,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })
            .to_string();

            let _ = tx.send(log_msg);
        }
    }

    // Stream stderr
    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        let tx = log_tx.clone();
        let jid = job_id.to_string();

        while let Ok(Some(line)) = lines.next_line().await {
            full_output.push_str(&line);
            full_output.push('\n');

            let log_msg = json!({
                "job_id": jid,
                "type": "stderr",
                "line": line,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })
            .to_string();

            let _ = tx.send(log_msg);
        }
    }

    let status = child.wait().await?;

    // Send completion message
    let completion_msg = json!({
        "job_id": job_id,
        "type": "complete",
        "exit_code": status.code().unwrap_or(-1),
        "timestamp": chrono::Utc::now().to_rfc3339()
    })
    .to_string();
    let _ = log_tx.send(completion_msg);

    Ok(ExecutorOutput {
        exit_code: status.code().unwrap_or(-1),
        full_output,
    })
}
