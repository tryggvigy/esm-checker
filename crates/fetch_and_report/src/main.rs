use fetch_and_report::fetch_and_analyze_package;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let package_name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "react".to_string());
    let result = fetch_and_analyze_package(&[package_name.clone()], None).await?;
    println!(
        "Report for {}: {}",
        package_name,
        serde_json::to_string_pretty(&result)?
    );
    Ok(())
}
