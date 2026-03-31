use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::db::models::{Claims, KeyStoreEntry, KeyStoreResponse, CreateKeyStoreEntry};
use crate::AppState;

pub async fn list_keys(
    State(state): State<AppState>,
) -> Result<Json<Vec<KeyStoreResponse>>, (StatusCode, Json<serde_json::Value>)> {
    let keys = sqlx::query_as::<_, KeyStoreEntry>(
        "SELECT * FROM key_store ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
    })?;

    Ok(Json(keys.into_iter().map(KeyStoreResponse::from).collect()))
}

pub async fn create_key(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateKeyStoreEntry>,
) -> Result<(StatusCode, Json<KeyStoreResponse>), (StatusCode, Json<serde_json::Value>)> {
    if payload.name.trim().is_empty() || payload.key_data.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Name and key data are required"})),
        ));
    }

    let valid_types = ["ssh_key", "login_password", "token"];
    if !valid_types.contains(&payload.key_type.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "key_type must be 'ssh_key', 'login_password', or 'token'"})),
        ));
    }

    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO key_store (id, name, key_type, key_data, description, created_by) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(payload.name.trim())
    .bind(&payload.key_type)
    .bind(&payload.key_data)
    .bind(&payload.description)
    .bind(&claims.sub)
    .execute(&state.db)
    .await
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("UNIQUE") {
            (StatusCode::CONFLICT, Json(json!({"error": "Key name already exists"})))
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Database error: {}", e)})))
        }
    })?;

    let key = sqlx::query_as::<_, KeyStoreEntry>("SELECT * FROM key_store WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", e)})),
            )
        })?;

    Ok((StatusCode::CREATED, Json(KeyStoreResponse::from(key))))
}

pub async fn delete_key(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let result = sqlx::query("DELETE FROM key_store WHERE id = ?")
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
            Json(json!({"error": "Key not found"})),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}
