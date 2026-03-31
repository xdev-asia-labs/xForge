use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use cron::Schedule as CronSchedule;
use serde_json::json;
use std::str::FromStr;
use uuid::Uuid;

use crate::db::models::{
    Claims, CreateSchedule, Schedule, ScheduleResponse, UpdateSchedule,
};
use crate::AppState;

fn parse_cron(expr: &str) -> Result<CronSchedule, (StatusCode, Json<serde_json::Value>)> {
    // User provides 5-field cron (min hour dom month dow)
    // The cron crate expects 6-7 fields (sec min hour dom month dow [year])
    let full_expr = format!("0 {}", expr.trim());
    CronSchedule::from_str(&full_expr).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Invalid cron expression: {}", e)})),
        )
    })
}

fn calculate_next_run(cron_expr: &str) -> Option<chrono::NaiveDateTime> {
    let full_expr = format!("0 {}", cron_expr.trim());
    if let Ok(schedule) = CronSchedule::from_str(&full_expr) {
        schedule
            .upcoming(chrono::Utc)
            .next()
            .map(|dt| dt.naive_utc())
    } else {
        None
    }
}

pub async fn list_schedules(
    State(state): State<AppState>,
) -> Result<Json<Vec<ScheduleResponse>>, (StatusCode, Json<serde_json::Value>)> {
    let schedules = sqlx::query_as::<_, Schedule>(
        "SELECT * FROM schedules ORDER BY created_at DESC",
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
        schedules
            .into_iter()
            .map(ScheduleResponse::from)
            .collect(),
    ))
}

pub async fn create_schedule(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateSchedule>,
) -> Result<(StatusCode, Json<ScheduleResponse>), (StatusCode, Json<serde_json::Value>)> {
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Schedule name is required"})),
        ));
    }

    // Validate cron expression
    let _cron = parse_cron(&payload.cron_expression)?;

    // Validate recipe exists
    let recipes =
        crate::core::recipe::load_recipes(&state.config.recipes_dir).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to load recipes: {}", e)})),
            )
        })?;
    if !recipes.iter().any(|r| r.name == payload.recipe_name) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Recipe not found"})),
        ));
    }

    if payload.server_ids.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "At least one server is required"})),
        ));
    }

    let id = Uuid::new_v4().to_string();
    let server_ids_json = serde_json::to_string(&payload.server_ids).unwrap();
    let params_json = payload
        .params
        .as_ref()
        .map(|p| serde_json::to_string(p).unwrap());
    let next_run = calculate_next_run(&payload.cron_expression);

    sqlx::query(
        "INSERT INTO schedules (id, name, recipe_name, server_ids, params, cron_expression, next_run_at, created_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(payload.name.trim())
    .bind(&payload.recipe_name)
    .bind(&server_ids_json)
    .bind(&params_json)
    .bind(&payload.cron_expression)
    .bind(next_run)
    .bind(&claims.sub)
    .execute(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
    })?;

    let schedule = sqlx::query_as::<_, Schedule>("SELECT * FROM schedules WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", e)})),
            )
        })?;

    Ok((StatusCode::CREATED, Json(ScheduleResponse::from(schedule))))
}

pub async fn update_schedule(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateSchedule>,
) -> Result<Json<ScheduleResponse>, (StatusCode, Json<serde_json::Value>)> {
    let existing = sqlx::query_as::<_, Schedule>("SELECT * FROM schedules WHERE id = ?")
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
                Json(json!({"error": "Schedule not found"})),
            )
        })?;

    let name = payload.name.unwrap_or(existing.name);
    let cron_expression = if let Some(ref expr) = payload.cron_expression {
        let _cron = parse_cron(expr)?;
        expr.clone()
    } else {
        existing.cron_expression
    };
    let server_ids = payload
        .server_ids
        .map(|s| serde_json::to_string(&s).unwrap())
        .unwrap_or(existing.server_ids);
    let params = payload
        .params
        .map(|p| serde_json::to_string(&p).unwrap())
        .or(existing.params);
    let enabled = payload.enabled.map(|e| e as i64).unwrap_or(existing.enabled);
    let next_run = calculate_next_run(&cron_expression);

    sqlx::query(
        "UPDATE schedules SET name = ?, cron_expression = ?, server_ids = ?, params = ?, enabled = ?, next_run_at = ? WHERE id = ?",
    )
    .bind(&name)
    .bind(&cron_expression)
    .bind(&server_ids)
    .bind(&params)
    .bind(enabled)
    .bind(next_run)
    .bind(&id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
    })?;

    let schedule = sqlx::query_as::<_, Schedule>("SELECT * FROM schedules WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", e)})),
            )
        })?;

    Ok(Json(ScheduleResponse::from(schedule)))
}

pub async fn delete_schedule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let result = sqlx::query("DELETE FROM schedules WHERE id = ?")
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
            Json(json!({"error": "Schedule not found"})),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}
