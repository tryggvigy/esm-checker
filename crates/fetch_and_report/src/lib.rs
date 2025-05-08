use anyhow::{Context, Result};
use is_esm_ready_yet::generate_report::generate_report;
use std::path::PathBuf;
use tempfile::TempDir;
use tracing::{info, warn};

pub async fn fetch_and_analyze_package(
    package_names: &[String],
    debug_dir: Option<PathBuf>,
) -> Result<serde_json::Value> {
    info!("Starting package analysis for: {:?}", package_names);

    // Create a temporary directory for the npm install or use debug directory
    let (temp_dir, temp_path) = if let Some(debug_path) = debug_dir {
        info!("Using debug directory at: {:?}", debug_path);
        // Create the directory if it doesn't exist
        std::fs::create_dir_all(&debug_path).context("Failed to create debug directory")?;
        (None, debug_path)
    } else {
        let dir = TempDir::new().context("Failed to create temporary directory")?;
        let path = dir.path().to_path_buf();
        info!("Created temporary directory at: {:?}", path);
        (Some(dir), path)
    };

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
    info!("Created package.json at: {:?}", package_json_path);

    // Run npm install with cache
    info!("Running npm install...");
    let output = tokio::process::Command::new("npm")
        .arg("install")
        .arg("--no-cache")
        .current_dir(&temp_path)
        .output()
        .await
        .context("Failed to run npm install")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        warn!("npm install failed: {}", error);
        anyhow::bail!("npm install failed: {}", error);
    }
    info!("npm install completed successfully");

    // Generate the report for all packages
    info!("Generating report...");
    let report = generate_report(
        package_json_path.to_str().unwrap(),
        Some(package_names.to_vec()),
    )
    .map_err(|e| anyhow::anyhow!("Failed to generate report: {}", e))?;

    // Convert to JSON
    let json_report = serde_json::to_value(report)?;
    info!("Report generation completed successfully");

    // Keep the temp_dir in scope until the end of the function
    drop(temp_dir);

    Ok(json_report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_and_analyze_multiple_packages() {
        let packages = vec!["react".to_string(), "vue".to_string()];
        let result = fetch_and_analyze_package(&packages, None).await.unwrap();
        println!(
            "Report for multiple packages: {}",
            serde_json::to_string_pretty(&result).unwrap()
        );
    }
}
