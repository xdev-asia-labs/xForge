use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub name: String,
    pub version: String,
    pub description: String,
    pub params: Option<Vec<RecipeParam>>,
    pub requires: Option<RecipeRequirements>,
    pub playbook: String,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeParam {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub default: Option<serde_yaml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeRequirements {
    pub min_servers: Option<i32>,
    pub os: Option<String>,
}

pub fn load_recipes(recipes_dir: &str) -> Result<Vec<Recipe>> {
    let dir = Path::new(recipes_dir);
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut recipes = Vec::new();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let recipe_file = path.join("recipe.yaml");
            if recipe_file.exists() {
                let content = std::fs::read_to_string(&recipe_file)?;
                match serde_yaml::from_str::<Recipe>(&content) {
                    Ok(recipe) => recipes.push(recipe),
                    Err(e) => {
                        tracing::warn!(
                            "Failed to parse recipe at {}: {}",
                            recipe_file.display(),
                            e
                        );
                    }
                }
            }
        }
    }

    Ok(recipes)
}
