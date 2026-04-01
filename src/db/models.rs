use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub role: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        UserResponse {
            id: u.id,
            username: u.username,
            role: u.role,
            email: u.email,
            display_name: u.display_name,
            created_at: u.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
    pub role: Option<String>,
    pub email: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUser {
    pub password: Option<String>,
    pub role: Option<String>,
    pub email: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Server {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: i64,
    pub ssh_user: String,
    pub ssh_key_path: Option<String>,
    pub labels: Option<String>,
    pub group_name: Option<String>,
    pub status: String,
    pub last_health_check: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
    pub key_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerResponse {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: i64,
    pub ssh_user: String,
    pub ssh_key_path: Option<String>,
    pub labels: Vec<String>,
    pub group_name: Option<String>,
    pub status: String,
    pub last_health_check: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
    pub key_id: Option<String>,
}

impl From<Server> for ServerResponse {
    fn from(s: Server) -> Self {
        let labels: Vec<String> = s
            .labels
            .as_deref()
            .and_then(|l| serde_json::from_str(l).ok())
            .unwrap_or_default();
        ServerResponse {
            id: s.id,
            name: s.name,
            host: s.host,
            port: s.port,
            ssh_user: s.ssh_user,
            ssh_key_path: s.ssh_key_path,
            labels,
            group_name: s.group_name,
            status: s.status,
            last_health_check: s.last_health_check,
            created_at: s.created_at,
            key_id: s.key_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServer {
    pub name: String,
    pub host: String,
    pub port: Option<i64>,
    pub ssh_user: Option<String>,
    pub ssh_key_path: Option<String>,
    pub labels: Option<Vec<String>>,
    pub group_name: Option<String>,
    pub key_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateServer {
    pub name: Option<String>,
    pub host: Option<String>,
    pub port: Option<i64>,
    pub ssh_user: Option<String>,
    pub ssh_key_path: Option<String>,
    pub labels: Option<Vec<String>>,
    pub group_name: Option<String>,
    pub key_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Job {
    pub id: String,
    pub recipe_name: String,
    pub server_ids: String,
    pub params: Option<String>,
    pub status: String,
    pub output: Option<String>,
    pub started_at: Option<NaiveDateTime>,
    pub finished_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResponse {
    pub id: String,
    pub recipe_name: String,
    pub server_ids: Vec<String>,
    pub params: Option<serde_json::Value>,
    pub status: String,
    pub output: Option<String>,
    pub started_at: Option<NaiveDateTime>,
    pub finished_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

impl From<Job> for JobResponse {
    fn from(j: Job) -> Self {
        let server_ids: Vec<String> = serde_json::from_str(&j.server_ids).unwrap_or_default();
        let params: Option<serde_json::Value> =
            j.params.as_deref().and_then(|p| serde_json::from_str(p).ok());
        JobResponse {
            id: j.id,
            recipe_name: j.recipe_name,
            server_ids,
            params,
            status: j.status,
            output: j.output,
            started_at: j.started_at,
            finished_at: j.finished_at,
            created_by: j.created_by,
            created_at: j.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJob {
    pub recipe_name: String,
    pub server_ids: Vec<String>,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub role: String,
    pub exp: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub server_count: i64,
    pub servers_online: i64,
    pub servers_offline: i64,
    pub active_jobs: i64,
    pub total_jobs: i64,
    pub successful_jobs: i64,
    pub failed_jobs: i64,
    pub recent_jobs: Vec<JobResponse>,
    pub active_schedules: i64,
}

// ─── Marketplace: Recipe Sources ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RecipeSource {
    pub id: String,
    pub name: String,
    pub url: String,
    pub description: Option<String>,
    pub status: String,
    pub sync_error: Option<String>,
    pub last_synced_at: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SourceRecipe {
    pub id: String,
    pub source_id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub playbook: String,
    pub version: String,
    pub tags: String,
    pub installed: i64,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRecipeResponse {
    pub id: String,
    pub source_id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub playbook: String,
    pub version: String,
    pub tags: Vec<String>,
    pub installed: bool,
    pub created_at: Option<NaiveDateTime>,
}

impl From<SourceRecipe> for SourceRecipeResponse {
    fn from(r: SourceRecipe) -> Self {
        let tags: Vec<String> = serde_json::from_str(&r.tags).unwrap_or_default();
        SourceRecipeResponse {
            id: r.id,
            source_id: r.source_id,
            slug: r.slug,
            name: r.name,
            description: r.description,
            playbook: r.playbook,
            version: r.version,
            tags,
            installed: r.installed != 0,
            created_at: r.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeSourceWithRecipes {
    #[serde(flatten)]
    pub source: RecipeSource,
    pub recipes: Vec<SourceRecipeResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSource {
    pub url: String,
    pub description: Option<String>,
}

// ─── Audit Log ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLog {
    pub id: String,
    pub username: String,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

// ─── Key Store ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct KeyStoreEntry {
    pub id: String,
    pub name: String,
    pub key_type: String,
    pub key_data: String,
    pub description: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyStoreResponse {
    pub id: String,
    pub name: String,
    pub key_type: String,
    pub has_data: bool,
    pub description: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

impl From<KeyStoreEntry> for KeyStoreResponse {
    fn from(k: KeyStoreEntry) -> Self {
        KeyStoreResponse {
            id: k.id,
            name: k.name,
            key_type: k.key_type,
            has_data: !k.key_data.is_empty(),
            description: k.description,
            created_by: k.created_by,
            created_at: k.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKeyStoreEntry {
    pub name: String,
    pub key_type: String,
    pub key_data: String,
    pub description: Option<String>,
}

// ─── Schedules ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Schedule {
    pub id: String,
    pub name: String,
    pub recipe_name: String,
    pub server_ids: String,
    pub params: Option<String>,
    pub cron_expression: String,
    pub enabled: i64,
    pub last_run_at: Option<NaiveDateTime>,
    pub next_run_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleResponse {
    pub id: String,
    pub name: String,
    pub recipe_name: String,
    pub server_ids: Vec<String>,
    pub params: Option<serde_json::Value>,
    pub cron_expression: String,
    pub enabled: bool,
    pub last_run_at: Option<NaiveDateTime>,
    pub next_run_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

impl From<Schedule> for ScheduleResponse {
    fn from(s: Schedule) -> Self {
        ScheduleResponse {
            id: s.id,
            name: s.name,
            recipe_name: s.recipe_name,
            server_ids: serde_json::from_str(&s.server_ids).unwrap_or_default(),
            params: s.params.as_deref().and_then(|p| serde_json::from_str(p).ok()),
            cron_expression: s.cron_expression,
            enabled: s.enabled != 0,
            last_run_at: s.last_run_at,
            next_run_at: s.next_run_at,
            created_by: s.created_by,
            created_at: s.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSchedule {
    pub name: String,
    pub recipe_name: String,
    pub server_ids: Vec<String>,
    pub params: Option<serde_json::Value>,
    pub cron_expression: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSchedule {
    pub name: Option<String>,
    pub cron_expression: Option<String>,
    pub server_ids: Option<Vec<String>>,
    pub params: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

// ─── Notification Channels ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NotificationChannel {
    pub id: String,
    pub name: String,
    pub channel_type: String,
    pub config: String,
    pub events: String,
    pub enabled: i64,
    pub created_by: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannelResponse {
    pub id: String,
    pub name: String,
    pub channel_type: String,
    pub config: serde_json::Value,
    pub events: Vec<String>,
    pub enabled: bool,
    pub created_by: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

impl From<NotificationChannel> for NotificationChannelResponse {
    fn from(n: NotificationChannel) -> Self {
        NotificationChannelResponse {
            id: n.id,
            name: n.name,
            channel_type: n.channel_type,
            config: serde_json::from_str(&n.config).unwrap_or(serde_json::json!({})),
            events: serde_json::from_str(&n.events).unwrap_or_default(),
            enabled: n.enabled != 0,
            created_by: n.created_by,
            created_at: n.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNotificationChannel {
    pub name: String,
    pub channel_type: String,
    pub config: serde_json::Value,
    pub events: Vec<String>,
}

// ─── Security Audits ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SecurityAudit {
    pub id: String,
    pub server_id: String,
    pub status: String,
    pub score: Option<i64>,
    pub results: Option<String>,
    pub started_at: Option<NaiveDateTime>,
    pub finished_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditResponse {
    pub id: String,
    pub server_id: String,
    pub status: String,
    pub score: Option<i64>,
    pub results: Option<Vec<SecurityCheckResult>>,
    pub started_at: Option<NaiveDateTime>,
    pub finished_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityCheckResult {
    pub name: String,
    pub category: String,
    pub status: String,    // pass | warn | fail
    pub detail: String,
    pub points: i64,       // points earned
    pub max_points: i64,   // max possible
}

impl From<SecurityAudit> for SecurityAuditResponse {
    fn from(a: SecurityAudit) -> Self {
        let results: Option<Vec<SecurityCheckResult>> = a
            .results
            .as_deref()
            .and_then(|r| serde_json::from_str(r).ok());
        SecurityAuditResponse {
            id: a.id,
            server_id: a.server_id,
            status: a.status,
            score: a.score,
            results,
            started_at: a.started_at,
            finished_at: a.finished_at,
            created_by: a.created_by,
            created_at: a.created_at,
        }
    }
}
