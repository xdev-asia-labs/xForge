use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use crate::db::models::{Claims, SecurityAudit, SecurityAuditResponse, SecurityCheckResult, Server};
use crate::AppState;

pub async fn start_audit(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(server_id): Path<String>,
) -> Result<(StatusCode, Json<SecurityAuditResponse>), (StatusCode, Json<serde_json::Value>)> {
    // Verify server exists
    let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
        .bind(&server_id)
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
                Json(json!({"error": "Server not found"})),
            )
        })?;

    let audit_id = Uuid::new_v4().to_string();
    let now = Utc::now().naive_utc();

    sqlx::query(
        "INSERT INTO security_audits (id, server_id, status, started_at, created_by, created_at) VALUES (?, ?, 'running', ?, ?, ?)",
    )
    .bind(&audit_id)
    .bind(&server_id)
    .bind(now)
    .bind(&claims.username)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
    })?;

    // Resolve SSH key
    let key_data = if let Some(ref key_id) = server.key_id {
        sqlx::query_as::<_, crate::db::models::KeyStoreEntry>(
            "SELECT * FROM key_store WHERE id = ?",
        )
        .bind(key_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
    } else {
        None
    };

    // Spawn background task for the audit
    let db = state.db.clone();
    let aid = audit_id.clone();
    tokio::spawn(async move {
        let results = run_security_checks(&server, key_data.as_ref()).await;
        let (score, check_results) = match results {
            Ok(checks) => {
                let total_max: i64 = checks.iter().map(|c| c.max_points).sum();
                let total_earned: i64 = checks.iter().map(|c| c.points).sum();
                let score = if total_max > 0 {
                    (total_earned * 100) / total_max
                } else {
                    0
                };
                (score, checks)
            }
            Err(e) => {
                let fail_check = SecurityCheckResult {
                    name: "Connection".to_string(),
                    category: "connectivity".to_string(),
                    status: "fail".to_string(),
                    detail: format!("Failed to connect: {}", e),
                    points: 0,
                    max_points: 10,
                };
                (0, vec![fail_check])
            }
        };

        let results_json = serde_json::to_string(&check_results).unwrap_or_default();
        let finished = Utc::now().naive_utc();

        let _ = sqlx::query(
            "UPDATE security_audits SET status = 'completed', score = ?, results = ?, finished_at = ? WHERE id = ?",
        )
        .bind(score)
        .bind(&results_json)
        .bind(finished)
        .bind(&aid)
        .execute(&db)
        .await;
    });

    let audit = sqlx::query_as::<_, SecurityAudit>("SELECT * FROM security_audits WHERE id = ?")
        .bind(&audit_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", e)})),
            )
        })?;

    Ok((StatusCode::CREATED, Json(SecurityAuditResponse::from(audit))))
}

pub async fn list_audits(
    State(state): State<AppState>,
    Path(server_id): Path<String>,
) -> Result<Json<Vec<SecurityAuditResponse>>, (StatusCode, Json<serde_json::Value>)> {
    let audits = sqlx::query_as::<_, SecurityAudit>(
        "SELECT * FROM security_audits WHERE server_id = ? ORDER BY created_at DESC LIMIT 20",
    )
    .bind(&server_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
    })?;

    Ok(Json(
        audits.into_iter().map(SecurityAuditResponse::from).collect(),
    ))
}

pub async fn get_audit(
    State(state): State<AppState>,
    Path(audit_id): Path<String>,
) -> Result<Json<SecurityAuditResponse>, (StatusCode, Json<serde_json::Value>)> {
    let audit = sqlx::query_as::<_, SecurityAudit>("SELECT * FROM security_audits WHERE id = ?")
        .bind(&audit_id)
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
                Json(json!({"error": "Audit not found"})),
            )
        })?;

    Ok(Json(SecurityAuditResponse::from(audit)))
}

// ─── SSH Security Check Runner ───────────────────────────────────────────────

async fn ssh_command(
    server: &Server,
    key_data: Option<&crate::db::models::KeyStoreEntry>,
    command: &str,
) -> Result<String, String> {
    let mut cmd = tokio::process::Command::new("ssh");
    cmd.arg("-o")
        .arg("StrictHostKeyChecking=no")
        .arg("-o")
        .arg("UserKnownHostsFile=/dev/null")
        .arg("-o")
        .arg("ConnectTimeout=10")
        .arg("-o")
        .arg("BatchMode=yes")
        .arg("-p")
        .arg(server.port.to_string());

    let temp_key_path = if let Some(key_entry) = key_data {
        if key_entry.key_type == "ssh_key" {
            let path = format!("/tmp/xforge-audit-{}", uuid::Uuid::new_v4());
            tokio::fs::write(&path, &key_entry.key_data)
                .await
                .map_err(|e| format!("Failed to write key: {}", e))?;
            let _ = tokio::process::Command::new("chmod")
                .arg("600")
                .arg(&path)
                .output()
                .await;
            cmd.arg("-i").arg(&path);
            Some(path)
        } else {
            None
        }
    } else if let Some(ref key_path) = server.ssh_key_path {
        cmd.arg("-i").arg(key_path);
        None
    } else {
        None
    };

    cmd.arg(format!("{}@{}", server.ssh_user, server.host))
        .arg(command);

    let output = cmd
        .output()
        .await
        .map_err(|e| format!("SSH execution error: {}", e))?;

    // Cleanup temp key
    if let Some(path) = temp_key_path {
        let _ = tokio::fs::remove_file(path).await;
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() && stdout.is_empty() {
        return Err(format!("Command failed: {}", stderr.trim()));
    }

    Ok(stdout)
}

async fn run_security_checks(
    server: &Server,
    key_data: Option<&crate::db::models::KeyStoreEntry>,
) -> Result<Vec<SecurityCheckResult>, String> {
    let mut checks = Vec::new();

    // 1. SSH Root Login
    checks.push(check_ssh_root_login(server, key_data).await);

    // 2. SSH Password Authentication
    checks.push(check_ssh_password_auth(server, key_data).await);

    // 3. Firewall Status
    checks.push(check_firewall(server, key_data).await);

    // 4. Open Ports
    checks.push(check_open_ports(server, key_data).await);

    // 5. Pending Updates
    checks.push(check_pending_updates(server, key_data).await);

    // 6. Failed Login Attempts
    checks.push(check_failed_logins(server, key_data).await);

    // 7. Users with UID 0
    checks.push(check_root_users(server, key_data).await);

    // 8. World-Writable Files
    checks.push(check_world_writable(server, key_data).await);

    // 9. Running Services
    checks.push(check_running_services(server, key_data).await);

    // 10. Unattended Upgrades
    checks.push(check_auto_updates(server, key_data).await);

    Ok(checks)
}

async fn check_ssh_root_login(
    server: &Server,
    key_data: Option<&crate::db::models::KeyStoreEntry>,
) -> SecurityCheckResult {
    let result = ssh_command(
        server,
        key_data,
        "grep -i '^PermitRootLogin' /etc/ssh/sshd_config 2>/dev/null || echo 'NOT_SET'",
    )
    .await;

    match result {
        Ok(output) => {
            let line = output.trim().to_lowercase();
            if line.contains("no") {
                SecurityCheckResult {
                    name: "SSH Root Login".to_string(),
                    category: "ssh".to_string(),
                    status: "pass".to_string(),
                    detail: "Root login is disabled".to_string(),
                    points: 10,
                    max_points: 10,
                }
            } else if line.contains("prohibit-password") || line.contains("without-password") {
                SecurityCheckResult {
                    name: "SSH Root Login".to_string(),
                    category: "ssh".to_string(),
                    status: "warn".to_string(),
                    detail: "Root login allowed with key only".to_string(),
                    points: 7,
                    max_points: 10,
                }
            } else {
                SecurityCheckResult {
                    name: "SSH Root Login".to_string(),
                    category: "ssh".to_string(),
                    status: "fail".to_string(),
                    detail: format!("Root login permitted: {}", line),
                    points: 0,
                    max_points: 10,
                }
            }
        }
        Err(e) => SecurityCheckResult {
            name: "SSH Root Login".to_string(),
            category: "ssh".to_string(),
            status: "fail".to_string(),
            detail: format!("Check failed: {}", e),
            points: 0,
            max_points: 10,
        },
    }
}

async fn check_ssh_password_auth(
    server: &Server,
    key_data: Option<&crate::db::models::KeyStoreEntry>,
) -> SecurityCheckResult {
    let result = ssh_command(
        server,
        key_data,
        "grep -i '^PasswordAuthentication' /etc/ssh/sshd_config 2>/dev/null || echo 'NOT_SET'",
    )
    .await;

    match result {
        Ok(output) => {
            let line = output.trim().to_lowercase();
            if line.contains("no") {
                SecurityCheckResult {
                    name: "SSH Password Auth".to_string(),
                    category: "ssh".to_string(),
                    status: "pass".to_string(),
                    detail: "Password authentication is disabled".to_string(),
                    points: 10,
                    max_points: 10,
                }
            } else {
                SecurityCheckResult {
                    name: "SSH Password Auth".to_string(),
                    category: "ssh".to_string(),
                    status: "fail".to_string(),
                    detail: "Password authentication is enabled".to_string(),
                    points: 0,
                    max_points: 10,
                }
            }
        }
        Err(e) => SecurityCheckResult {
            name: "SSH Password Auth".to_string(),
            category: "ssh".to_string(),
            status: "fail".to_string(),
            detail: format!("Check failed: {}", e),
            points: 0,
            max_points: 10,
        },
    }
}

async fn check_firewall(
    server: &Server,
    key_data: Option<&crate::db::models::KeyStoreEntry>,
) -> SecurityCheckResult {
    let result = ssh_command(
        server,
        key_data,
        "if command -v ufw >/dev/null 2>&1; then ufw status 2>/dev/null; elif command -v firewall-cmd >/dev/null 2>&1; then firewall-cmd --state 2>/dev/null; else iptables -L -n 2>/dev/null | head -5; fi",
    )
    .await;

    match result {
        Ok(output) => {
            let lower = output.trim().to_lowercase();
            if lower.contains("status: active") || lower.contains("running") {
                SecurityCheckResult {
                    name: "Firewall".to_string(),
                    category: "network".to_string(),
                    status: "pass".to_string(),
                    detail: "Firewall is active".to_string(),
                    points: 15,
                    max_points: 15,
                }
            } else if lower.contains("chain") && lower.contains("target") {
                SecurityCheckResult {
                    name: "Firewall".to_string(),
                    category: "network".to_string(),
                    status: "warn".to_string(),
                    detail: "iptables rules present but no managed firewall".to_string(),
                    points: 8,
                    max_points: 15,
                }
            } else {
                SecurityCheckResult {
                    name: "Firewall".to_string(),
                    category: "network".to_string(),
                    status: "fail".to_string(),
                    detail: "No active firewall detected".to_string(),
                    points: 0,
                    max_points: 15,
                }
            }
        }
        Err(e) => SecurityCheckResult {
            name: "Firewall".to_string(),
            category: "network".to_string(),
            status: "fail".to_string(),
            detail: format!("Check failed: {}", e),
            points: 0,
            max_points: 15,
        },
    }
}

async fn check_open_ports(
    server: &Server,
    key_data: Option<&crate::db::models::KeyStoreEntry>,
) -> SecurityCheckResult {
    let result = ssh_command(
        server,
        key_data,
        "ss -tlnp 2>/dev/null | grep LISTEN | wc -l",
    )
    .await;

    match result {
        Ok(output) => {
            let count: i64 = output.trim().parse().unwrap_or(0);
            if count <= 5 {
                SecurityCheckResult {
                    name: "Open Ports".to_string(),
                    category: "network".to_string(),
                    status: "pass".to_string(),
                    detail: format!("{} listening ports (minimal exposure)", count),
                    points: 10,
                    max_points: 10,
                }
            } else if count <= 15 {
                SecurityCheckResult {
                    name: "Open Ports".to_string(),
                    category: "network".to_string(),
                    status: "warn".to_string(),
                    detail: format!("{} listening ports", count),
                    points: 5,
                    max_points: 10,
                }
            } else {
                SecurityCheckResult {
                    name: "Open Ports".to_string(),
                    category: "network".to_string(),
                    status: "fail".to_string(),
                    detail: format!("{} listening ports (excessive)", count),
                    points: 0,
                    max_points: 10,
                }
            }
        }
        Err(e) => SecurityCheckResult {
            name: "Open Ports".to_string(),
            category: "network".to_string(),
            status: "fail".to_string(),
            detail: format!("Check failed: {}", e),
            points: 0,
            max_points: 10,
        },
    }
}

async fn check_pending_updates(
    server: &Server,
    key_data: Option<&crate::db::models::KeyStoreEntry>,
) -> SecurityCheckResult {
    let result = ssh_command(
        server,
        key_data,
        "if command -v apt >/dev/null 2>&1; then apt list --upgradable 2>/dev/null | grep -c upgradable || echo 0; elif command -v yum >/dev/null 2>&1; then yum check-update 2>/dev/null | tail -n +3 | wc -l; else echo 0; fi",
    )
    .await;

    match result {
        Ok(output) => {
            let count: i64 = output.trim().parse().unwrap_or(0);
            if count == 0 {
                SecurityCheckResult {
                    name: "Pending Updates".to_string(),
                    category: "system".to_string(),
                    status: "pass".to_string(),
                    detail: "System is up to date".to_string(),
                    points: 10,
                    max_points: 10,
                }
            } else if count <= 10 {
                SecurityCheckResult {
                    name: "Pending Updates".to_string(),
                    category: "system".to_string(),
                    status: "warn".to_string(),
                    detail: format!("{} packages need updating", count),
                    points: 5,
                    max_points: 10,
                }
            } else {
                SecurityCheckResult {
                    name: "Pending Updates".to_string(),
                    category: "system".to_string(),
                    status: "fail".to_string(),
                    detail: format!("{} packages need updating", count),
                    points: 0,
                    max_points: 10,
                }
            }
        }
        Err(e) => SecurityCheckResult {
            name: "Pending Updates".to_string(),
            category: "system".to_string(),
            status: "fail".to_string(),
            detail: format!("Check failed: {}", e),
            points: 0,
            max_points: 10,
        },
    }
}

async fn check_failed_logins(
    server: &Server,
    key_data: Option<&crate::db::models::KeyStoreEntry>,
) -> SecurityCheckResult {
    let result = ssh_command(
        server,
        key_data,
        "if [ -f /var/log/auth.log ]; then grep -c 'Failed password' /var/log/auth.log 2>/dev/null || echo 0; elif [ -f /var/log/secure ]; then grep -c 'Failed password' /var/log/secure 2>/dev/null || echo 0; else journalctl -u sshd --since '7 days ago' 2>/dev/null | grep -c 'Failed password' || echo 0; fi",
    )
    .await;

    match result {
        Ok(output) => {
            let count: i64 = output.trim().parse().unwrap_or(0);
            if count == 0 {
                SecurityCheckResult {
                    name: "Failed Logins".to_string(),
                    category: "auth".to_string(),
                    status: "pass".to_string(),
                    detail: "No failed login attempts detected".to_string(),
                    points: 10,
                    max_points: 10,
                }
            } else if count <= 50 {
                SecurityCheckResult {
                    name: "Failed Logins".to_string(),
                    category: "auth".to_string(),
                    status: "warn".to_string(),
                    detail: format!("{} failed login attempts found", count),
                    points: 5,
                    max_points: 10,
                }
            } else {
                SecurityCheckResult {
                    name: "Failed Logins".to_string(),
                    category: "auth".to_string(),
                    status: "fail".to_string(),
                    detail: format!("{} failed login attempts (possible brute-force)", count),
                    points: 0,
                    max_points: 10,
                }
            }
        }
        Err(e) => SecurityCheckResult {
            name: "Failed Logins".to_string(),
            category: "auth".to_string(),
            status: "fail".to_string(),
            detail: format!("Check failed: {}", e),
            points: 0,
            max_points: 10,
        },
    }
}

async fn check_root_users(
    server: &Server,
    key_data: Option<&crate::db::models::KeyStoreEntry>,
) -> SecurityCheckResult {
    let result = ssh_command(
        server,
        key_data,
        "awk -F: '$3 == 0 { print $1 }' /etc/passwd 2>/dev/null",
    )
    .await;

    match result {
        Ok(output) => {
            let users: Vec<&str> = output.trim().lines().collect();
            if users.len() <= 1 && users.first().map(|u| u.trim()) == Some("root") {
                SecurityCheckResult {
                    name: "Root Users".to_string(),
                    category: "auth".to_string(),
                    status: "pass".to_string(),
                    detail: "Only root has UID 0".to_string(),
                    points: 10,
                    max_points: 10,
                }
            } else {
                SecurityCheckResult {
                    name: "Root Users".to_string(),
                    category: "auth".to_string(),
                    status: "fail".to_string(),
                    detail: format!(
                        "Multiple users with UID 0: {}",
                        users.join(", ")
                    ),
                    points: 0,
                    max_points: 10,
                }
            }
        }
        Err(e) => SecurityCheckResult {
            name: "Root Users".to_string(),
            category: "auth".to_string(),
            status: "fail".to_string(),
            detail: format!("Check failed: {}", e),
            points: 0,
            max_points: 10,
        },
    }
}

async fn check_world_writable(
    server: &Server,
    key_data: Option<&crate::db::models::KeyStoreEntry>,
) -> SecurityCheckResult {
    let result = ssh_command(
        server,
        key_data,
        "find /etc /usr /var -maxdepth 2 -perm -o+w -type f 2>/dev/null | head -20 | wc -l",
    )
    .await;

    match result {
        Ok(output) => {
            let count: i64 = output.trim().parse().unwrap_or(0);
            if count == 0 {
                SecurityCheckResult {
                    name: "World-Writable Files".to_string(),
                    category: "filesystem".to_string(),
                    status: "pass".to_string(),
                    detail: "No world-writable files in critical directories".to_string(),
                    points: 10,
                    max_points: 10,
                }
            } else {
                SecurityCheckResult {
                    name: "World-Writable Files".to_string(),
                    category: "filesystem".to_string(),
                    status: "warn".to_string(),
                    detail: format!("{} world-writable files found", count),
                    points: 3,
                    max_points: 10,
                }
            }
        }
        Err(e) => SecurityCheckResult {
            name: "World-Writable Files".to_string(),
            category: "filesystem".to_string(),
            status: "fail".to_string(),
            detail: format!("Check failed: {}", e),
            points: 0,
            max_points: 10,
        },
    }
}

async fn check_running_services(
    server: &Server,
    key_data: Option<&crate::db::models::KeyStoreEntry>,
) -> SecurityCheckResult {
    let result = ssh_command(
        server,
        key_data,
        "systemctl list-units --type=service --state=running --no-legend 2>/dev/null | wc -l",
    )
    .await;

    match result {
        Ok(output) => {
            let count: i64 = output.trim().parse().unwrap_or(0);
            if count <= 20 {
                SecurityCheckResult {
                    name: "Running Services".to_string(),
                    category: "system".to_string(),
                    status: "pass".to_string(),
                    detail: format!("{} services running (minimal)", count),
                    points: 5,
                    max_points: 5,
                }
            } else if count <= 40 {
                SecurityCheckResult {
                    name: "Running Services".to_string(),
                    category: "system".to_string(),
                    status: "warn".to_string(),
                    detail: format!("{} services running", count),
                    points: 3,
                    max_points: 5,
                }
            } else {
                SecurityCheckResult {
                    name: "Running Services".to_string(),
                    category: "system".to_string(),
                    status: "fail".to_string(),
                    detail: format!("{} services running (excessive)", count),
                    points: 0,
                    max_points: 5,
                }
            }
        }
        Err(e) => SecurityCheckResult {
            name: "Running Services".to_string(),
            category: "system".to_string(),
            status: "fail".to_string(),
            detail: format!("Check failed: {}", e),
            points: 0,
            max_points: 5,
        },
    }
}

async fn check_auto_updates(
    server: &Server,
    key_data: Option<&crate::db::models::KeyStoreEntry>,
) -> SecurityCheckResult {
    let result = ssh_command(
        server,
        key_data,
        "if dpkg -l unattended-upgrades 2>/dev/null | grep -q '^ii'; then echo 'INSTALLED'; elif systemctl is-enabled dnf-automatic.timer 2>/dev/null | grep -q enabled; then echo 'INSTALLED'; else echo 'NOT_INSTALLED'; fi",
    )
    .await;

    match result {
        Ok(output) => {
            let line = output.trim();
            if line.contains("INSTALLED") {
                SecurityCheckResult {
                    name: "Auto Updates".to_string(),
                    category: "system".to_string(),
                    status: "pass".to_string(),
                    detail: "Automatic security updates are configured".to_string(),
                    points: 10,
                    max_points: 10,
                }
            } else {
                SecurityCheckResult {
                    name: "Auto Updates".to_string(),
                    category: "system".to_string(),
                    status: "warn".to_string(),
                    detail: "Automatic updates not configured".to_string(),
                    points: 0,
                    max_points: 10,
                }
            }
        }
        Err(e) => SecurityCheckResult {
            name: "Auto Updates".to_string(),
            category: "system".to_string(),
            status: "fail".to_string(),
            detail: format!("Check failed: {}", e),
            points: 0,
            max_points: 10,
        },
    }
}
