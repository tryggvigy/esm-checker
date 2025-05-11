use anyhow::{Context, Result};
use report_model::Report;
use reporter::generate_report::generate_report;
use std::path::PathBuf;
use tempfile::TempDir;
use tracing::{info, warn};

pub async fn fetch_and_analyze_package(
    package_names: &[String],
    debug_dir: Option<PathBuf>,
) -> Result<Report> {
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
        .arg("--ignore-scripts")
        .arg("--no-bin-links")
        .arg("--no-audit")
        .arg("--no-package-lock")
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

    info!("Report generation completed successfully");

    // Keep the temp_dir in scope until the end of the function
    drop(temp_dir);

    Ok(report)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::env;

//     #[tokio::test]
//     #[ignore = "Skipping network test in CI environment"]
//     async fn test_fetch_and_analyze_multiple_packages() {
//         // Skip test in CI environment
//         if env::var("CI").is_ok() {
//             return;
//         }

//         let packages = vec!["screenfull".to_string()];
//         let result = fetch_and_analyze_package(&packages, None).await.unwrap();

//         assert_eq!(result.total, 1);
//         assert_eq!(result.esm.len(), 1);
//         assert_eq!(result.cjs.len(), 0);
//         assert_eq!(result.faux_esm.with_commonjs_dependencies.len(), 0);
//         assert_eq!(result.faux_esm.with_missing_js_file_extensions.len(), 0);
//         assert_eq!(result.resolve_errors.len(), 0);
//         assert_eq!(result.parse_errors.len(), 0);
//     }
// }
