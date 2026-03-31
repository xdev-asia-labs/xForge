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
        "INSERT INTO servers (id, name, host, port, ssh_user, ssh_key_path, labels, group_name) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&payload.name)
    .bind(&payload.host)
    .bind(payload.port.unwrap_or(22))
    .bind(payload.ssh_user.as_deref().unwrap_or("root"))
    .bind(&payload.ssh_key_path)
    .bind(&labels_json)
    .bind(&payload.group_name)
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

    sqlx::query(
        "UPDATE servers SET name = ?, host = ?, port = ?, ssh_user = ?, ssh_key_path = ?, labels = ?, group_name = ? WHERE id = ?",
    )
    .bind(&name)
    .bind(&host)
    .bind(port)
    .bind(&ssh_user)
    .bind(&ssh_key_path)
    .bind(&labels_json)
    .bind(&group_name)
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
