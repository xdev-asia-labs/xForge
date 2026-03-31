---
description: "Use when writing Rust API handlers, adding new endpoints, or modifying existing route handlers in the Axum backend."
applyTo: "src/api/**/*.rs"
---
# Rust API Handler Patterns

## Handler Signature
Return `Result<Json<T>, (StatusCode, Json<serde_json::Value>)>` for all handlers:
```rust
pub async fn handler(
    State(state): State<AppState>,
    Json(payload): Json<RequestType>,
) -> Result<Json<ResponseType>, (StatusCode, Json<serde_json::Value>)> {
```

For creation endpoints, return tuple with status:
```rust
) -> Result<(StatusCode, Json<ResponseType>), (StatusCode, Json<serde_json::Value>)> {
    // ...
    Ok((StatusCode::CREATED, Json(response)))
}
```

## Error Mapping
Always use `.map_err()` on database calls with descriptive messages:
```rust
.map_err(|e| (
    StatusCode::INTERNAL_SERVER_ERROR,
    Json(json!({"error": format!("Database error: {}", e)})),
))?;
```

Use `.ok_or_else()` for not-found:
```rust
.ok_or_else(|| (
    StatusCode::NOT_FOUND,
    Json(json!({"error": "Resource not found"})),
))?;
```

## Auth-Protected Handlers
Extract claims via `Extension`:
```rust
use axum::extract::Extension;
use crate::db::models::Claims;

pub async fn handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    // ...
```

## Database Queries
Use `sqlx::query_as::<_, Model>()` with `?` bind parameters (SQLite).
