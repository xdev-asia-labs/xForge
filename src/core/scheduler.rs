use anyhow::Result;
use cron::Schedule;
use std::str::FromStr;
use tokio::time::{interval, Duration};
use tracing::{error, info};
use uuid::Uuid;

use crate::db::models::Schedule as DbSchedule;
use crate::AppState;

pub async fn start_scheduler(state: AppState) {
    info!("Scheduler started — checking every 30 seconds");
    let mut tick = interval(Duration::from_secs(30));

    loop {
        tick.tick().await;
        if let Err(e) = check_schedules(&state).await {
            error!("Scheduler error: {}", e);
        }
    }
}

async fn check_schedules(state: &AppState) -> Result<()> {
    let schedules = sqlx::query_as::<_, DbSchedule>(
        "SELECT * FROM schedules WHERE enabled = 1",
    )
    .fetch_all(&state.db)
    .await?;

    let now = chrono::Utc::now();

    for schedule in schedules {
        let should_run = if let Some(next_run) = schedule.next_run_at {
            let next_utc =
                chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(next_run, chrono::Utc);
            next_utc <= now
        } else {
            // No next_run_at set — calculate and skip this cycle
            let next = calculate_next_run(&schedule.cron_expression);
            if let Some(nr) = next {
                sqlx::query("UPDATE schedules SET next_run_at = ? WHERE id = ?")
                    .bind(nr)
                    .bind(&schedule.id)
                    .execute(&state.db)
                    .await
                    .ok();
            }
            false
        };

        if should_run {
            info!("Scheduler triggering: {} ({})", schedule.name, schedule.recipe_name);

            // Create a job
            let job_id = Uuid::new_v4().to_string();
            let created_by = schedule.created_by.as_deref().unwrap_or("scheduler");

            sqlx::query(
                "INSERT INTO jobs (id, recipe_name, server_ids, params, status, created_by) VALUES (?, ?, ?, ?, 'pending', ?)",
            )
            .bind(&job_id)
            .bind(&schedule.recipe_name)
            .bind(&schedule.server_ids)
            .bind(&schedule.params)
            .bind(created_by)
            .execute(&state.db)
            .await?;

            // Spawn job execution
            let job_state = state.clone();
            let jid = job_id.clone();
            let recipe = schedule.recipe_name.clone();
            tokio::spawn(async move {
                if let Err(e) =
                    crate::core::job_queue::execute_job(&job_state, &jid, &recipe).await
                {
                    error!("Scheduled job {} failed: {}", jid, e);
                }
            });

            // Update schedule times
            let next = calculate_next_run(&schedule.cron_expression);
            sqlx::query(
                "UPDATE schedules SET last_run_at = ?, next_run_at = ? WHERE id = ?",
            )
            .bind(now.naive_utc())
            .bind(next)
            .bind(&schedule.id)
            .execute(&state.db)
            .await?;
        }
    }

    Ok(())
}

fn calculate_next_run(cron_expr: &str) -> Option<chrono::NaiveDateTime> {
    let full_expr = format!("0 {}", cron_expr.trim());
    if let Ok(schedule) = Schedule::from_str(&full_expr) {
        schedule
            .upcoming(chrono::Utc)
            .next()
            .map(|dt| dt.naive_utc())
    } else {
        None
    }
}
