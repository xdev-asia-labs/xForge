use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use std::path::PathBuf;
use uuid::Uuid;

use crate::core::recipe::Recipe;
use crate::db::models::{
    AuditLog, Claims, CreateSource, RecipeSource, RecipeSourceWithRecipes, SourceRecipe,
    SourceRecipeResponse,
};
use crate::AppState;

// ─── List sources ─────────────────────────────────────────────────────────────

pub async fn list_sources(
    State(state): State<AppState>,
) -> Result<Json<Vec<RecipeSourceWithRecipes>>, (StatusCode, Json<serde_json::Value>)> {
    let sources = sqlx::query_as::<_, RecipeSource>(
        "SELECT * FROM recipe_sources ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
    })?;

    let mut result = Vec::new();
    for source in sources {
        let recipes = sqlx::query_as::<_, SourceRecipe>(
            "SELECT * FROM source_recipes WHERE source_id = ? ORDER BY name",
        )
        .bind(&source.id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

        result.push(RecipeSourceWithRecipes {
            source,
            recipes: recipes.into_iter().map(SourceRecipeResponse::from).collect(),
        });
    }

    Ok(Json(result))
}

// ─── Add a source ─────────────────────────────────────────────────────────────

pub async fn add_source(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateSource>,
) -> Result<(StatusCode, Json<RecipeSourceWithRecipes>), (StatusCode, Json<serde_json::Value>)> {
    // Derive name from URL (last path segment, strip .git)
    let name = payload
        .url
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("unknown")
        .trim_end_matches(".git")
        .to_string();

    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO recipe_sources (id, name, url, description, status) VALUES (?, ?, ?, ?, 'pending')",
    )
    .bind(&id)
    .bind(&name)
    .bind(&payload.url)
    .bind(&payload.description)
    .execute(&state.db)
    .await
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("UNIQUE") {
            (StatusCode::CONFLICT, Json(json!({"error": "Source URL already exists"})))
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Database error: {}", e)})))
        }
    })?;

    audit(&state, &claims.username, "source.add", "source", &id).await;

    let source = sqlx::query_as::<_, RecipeSource>("SELECT * FROM recipe_sources WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok((StatusCode::CREATED, Json(RecipeSourceWithRecipes { source, recipes: vec![] })))
}

// ─── Delete a source ──────────────────────────────────────────────────────────

pub async fn delete_source(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let source = sqlx::query_as::<_, RecipeSource>("SELECT * FROM recipe_sources WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(json!({"error": "Source not found"}))))?;

    // Remove cloned directory if it exists
    let clone_dir = PathBuf::from(&state.config.sources_dir).join(&id);
    if clone_dir.exists() {
        let _ = tokio::fs::remove_dir_all(&clone_dir).await;
    }

    sqlx::query("DELETE FROM recipe_sources WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    audit(&state, &claims.username, "source.delete", "source", &source.id).await;

    Ok(StatusCode::NO_CONTENT)
}

// ─── Sync (git clone / pull) ──────────────────────────────────────────────────

pub async fn sync_source(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<RecipeSourceWithRecipes>, (StatusCode, Json<serde_json::Value>)> {
    let source = sqlx::query_as::<_, RecipeSource>("SELECT * FROM recipe_sources WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(json!({"error": "Source not found"}))))?;

    // Mark as syncing
    sqlx::query("UPDATE recipe_sources SET status = 'syncing', sync_error = NULL WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .ok();

    let clone_dir = PathBuf::from(&state.config.sources_dir).join(&id);

    // Clone or pull
    let git_result = if clone_dir.exists() {
        run_git(&["pull", "--ff-only"], &clone_dir).await
    } else {
        tokio::fs::create_dir_all(&state.config.sources_dir).await.ok();
        // Pass just &id as the destination name; cwd is already sources_dir
        run_git(&["clone", "--depth=1", &source.url, &id], &PathBuf::from(&state.config.sources_dir)).await
    };

    match git_result {
        Err(e) => {
            let err_msg = e.to_string();
            sqlx::query("UPDATE recipe_sources SET status = 'error', sync_error = ? WHERE id = ?")
                .bind(&err_msg)
                .bind(&id)
                .execute(&state.db)
                .await
                .ok();
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": err_msg}))));
        }
        Ok(output) if !output.status.success() => {
            let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
            sqlx::query("UPDATE recipe_sources SET status = 'error', sync_error = ? WHERE id = ?")
                .bind(&err_msg)
                .bind(&id)
                .execute(&state.db)
                .await
                .ok();
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": err_msg}))));
        }
        _ => {}
    }

    // Scan discovered recipes
    let discovered = discover_recipes(&clone_dir, &source.name);

    // Clear old discovered recipes and insert new ones
    // Remove any recipe whose slug is no longer in the discovery set (handles renames)
    let new_slugs: Vec<&str> = discovered.iter().map(|r| r.slug.as_str()).collect();
    let existing_slugs: Vec<(String,)> = sqlx::query_as(
        "SELECT slug FROM source_recipes WHERE source_id = ?",
    )
    .bind(&id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    for (slug,) in &existing_slugs {
        if !new_slugs.contains(&slug.as_str()) {
            sqlx::query("DELETE FROM source_recipes WHERE source_id = ? AND slug = ?")
                .bind(&id)
                .bind(slug)
                .execute(&state.db)
                .await
                .ok();
        }
    }

    for rec in &discovered {
        let recipe_id = Uuid::new_v4().to_string();
        // Check if already installed (keep its installed flag)
        let existing: Option<(i64,)> = sqlx::query_as(
            "SELECT installed FROM source_recipes WHERE source_id = ? AND slug = ?",
        )
        .bind(&id)
        .bind(&rec.slug)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

        if existing.is_some() {
            // Update existing entry but keep installed flag
            sqlx::query(
                "UPDATE source_recipes SET name = ?, description = ?, playbook = ?, version = ?, tags = ? WHERE source_id = ? AND slug = ?",
            )
            .bind(&rec.name)
            .bind(&rec.description)
            .bind(&rec.playbook)
            .bind(&rec.version)
            .bind(&rec.tags)
            .bind(&id)
            .bind(&rec.slug)
            .execute(&state.db)
            .await
            .ok();
        } else {
            sqlx::query(
                "INSERT OR IGNORE INTO source_recipes (id, source_id, slug, name, description, playbook, version, tags, installed) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0)",
            )
            .bind(&recipe_id)
            .bind(&id)
            .bind(&rec.slug)
            .bind(&rec.name)
            .bind(&rec.description)
            .bind(&rec.playbook)
            .bind(&rec.version)
            .bind(&rec.tags)
            .execute(&state.db)
            .await
            .ok();
        }
    }

    // Update status
    let now = chrono::Utc::now().naive_utc();
    sqlx::query("UPDATE recipe_sources SET status = 'synced', last_synced_at = ? WHERE id = ?")
        .bind(now)
        .bind(&id)
        .execute(&state.db)
        .await
        .ok();

    audit(&state, &claims.username, "source.sync", "source", &id).await;

    // Return updated source
    fetch_source_with_recipes(&state, &id).await
}

// ─── Install a recipe from a source ──────────────────────────────────────────

pub async fn install_recipe(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path((source_id, recipe_slug)): Path<(String, String)>,
) -> Result<Json<SourceRecipeResponse>, (StatusCode, Json<serde_json::Value>)> {
    let source_recipe = sqlx::query_as::<_, SourceRecipe>(
        "SELECT * FROM source_recipes WHERE source_id = ? AND slug = ?",
    )
    .bind(&source_id)
    .bind(&recipe_slug)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, Json(json!({"error": "Recipe not found in source"}))))?;

    let clone_dir = PathBuf::from(&state.config.sources_dir).join(&source_id);
    if !clone_dir.exists() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Source not synced yet — sync the source first"})),
        ));
    }

    // The playbook path in the clone
    let playbook_abs = clone_dir.join(&source_recipe.playbook);
    if !playbook_abs.exists() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Playbook not found: {}", source_recipe.playbook)})),
        ));
    }

    // Create a recipe directory in recipes_dir
    let recipe_dir = PathBuf::from(&state.config.recipes_dir).join(&recipe_slug);

    // Generate recipe.yaml pointing to the source playbook with absolute path
    let tags: Vec<String> = serde_json::from_str(&source_recipe.tags).unwrap_or_default();
    let recipe = Recipe {
        name: source_recipe.name.clone(),
        version: source_recipe.version.clone(),
        description: source_recipe.description.clone().unwrap_or_default(),
        params: None,
        requires: None,
        playbook: tokio::fs::canonicalize(&playbook_abs).await
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| playbook_abs.to_string_lossy().to_string()),
        tags: if tags.is_empty() { None } else { Some(tags) },
    };

    tokio::fs::create_dir_all(&recipe_dir).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to create recipe dir: {}", e)})))
    })?;

    let yaml = serde_yaml::to_string(&recipe).map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Serialization error: {}", e)})))
    })?;

    tokio::fs::write(recipe_dir.join("recipe.yaml"), yaml).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to write recipe.yaml: {}", e)})))
    })?;

    // Mark as installed
    sqlx::query("UPDATE source_recipes SET installed = 1 WHERE source_id = ? AND slug = ?")
        .bind(&source_id)
        .bind(&recipe_slug)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    audit(&state, &claims.username, "source.install_recipe", "source_recipe", &source_recipe.id).await;

    let updated = sqlx::query_as::<_, SourceRecipe>(
        "SELECT * FROM source_recipes WHERE source_id = ? AND slug = ?",
    )
    .bind(&source_id)
    .bind(&recipe_slug)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(SourceRecipeResponse::from(updated)))
}

// ─── Audit log ────────────────────────────────────────────────────────────────

pub async fn list_audit_logs(
    State(state): State<AppState>,
) -> Result<Json<Vec<AuditLog>>, (StatusCode, Json<serde_json::Value>)> {
    let logs = sqlx::query_as::<_, AuditLog>(
        "SELECT * FROM audit_logs ORDER BY created_at DESC LIMIT 200",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
    })?;

    Ok(Json(logs))
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

async fn run_git(args: &[&str], cwd: &PathBuf) -> std::io::Result<std::process::Output> {
    tokio::process::Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .await
}

struct DiscoveredRecipe {
    slug: String,
    name: String,
    description: Option<String>,
    playbook: String,
    version: String,
    tags: String,
}

fn discover_recipes(clone_dir: &PathBuf, source_name: &str) -> Vec<DiscoveredRecipe> {
    let mut found = Vec::new();

    // 1. Look for recipe.yaml files in subdirectories (xForge native format)
    if let Ok(entries) = std::fs::read_dir(clone_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let recipe_file = path.join("recipe.yaml");
                if recipe_file.exists() {
                    if let Ok(content) = std::fs::read_to_string(&recipe_file) {
                        if let Ok(recipe) = serde_yaml::from_str::<Recipe>(&content) {
                            let playbook_abs = path.join(&recipe.playbook);
                            let playbook_rel = playbook_abs
                                .strip_prefix(clone_dir)
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or_else(|_| recipe.playbook.clone());

                            found.push(DiscoveredRecipe {
                                slug: recipe.name.clone(),
                                name: recipe.name,
                                description: Some(recipe.description),
                                playbook: playbook_rel,
                                version: recipe.version,
                                tags: serde_json::to_string(&recipe.tags.unwrap_or_default())
                                    .unwrap_or_else(|_| "[]".to_string()),
                            });
                        }
                    }
                }
            }
        }
    }

    // 2. Check for recipe.yaml at root
    let root_recipe = clone_dir.join("recipe.yaml");
    if root_recipe.exists() {
        if let Ok(content) = std::fs::read_to_string(&root_recipe) {
            if let Ok(recipe) = serde_yaml::from_str::<Recipe>(&content) {
                found.push(DiscoveredRecipe {
                    slug: recipe.name.clone(),
                    name: recipe.name,
                    description: Some(recipe.description),
                    playbook: recipe.playbook,
                    version: recipe.version,
                    tags: serde_json::to_string(&recipe.tags.unwrap_or_default())
                        .unwrap_or_else(|_| "[]".to_string()),
                });
            }
        }
    }

    // 3. Auto-detect common Ansible entry points if no recipe.yaml found
    if found.is_empty() {
        let repo_name = source_name;

        let candidates = [
            ("site.yml", "site"),
            ("main.yml", "main"),
            ("playbooks/site.yml", "site"),
            ("playbooks/main.yml", "main"),
        ];

        for (rel_path, suffix) in candidates {
            let pb = clone_dir.join(rel_path);
            if pb.exists() {
                let slug = format!("{}-{}", repo_name, suffix);
                let display_name = repo_name
                    .replace('-', " ")
                    .replace('_', " ")
                    .split_whitespace()
                    .map(|w| {
                        let mut c = w.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");

                found.push(DiscoveredRecipe {
                    slug,
                    name: format!("{} (auto)", display_name),
                    description: Some(format!("Auto-detected from {}", rel_path)),
                    playbook: rel_path.to_string(),
                    version: "auto".to_string(),
                    tags: "[\"auto-detected\"]".to_string(),
                });
                break; // only first match per auto-detect
            }
        }

        // Also scan playbooks/*.yml for named entries
        let playbooks_dir = clone_dir.join("playbooks");
        if playbooks_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&playbooks_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("yml")
                        || path.extension().and_then(|e| e.to_str()) == Some("yaml")
                    {
                        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");
                        // Skip site/main (already covered above)
                        if stem == "site" || stem == "main" {
                            continue;
                        }
                        let slug = format!("{}-{}", repo_name, stem);
                        let rel = format!("playbooks/{}", path.file_name().unwrap().to_str().unwrap());
                        found.push(DiscoveredRecipe {
                            slug,
                            name: format!("{} — {}", repo_name.replace('-', " "), stem.replace('-', " ")),
                            description: Some(format!("Playbook: {}", rel)),
                            playbook: rel,
                            version: "auto".to_string(),
                            tags: "[\"auto-detected\"]".to_string(),
                        });
                    }
                }
            }
        }
    }

    found
}

async fn fetch_source_with_recipes(
    state: &AppState,
    source_id: &str,
) -> Result<Json<RecipeSourceWithRecipes>, (StatusCode, Json<serde_json::Value>)> {
    let source =
        sqlx::query_as::<_, RecipeSource>("SELECT * FROM recipe_sources WHERE id = ?")
            .bind(source_id)
            .fetch_one(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let recipes = sqlx::query_as::<_, SourceRecipe>(
        "SELECT * FROM source_recipes WHERE source_id = ? ORDER BY name",
    )
    .bind(source_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Ok(Json(RecipeSourceWithRecipes {
        source,
        recipes: recipes.into_iter().map(SourceRecipeResponse::from).collect(),
    }))
}

async fn audit(state: &AppState, username: &str, action: &str, resource_type: &str, resource_id: &str) {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO audit_logs (id, username, action, resource_type, resource_id) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(username)
    .bind(action)
    .bind(resource_type)
    .bind(resource_id)
    .execute(&state.db)
    .await
    .ok();
}
