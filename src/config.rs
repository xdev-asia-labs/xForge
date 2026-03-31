use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub host: String,
    pub port: u16,
    pub recipes_dir: String,
    pub sources_dir: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let to_abs = |rel: String| -> String {
            let p = std::path::PathBuf::from(&rel);
            if p.is_absolute() { rel } else { cwd.join(p).to_string_lossy().to_string() }
        };
        AppConfig {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:xforge.db?mode=rwc".to_string()),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "xforge-dev-secret-change-in-production".to_string()),
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            recipes_dir: to_abs(std::env::var("RECIPES_DIR").unwrap_or_else(|_| "./recipes".to_string())),
            sources_dir: to_abs(std::env::var("SOURCES_DIR").unwrap_or_else(|_| "./sources".to_string())),
        }
    }
}
