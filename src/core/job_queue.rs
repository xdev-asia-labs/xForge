use anyhow::Result;
use tracing::{error, info};

use crate::core::executor;
use crate::core::inventory;
use crate::core::recipe;
use crate::db::models::Job;
use crate::AppState;

pub async fn execute_job(state: &AppState, job_id: &str, recipe_name: &str) -> Result<()> {
    info!("Starting job execution: {}", job_id);

    // Update status to running
    let now = chrono::Utc::now().naive_utc();
    sqlx::query("UPDATE jobs SET status = 'running', started_at = ? WHERE id = ?")
        .bind(now)
        .bind(job_id)
        .execute(&state.db)
        .await?;

    // Load job details
    let job = sqlx::query_as::<_, Job>("SELECT * FROM jobs WHERE id = ?")
        .bind(job_id)
        .fetch_one(&state.db)
        .await?;

    // Load recipe
    let recipes = recipe::load_recipes(&state.config.recipes_dir)?;
    let recipe = recipes
        .iter()
        .find(|r| r.name == recipe_name)
        .ok_or_else(|| anyhow::anyhow!("Recipe not found: {}", recipe_name))?;

    // Parse server IDs
    let server_ids: Vec<String> = serde_json::from_str(&job.server_ids)?;

    // Generate inventory file
    let inventory_path = format!("/tmp/xforge-inventory-{}.json", job_id);
    let inventory_content = inventory::generate_inventory(&state.db, &server_ids).await?;
    tokio::fs::write(&inventory_path, &inventory_content).await?;

    // Determine playbook path (support absolute paths from marketplace installs)
    let playbook_path = if recipe.playbook.starts_with('/') {
        recipe.playbook.clone()
    } else {
        format!(
            "{}/{}/{}",
            state.config.recipes_dir, recipe_name, recipe.playbook
        )
    };

    // Parse extra vars
    let extra_vars: Option<serde_json::Value> =
        job.params.as_deref().and_then(|p| serde_json::from_str(p).ok());

    // Execute playbook
    let result = executor::run_playbook(
        &playbook_path,
        &inventory_path,
        extra_vars.as_ref(),
        job_id,
        &state.log_broadcast,
    )
    .await;

    // Clean up inventory file
    let _ = tokio::fs::remove_file(&inventory_path).await;

    // Update job status
    let finished_at = chrono::Utc::now().naive_utc();
    match result {
        Ok(output) => {
            let status = if output.exit_code == 0 {
                "success"
            } else {
                "failed"
            };
            sqlx::query(
                "UPDATE jobs SET status = ?, output = ?, finished_at = ? WHERE id = ?",
            )
            .bind(status)
            .bind(&output.full_output)
            .bind(finished_at)
            .bind(job_id)
            .execute(&state.db)
            .await?;
            info!("Job {} completed with status: {}", job_id, status);
        }
        Err(e) => {
            error!("Job {} execution error: {}", job_id, e);
            sqlx::query(
                "UPDATE jobs SET status = 'failed', output = ?, finished_at = ? WHERE id = ?",
            )
            .bind(format!("Execution error: {}", e))
            .bind(finished_at)
            .bind(job_id)
            .execute(&state.db)
            .await?;
        }
    }

    Ok(())
}
