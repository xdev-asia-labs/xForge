use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::db::models::{CreateServer, Server, ServerResponse, UpdateServer};
use crate::AppState;

pub async fn list_servers(
    State(state): State<AppState>,
) -> Result<Json<Vec<ServerResponse>>, (StatusCode, Json<serde_json::Value>)> {
    let servers = sqlx::query_as::<_, Server>("SELECT * FROM servers ORDER BY created_at DESC")
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", e)})),
            )
        })?;

    Ok(Json(servers.into_iter().map(ServerResponse::from).collect()))
}

pub async fn get_server(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ServerResponse>, (StatusCode, Json<serde_json::Value>)> {
    let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", e)})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Server not found"})),
            )
        })?;

    Ok(Json(ServerResponse::from(server)))
}

pub async fn create_server(
    State(state): State<AppState>,
    Json(payload): Json<CreateServer>,
) -> Result<(StatusCode, Json<ServerResponse>), (StatusCode, Json<serde_json::Value>)> {
    let id = Uuid::new_v4().to_string();
    let labels_json = payload
        .labels
        .as_ref()
        .map(|l| serde_json::to_string(l).unwrap_or_else(|_| "[]".to_string()));

    sqlx::query(
        "INSERT INTO servers (id, name, host, port, ssh_user, ssh_key_path, labels, group_name, key_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&payload.name)
    .bind(&payload.host)
    .bind(payload.port.unwrap_or(22))
    .bind(payload.ssh_user.as_deref().unwrap_or("root"))
    .bind(&payload.ssh_key_path)
    .bind(&labels_json)
    .bind(&payload.group_name)
    .bind(&payload.key_id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
    })?;

    let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", e)})),
            )
        })?;

    Ok((StatusCode::CREATED, Json(ServerResponse::from(server))))
}

pub async fn update_server(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateServer>,
) -> Result<Json<ServerResponse>, (StatusCode, Json<serde_json::Value>)> {
    let existing = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", e)})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Server not found"})),
            )
        })?;

    let name = payload.name.unwrap_or(existing.name);
    let host = payload.host.unwrap_or(existing.host);
    let port = payload.port.unwrap_or(existing.port);
    let ssh_user = payload.ssh_user.unwrap_or(existing.ssh_user);
    let ssh_key_path = payload.ssh_key_path.or(existing.ssh_key_path);
    let labels_json = payload
        .labels
        .map(|l| serde_json::to_string(&l).unwrap_or_else(|_| "[]".to_string()))
        .or(existing.labels);
    let group_name = payload.group_name.or(existing.group_name);
    let key_id = payload.key_id.or(existing.key_id);

    sqlx::query(
        "UPDATE servers SET name = ?, host = ?, port = ?, ssh_user = ?, ssh_key_path = ?, labels = ?, group_name = ?, key_id = ? WHERE id = ?",
    )
    .bind(&name)
    .bind(&host)
    .bind(port)
    .bind(&ssh_user)
    .bind(&ssh_key_path)
    .bind(&labels_json)
    .bind(&group_name)
    .bind(&key_id)
    .bind(&id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
    })?;

    let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", e)})),
            )
        })?;

    Ok(Json(ServerResponse::from(server)))
}

pub async fn delete_server(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let result = sqlx::query("DELETE FROM servers WHERE id = ?")
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
            Json(json!({"error": "Server not found"})),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn health_check(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", e)})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Server not found"})),
            )
        })?;

    let status = crate::ssh::check_health(&server).await;
    let status_str = if status { "online" } else { "offline" };
    let now = chrono::Utc::now().naive_utc();

    sqlx::query("UPDATE servers SET status = ?, last_health_check = ? WHERE id = ?")
        .bind(status_str)
        .bind(now)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", e)})),
            )
        })?;

    Ok(Json(json!({
        "id": id,
        "status": status_str,
        "checked_at": now.to_string()
    })))
}

// ─── Server Groups ────────────────────────────────────────────────────────────

pub async fn list_groups(
    State(state): State<AppState>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, Json<serde_json::Value>)> {
    let groups: Vec<(Option<String>, i64, i64)> = sqlx::query_as(
        "SELECT group_name, COUNT(*) as count, SUM(CASE WHEN status = 'online' THEN 1 ELSE 0 END) as online FROM servers GROUP BY group_name ORDER BY group_name",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
    })?;

    let result: Vec<serde_json::Value> = groups
        .into_iter()
        .map(|(name, count, online)| {
            json!({
                "name": name.unwrap_or_else(|| "ungrouped".to_string()),
                "server_count": count,
                "online_count": online
            })
        })
        .collect();

    Ok(Json(result))
}

// ─── Bulk Operations ──────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
pub struct BulkServerIds {
    pub server_ids: Vec<String>,
}

pub async fn bulk_health_check(
    State(state): State<AppState>,
    Json(payload): Json<BulkServerIds>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, Json<serde_json::Value>)> {
    let mut results = Vec::new();

    for server_id in &payload.server_ids {
        let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
            .bind(server_id)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": format!("Database error: {}", e)})),
                )
            })?;

        if let Some(server) = server {
            let online = crate::ssh::check_health(&server).await;
            let status_str = if online { "online" } else { "offline" };
            let now = chrono::Utc::now().naive_utc();

            sqlx::query("UPDATE servers SET status = ?, last_health_check = ? WHERE id = ?")
                .bind(status_str)
                .bind(now)
                .bind(server_id)
                .execute(&state.db)
                .await
                .ok();

            results.push(json!({
                "id": server_id,
                "status": status_str,
                "checked_at": now.to_string()
            }));
        }
    }

    Ok(Json(results))
}
