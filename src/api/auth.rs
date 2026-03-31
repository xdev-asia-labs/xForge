use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde_json::json;

use crate::db::models::{Claims, LoginRequest, LoginResponse, UserInfo};
use crate::AppState;

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user = sqlx::query_as::<_, crate::db::models::User>(
        "SELECT * FROM users WHERE username = ?",
    )
    .bind(&payload.username)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Database error"})),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid credentials"})),
        )
    })?;

    let valid = bcrypt::verify(&payload.password, &user.password_hash).unwrap_or(false);
    if !valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid credentials"})),
        ));
    }

    let claims = Claims {
        sub: user.id.clone(),
        username: user.username.clone(),
        role: user.role.clone(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Token generation failed"})),
        )
    })?;

    Ok(Json(LoginResponse {
        token,
        user: UserInfo {
            id: user.id,
            username: user.username,
            role: user.role,
        },
    }))
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !auth_header.starts_with("Bearer ") {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Missing authorization header"})),
        )
            .into_response();
    }

    let token = &auth_header[7..];

    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(token_data) => {
            request.extensions_mut().insert(token_data.claims);
            next.run(request).await
        }
        Err(_) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid token"})),
        )
            .into_response(),
    }
}
