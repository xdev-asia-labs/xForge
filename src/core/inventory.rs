use anyhow::Result;
use serde_json::json;
use sqlx::SqlitePool;

use crate::db::models::Server;

pub async fn generate_inventory(db: &SqlitePool, server_ids: &[String]) -> Result<String> {
    let mut hosts = serde_json::Map::new();
    let mut groups: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

    for server_id in server_ids {
        let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
            .bind(server_id)
            .fetch_one(db)
            .await?;

        let mut host_vars = serde_json::Map::new();
        host_vars.insert(
            "ansible_host".to_string(),
            json!(server.host),
        );
        host_vars.insert(
            "ansible_port".to_string(),
            json!(server.port),
        );
        host_vars.insert(
            "ansible_user".to_string(),
            json!(server.ssh_user),
        );
        if let Some(ref key_path) = server.ssh_key_path {
            host_vars.insert(
                "ansible_ssh_private_key_file".to_string(),
                json!(key_path),
            );
        }
        host_vars.insert(
            "ansible_ssh_common_args".to_string(),
            json!("-o StrictHostKeyChecking=no"),
        );

        hosts.insert(
            server.name.clone(),
            json!(host_vars),
        );

        // Add to group if specified
        let group = server.group_name.unwrap_or_else(|| "ungrouped".to_string());
        groups
            .entry(group)
            .or_default()
            .push(server.name.clone());
    }

    // Build Ansible JSON inventory format
    let mut inventory = serde_json::Map::new();
    let mut all = serde_json::Map::new();
    all.insert("hosts".to_string(), json!(hosts));

    let mut children = serde_json::Map::new();
    for (group_name, group_hosts) in &groups {
        let mut group_data = serde_json::Map::new();
        let hosts_map: serde_json::Map<String, serde_json::Value> = group_hosts
            .iter()
            .map(|h| (h.clone(), json!({})))
            .collect();
        group_data.insert("hosts".to_string(), json!(hosts_map));
        children.insert(group_name.clone(), json!(group_data));
    }
    all.insert("children".to_string(), json!(children));

    inventory.insert("all".to_string(), json!(all));

    let inventory_json = serde_json::to_string_pretty(&inventory)?;
    Ok(inventory_json)
}
