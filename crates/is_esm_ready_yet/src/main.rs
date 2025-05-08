#![warn(missing_debug_implementations, rust_2018_idioms)]

use crate::generate_report::generate_report;
use clap::Parser as ClapParser;
use std::{error::Error, path::PathBuf, time::Instant};
use tracing::{info, Level};
use tracing_subscriber::{EnvFilter, FmtSubscriber};
mod generate_report;
mod pkg_json;

#[derive(ClapParser, Debug)]
#[command(author, version, about = "Checks ESM readiness of a project")]
struct Args {
    #[arg(short, long)]
    /// package.json file to check
    package_json_location: String,

    #[arg(short, long)]
    /// output .json file to write results to (absolute path)
    outfile: Option<String>,

    #[arg(short, long, value_delimiter = ',')]
    /// The dependencies to check, checks all if omitted.
    check: Option<Vec<String>>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_names(true)
        .with_level(true)
        .with_ansi(true)
        .pretty()
        .init();

    let args = Args::parse();

    let report = generate_report(&args.package_json_location, args.check.clone())?;

    if let Some(out) = &args.outfile {
        let outfile = PathBuf::from(out);

        let json_report = serde_json::to_string_pretty(&report)?;

        std::fs::write(&outfile, json_report)?;

        println!("Report written to {:?}", outfile);
    } else {
        println!("Report:");
        println!("{:?}", report);
    }

    let duration = start.elapsed();
    info!("Scanned {} dependencies", report.total);
    info!("ESM: {}", report.esm.len());
    info!("CommonJS: {}", report.cjs.len());
    info!(
        "Faux ESM with CommonJS transitive dependencies: {}",
        report.faux_esm.with_commonjs_dependencies.len()
    );
    info!(
        "Faux ESM with missing JS file extensions: {}",
        report.faux_esm.with_missing_js_file_extensions.len()
    );
    info!("Resolve errors: {}", report.resolve_errors.len());
    info!("Parse errors: {}", report.parse_errors.len());

    println!("Done in {:#?}", duration);

    Ok(())
}
