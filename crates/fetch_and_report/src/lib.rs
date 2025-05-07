use anyhow::{Context, Result};
use is_esm_ready_yet::generate_report::generate_report;
use std::path::PathBuf;
use std::sync::OnceLock;
use tempfile::TempDir;

static NPM_CACHE_DIR: OnceLock<PathBuf> = OnceLock::new();

fn get_npm_cache_dir() -> PathBuf {
    NPM_CACHE_DIR
        .get_or_init(|| {
            let cache_dir = std::env::var("NPM_CACHE_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    let mut dir = std::env::temp_dir();
                    dir.push("npm-cache");
                    dir
                });

            // Ensure the cache directory exists
            std::fs::create_dir_all(&cache_dir).expect("Failed to create npm cache directory");

            cache_dir
        })
        .clone()
}

pub async fn fetch_and_analyze_package(package_names: &[String]) -> Result<serde_json::Value> {
    // Create a temporary directory for the npm install
    let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
    let temp_path = temp_dir.path();

    // Create a package.json file with all dependencies
    let dependencies = package_names
        .iter()
        .map(|name| format!(r#""{}": "latest""#, name))
        .collect::<Vec<_>>()
        .join(",\n                ");

    let package_json = format!(
        r#"{{
            "name": "temp-package",
            "version": "1.0.0",
            "dependencies": {{
                {}
            }}
        }}"#,
        dependencies
    );

    let package_json_path = temp_path.join("package.json");
    std::fs::write(&package_json_path, package_json).context("Failed to write package.json")?;

    // Get the npm cache directory
    let cache_dir = get_npm_cache_dir();

    // Run npm install with cache
    let output = tokio::process::Command::new("npm")
        .arg("install")
        .arg("--cache")
        .arg(cache_dir)
        .current_dir(temp_path)
        .output()
        .await
        .context("Failed to run npm install")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("npm install failed: {}", error);
    }

    // Generate the report for all packages
    let report = generate_report(
        package_json_path.to_str().unwrap(),
        Some(package_names.to_vec()),
    )
    .map_err(|e| anyhow::anyhow!("Failed to generate report: {}", e))?;

    // Convert to JSON
    let json_report = serde_json::to_value(report)?;

    Ok(json_report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_and_analyze_multiple_packages() {
        let packages = vec!["react".to_string(), "vue".to_string()];
        let result = fetch_and_analyze_package(&packages).await.unwrap();
        println!(
            "Report for multiple packages: {}",
            serde_json::to_string_pretty(&result).unwrap()
        );
    }
}
