use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::db::models::{Claims, CreateJob, DashboardStats, Job, JobResponse};
use crate::AppState;

pub async fn list_jobs(
    State(state): State<AppState>,
) -> Result<Json<Vec<JobResponse>>, (StatusCode, Json<serde_json::Value>)> {
    let jobs = sqlx::query_as::<_, Job>("SELECT * FROM jobs ORDER BY created_at DESC")
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", e)})),
            )
        })?;

    Ok(Json(jobs.into_iter().map(JobResponse::from).collect()))
}

pub async fn get_job(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<JobResponse>, (StatusCode, Json<serde_json::Value>)> {
    let job = sqlx::query_as::<_, Job>("SELECT * FROM jobs WHERE id = ?")
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
                Json(json!({"error": "Job not found"})),
            )
        })?;

    Ok(Json(JobResponse::from(job)))
}

pub async fn create_job(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateJob>,
) -> Result<(StatusCode, Json<JobResponse>), (StatusCode, Json<serde_json::Value>)> {
    // Validate recipe exists
    let recipes = crate::core::recipe::load_recipes(&state.config.recipes_dir).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to load recipes: {}", e)})),
        )
    })?;

    let recipe = recipes
        .iter()
        .find(|r| r.name == payload.recipe_name)
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Recipe not found"})),
            )
        })?;

    // Validate servers exist
    for server_id in &payload.server_ids {
        sqlx::query("SELECT id FROM servers WHERE id = ?")
            .bind(server_id)
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
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": format!("Server {} not found", server_id)})),
                )
            })?;
    }

    // Validate min_servers
    if let Some(min) = recipe.requires.as_ref().and_then(|r| r.min_servers) {
        if payload.server_ids.len() < min as usize {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": format!(
                    "Recipe requires at least {} servers, got {}",
                    min,
                    payload.server_ids.len()
                )})),
            ));
        }
    }

    let id = Uuid::new_v4().to_string();
    let server_ids_json = serde_json::to_string(&payload.server_ids).unwrap();
    let params_json = payload
        .params
        .as_ref()
        .map(|p| serde_json::to_string(p).unwrap());

    sqlx::query(
        "INSERT INTO jobs (id, recipe_name, server_ids, params, status, created_by) VALUES (?, ?, ?, ?, 'pending', ?)",
    )
    .bind(&id)
    .bind(&payload.recipe_name)
    .bind(&server_ids_json)
    .bind(&params_json)
    .bind(&claims.sub)
    .execute(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
    })?;

    // Spawn job execution in background
    let job_state = state.clone();
    let job_id = id.clone();
    let recipe_name = payload.recipe_name.clone();
    tokio::spawn(async move {
        if let Err(e) = crate::core::job_queue::execute_job(&job_state, &job_id, &recipe_name).await
        {
            tracing::error!("Job {} failed: {}", job_id, e);
        }
    });

    let job = sqlx::query_as::<_, Job>("SELECT * FROM jobs WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", e)})),
            )
        })?;

    Ok((StatusCode::CREATED, Json(JobResponse::from(job))))
}

pub async fn cancel_job(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<JobResponse>, (StatusCode, Json<serde_json::Value>)> {
    let job = sqlx::query_as::<_, Job>("SELECT * FROM jobs WHERE id = ?")
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
                Json(json!({"error": "Job not found"})),
            )
        })?;

    if job.status != "pending" && job.status != "running" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Job cannot be cancelled in its current state"})),
        ));
    }

    let now = chrono::Utc::now().naive_utc();
    sqlx::query("UPDATE jobs SET status = 'cancelled', finished_at = ? WHERE id = ?")
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

    let job = sqlx::query_as::<_, Job>("SELECT * FROM jobs WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", e)})),
            )
        })?;

    Ok(Json(JobResponse::from(job)))
}

pub async fn dashboard(
    State(state): State<AppState>,
) -> Result<Json<DashboardStats>, (StatusCode, Json<serde_json::Value>)> {
    let db_err = |e: sqlx::Error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
    };

    let server_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM servers")
            .fetch_one(&state.db)
            .await
            .map_err(db_err)?;

    let servers_online: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM servers WHERE status = 'online'")
            .fetch_one(&state.db)
            .await
            .map_err(db_err)?;

    let servers_offline: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM servers WHERE status = 'offline'")
            .fetch_one(&state.db)
            .await
            .map_err(db_err)?;

    let active_jobs: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM jobs WHERE status IN ('pending', 'running')",
    )
    .fetch_one(&state.db)
    .await
    .map_err(db_err)?;

    let total_jobs: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM jobs")
            .fetch_one(&state.db)
            .await
            .map_err(db_err)?;

    let successful_jobs: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM jobs WHERE status = 'success'")
            .fetch_one(&state.db)
            .await
            .map_err(db_err)?;

    let failed_jobs: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM jobs WHERE status = 'failed'")
            .fetch_one(&state.db)
            .await
            .map_err(db_err)?;

    let active_schedules: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM schedules WHERE enabled = 1")
            .fetch_one(&state.db)
            .await
            .map_err(db_err)?;

    let recent_jobs = sqlx::query_as::<_, Job>(
        "SELECT * FROM jobs ORDER BY created_at DESC LIMIT 10",
    )
    .fetch_all(&state.db)
    .await
    .map_err(db_err)?;

    Ok(Json(DashboardStats {
        server_count: server_count.0,
        servers_online: servers_online.0,
        servers_offline: servers_offline.0,
        active_jobs: active_jobs.0,
        total_jobs: total_jobs.0,
        successful_jobs: successful_jobs.0,
        failed_jobs: failed_jobs.0,
        recent_jobs: recent_jobs.into_iter().map(JobResponse::from).collect(),
        active_schedules: active_schedules.0,
    }))
}

// ─── Re-run a job ─────────────────────────────────────────────────────────────

pub async fn rerun_job(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<JobResponse>), (StatusCode, Json<serde_json::Value>)> {
    let original = sqlx::query_as::<_, Job>("SELECT * FROM jobs WHERE id = ?")
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
                Json(json!({"error": "Job not found"})),
            )
        })?;

    // Create new job with same params
    let new_id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO jobs (id, recipe_name, server_ids, params, status, created_by) VALUES (?, ?, ?, ?, 'pending', ?)",
    )
    .bind(&new_id)
    .bind(&original.recipe_name)
    .bind(&original.server_ids)
    .bind(&original.params)
    .bind(&claims.sub)
    .execute(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
    })?;

    // Spawn execution
    let job_state = state.clone();
    let job_id = new_id.clone();
    let recipe_name = original.recipe_name.clone();
    tokio::spawn(async move {
        if let Err(e) = crate::core::job_queue::execute_job(&job_state, &job_id, &recipe_name).await
        {
            tracing::error!("Re-run job {} failed: {}", job_id, e);
        }
    });

    let job = sqlx::query_as::<_, Job>("SELECT * FROM jobs WHERE id = ?")
        .bind(&new_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", e)})),
            )
        })?;

    Ok((StatusCode::CREATED, Json(JobResponse::from(job))))
}
