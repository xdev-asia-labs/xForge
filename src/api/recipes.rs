use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json;

use crate::core::recipe::Recipe;
use crate::AppState;

pub async fn list_recipes(
    State(state): State<AppState>,
) -> Result<Json<Vec<Recipe>>, (StatusCode, Json<serde_json::Value>)> {
    let recipes = crate::core::recipe::load_recipes(&state.config.recipes_dir).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to load recipes: {}", e)})),
        )
    })?;

    Ok(Json(recipes))
}

pub async fn get_recipe(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Recipe>, (StatusCode, Json<serde_json::Value>)> {
    let recipes = crate::core::recipe::load_recipes(&state.config.recipes_dir).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to load recipes: {}", e)})),
        )
    })?;

    let recipe = recipes
        .into_iter()
        .find(|r| r.name == name)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Recipe not found"})),
            )
        })?;

    Ok(Json(recipe))
}
