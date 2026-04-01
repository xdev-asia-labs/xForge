use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;

use crate::AppState;

#[derive(Deserialize)]
pub struct WsQuery {
    pub job_id: Option<String>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, query.job_id))
}

async fn handle_socket(socket: WebSocket, state: AppState, job_id: Option<String>) {
    let (mut sender, mut receiver) = socket.split();

    let mut rx = state.log_broadcast.subscribe();

    let job_filter = job_id.clone();

    // Task to forward broadcast messages to the WebSocket client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // Filter by job_id if specified
            if let Some(ref filter) = job_filter {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&msg) {
                    if let Some(msg_job_id) = parsed.get("job_id").and_then(|v| v.as_str()) {
                        if msg_job_id != filter {
                            continue;
                        }
                    }
                }
            }

            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Task to receive messages from the client (ping/pong, close)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
}
