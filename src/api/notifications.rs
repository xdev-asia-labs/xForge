use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::db::models::{
    Claims, CreateNotificationChannel, NotificationChannel, NotificationChannelResponse,
};
use crate::AppState;

pub async fn list_channels(
    State(state): State<AppState>,
) -> Result<Json<Vec<NotificationChannelResponse>>, (StatusCode, Json<serde_json::Value>)> {
    let channels = sqlx::query_as::<_, NotificationChannel>(
        "SELECT * FROM notification_channels ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
    })?;

    Ok(Json(
        channels
            .into_iter()
            .map(NotificationChannelResponse::from)
            .collect(),
    ))
}

pub async fn create_channel(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateNotificationChannel>,
) -> Result<(StatusCode, Json<NotificationChannelResponse>), (StatusCode, Json<serde_json::Value>)>
{
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Channel name is required"})),
        ));
    }

    if payload.channel_type != "webhook" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Only 'webhook' channel type is supported"})),
        ));
    }

    // Validate webhook config has a url
    if payload.config.get("url").and_then(|u| u.as_str()).map(|u| u.is_empty()).unwrap_or(true) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Webhook config must include a 'url' field"})),
        ));
    }

    let id = Uuid::new_v4().to_string();
    let config_json = serde_json::to_string(&payload.config).unwrap();
    let events_json = serde_json::to_string(&payload.events).unwrap();

    sqlx::query(
        "INSERT INTO notification_channels (id, name, channel_type, config, events, created_by) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(payload.name.trim())
    .bind(&payload.channel_type)
    .bind(&config_json)
    .bind(&events_json)
    .bind(&claims.sub)
    .execute(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
    })?;

    let channel = sqlx::query_as::<_, NotificationChannel>(
        "SELECT * FROM notification_channels WHERE id = ?",
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
    })?;

    Ok((
        StatusCode::CREATED,
        Json(NotificationChannelResponse::from(channel)),
    ))
}

pub async fn delete_channel(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let result = sqlx::query("DELETE FROM notification_channels WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", e)})),
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Channel not found"})),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Send notification for a job event
pub async fn send_job_notification(
    db: &sqlx::SqlitePool,
    event: &str,
    job_id: &str,
    recipe_name: &str,
    status: &str,
) {
    let channels = match sqlx::query_as::<_, NotificationChannel>(
        "SELECT * FROM notification_channels WHERE enabled = 1",
    )
    .fetch_all(db)
    .await
    {
        Ok(c) => c,
        Err(_) => return,
    };

    let payload = json!({
        "event": event,
        "job": {
            "id": job_id,
            "recipe_name": recipe_name,
            "status": status,
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    for channel in channels {
        let events: Vec<String> = serde_json::from_str(&channel.events).unwrap_or_default();
        if !events.contains(&event.to_string()) {
            continue;
        }

        if channel.channel_type == "webhook" {
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&channel.config) {
                if let Some(url) = config.get("url").and_then(|u| u.as_str()) {
                    let client = reqwest::Client::new();
                    let mut req = client.post(url).json(&payload);

                    // Add custom headers if configured
                    if let Some(headers) = config.get("headers").and_then(|h| h.as_object()) {
                        for (key, value) in headers {
                            if let Some(v) = value.as_str() {
                                if let (Ok(name), Ok(val)) = (
                                    reqwest::header::HeaderName::from_bytes(key.as_bytes()),
                                    reqwest::header::HeaderValue::from_str(v),
                                ) {
                                    req = req.header(name, val);
                                }
                            }
                        }
                    }

                    if let Err(e) = req.send().await {
                        tracing::warn!(
                            "Failed to send webhook notification to {}: {}",
                            channel.name,
                            e
                        );
                    }
                }
            }
        }
    }
}
