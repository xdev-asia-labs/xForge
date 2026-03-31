use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;

use crate::db::models::{KeyStoreEntry, Server};
use crate::AppState;

#[derive(Deserialize)]
pub struct TerminalQuery {
    pub server_id: String,
    pub token: String,
}

pub async fn terminal_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(query): Query<TerminalQuery>,
) -> impl IntoResponse {
    // Validate JWT token
    let claims = match jsonwebtoken::decode::<crate::db::models::Claims>(
        &query.token,
        &jsonwebtoken::DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    ) {
        Ok(token_data) => token_data.claims,
        Err(_) => {
            return axum::http::Response::builder()
                .status(401)
                .body(axum::body::Body::from("Unauthorized"))
                .unwrap()
                .into_response();
        }
    };

    // Load server details
    let server = match sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
        .bind(&query.server_id)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(s)) => s,
        _ => {
            return axum::http::Response::builder()
                .status(404)
                .body(axum::body::Body::from("Server not found"))
                .unwrap()
                .into_response();
        }
    };

    // Resolve SSH key if key_id is set
    let key_data = if let Some(ref key_id) = server.key_id {
        sqlx::query_as::<_, KeyStoreEntry>("SELECT * FROM key_store WHERE id = ?")
            .bind(key_id)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten()
    } else {
        None
    };

    let _claims = claims;
    ws.on_upgrade(move |socket| handle_terminal(socket, server, key_data))
}

async fn handle_terminal(socket: WebSocket, server: Server, key_data: Option<KeyStoreEntry>) {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Build SSH command
    let mut cmd = Command::new("ssh");
    cmd.arg("-tt")
        .arg("-o")
        .arg("StrictHostKeyChecking=no")
        .arg("-o")
        .arg("UserKnownHostsFile=/dev/null")
        .arg("-p")
        .arg(server.port.to_string());

    // Handle SSH key from key store (write to temp file)
    let temp_key_path = if let Some(ref key_entry) = key_data {
        if key_entry.key_type == "ssh_key" {
            let path = format!("/tmp/xforge-sshkey-{}", uuid::Uuid::new_v4());
            if tokio::fs::write(&path, &key_entry.key_data).await.is_ok() {
                // Set restrictive permissions
                let _ = tokio::process::Command::new("chmod")
                    .arg("600")
                    .arg(&path)
                    .output()
                    .await;
                cmd.arg("-i").arg(&path);
                Some(path)
            } else {
                None
            }
        } else {
            None
        }
    } else if let Some(ref key_path) = server.ssh_key_path {
        cmd.arg("-i").arg(key_path);
        None
    } else {
        None
    };

    cmd.arg(format!("{}@{}", server.ssh_user, server.host))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let msg = json!({"type": "error", "data": format!("Failed to start SSH: {}", e)});
            let _ = ws_sender.send(Message::Text(msg.to_string())).await;
            return;
        }
    };

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    // Send connected message
    let msg = json!({"type": "status", "data": "connected"});
    let _ = ws_sender.send(Message::Text(msg.to_string())).await;

    // Task: stdout → WS
    let ws_sender_clone = std::sync::Arc::new(tokio::sync::Mutex::new(ws_sender));
    let ws_sender_for_stdout = ws_sender_clone.clone();
    let stdout_task = tokio::spawn(async move {
        let mut reader = tokio::io::BufReader::new(stdout);
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    let text = String::from_utf8_lossy(&buf[..n]).to_string();
                    let msg = json!({"type": "output", "data": text});
                    let mut sender = ws_sender_for_stdout.lock().await;
                    if sender.send(Message::Text(msg.to_string())).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Task: stderr → WS
    let ws_sender_for_stderr = ws_sender_clone.clone();
    let stderr_task = tokio::spawn(async move {
        let mut reader = tokio::io::BufReader::new(stderr);
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    let text = String::from_utf8_lossy(&buf[..n]).to_string();
                    let msg = json!({"type": "output", "data": text});
                    let mut sender = ws_sender_for_stderr.lock().await;
                    if sender.send(Message::Text(msg.to_string())).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Task: WS → stdin
    let stdin_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                        match parsed.get("type").and_then(|t| t.as_str()) {
                            Some("input") => {
                                if let Some(data) = parsed.get("data").and_then(|d| d.as_str()) {
                                    let _ = stdin.write_all(data.as_bytes()).await;
                                    let _ = stdin.flush().await;
                                }
                            }
                            Some("resize") => {
                                // Remote resize via stty (best effort)
                                let cols = parsed.get("cols").and_then(|c| c.as_u64()).unwrap_or(80);
                                let rows = parsed.get("rows").and_then(|r| r.as_u64()).unwrap_or(24);
                                let resize_cmd = format!("stty cols {} rows {}\n", cols, rows);
                                let _ = stdin.write_all(resize_cmd.as_bytes()).await;
                                let _ = stdin.flush().await;
                            }
                            _ => {}
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait for any task to finish
    tokio::select! {
        _ = stdout_task => {},
        _ = stderr_task => {},
        _ = stdin_task => {},
    }

    // Cleanup
    let _ = child.kill().await;

    // Remove temp key file if created
    if let Some(path) = temp_key_path {
        let _ = tokio::fs::remove_file(path).await;
    }

    // Send disconnect message
    let ws_sender_final = ws_sender_clone.clone();
    let msg = json!({"type": "status", "data": "disconnected"});
    let mut sender = ws_sender_final.lock().await;
    let _ = sender.send(Message::Text(msg.to_string())).await;
}
