use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::db::models::{Claims, CreateUser, UpdateUser, User, UserResponse};
use crate::AppState;

fn require_admin(claims: &Claims) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    if claims.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Admin access required"})),
        ));
    }
    Ok(())
}

pub async fn list_users(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<UserResponse>>, (StatusCode, Json<serde_json::Value>)> {
    require_admin(&claims)?;

    let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", e)})),
            )
        })?;

    Ok(Json(users.into_iter().map(UserResponse::from).collect()))
}

pub async fn create_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateUser>,
) -> Result<(StatusCode, Json<UserResponse>), (StatusCode, Json<serde_json::Value>)> {
    require_admin(&claims)?;

    if payload.username.trim().is_empty() || payload.password.len() < 4 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Username required and password must be at least 4 characters"})),
        ));
    }

    let role = payload.role.as_deref().unwrap_or("operator");
    if role != "admin" && role != "operator" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Role must be 'admin' or 'operator'"})),
        ));
    }

    let id = Uuid::new_v4().to_string();
    let hash = bcrypt::hash(&payload.password, bcrypt::DEFAULT_COST).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to hash password"})),
        )
    })?;

    sqlx::query(
        "INSERT INTO users (id, username, password_hash, role, email, display_name) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(payload.username.trim())
    .bind(&hash)
    .bind(role)
    .bind(&payload.email)
    .bind(&payload.display_name)
    .execute(&state.db)
    .await
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("UNIQUE") {
            (StatusCode::CONFLICT, Json(json!({"error": "Username already exists"})))
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Database error: {}", e)})))
        }
    })?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", e)})),
            )
        })?;

    Ok((StatusCode::CREATED, Json(UserResponse::from(user))))
}

pub async fn update_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateUser>,
) -> Result<Json<UserResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Users can update their own profile, admins can update anyone
    if claims.sub != id {
        require_admin(&claims)?;
    }

    let existing = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
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
                Json(json!({"error": "User not found"})),
            )
        })?;

    // Only admin can change roles
    let role = if let Some(ref new_role) = payload.role {
        require_admin(&claims)?;
        if new_role != "admin" && new_role != "operator" {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Role must be 'admin' or 'operator'"})),
            ));
        }
        new_role.clone()
    } else {
        existing.role
    };

    let password_hash = if let Some(ref new_password) = payload.password {
        if new_password.len() < 4 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Password must be at least 4 characters"})),
            ));
        }
        bcrypt::hash(new_password, bcrypt::DEFAULT_COST).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to hash password"})),
            )
        })?
    } else {
        existing.password_hash
    };

    let email = payload.email.or(existing.email);
    let display_name = payload.display_name.or(existing.display_name);

    sqlx::query(
        "UPDATE users SET password_hash = ?, role = ?, email = ?, display_name = ? WHERE id = ?",
    )
    .bind(&password_hash)
    .bind(&role)
    .bind(&email)
    .bind(&display_name)
    .bind(&id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
    })?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", e)})),
            )
        })?;

    Ok(Json(UserResponse::from(user)))
}

pub async fn delete_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    require_admin(&claims)?;

    // Prevent deleting yourself
    if claims.sub == id {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Cannot delete your own account"})),
        ));
    }

    let result = sqlx::query("DELETE FROM users WHERE id = ?")
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
            Json(json!({"error": "User not found"})),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_current_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<UserResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(&claims.sub)
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
                Json(json!({"error": "User not found"})),
            )
        })?;

    Ok(Json(UserResponse::from(user)))
}
