use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub created_at: Option<NaiveDateTime>,
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
    pub active_jobs: i64,
    pub recent_jobs: Vec<JobResponse>,
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
