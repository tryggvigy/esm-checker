#![warn(missing_debug_implementations, rust_2018_idioms)]

use crate::generate_report::generate_report;
use clap::Parser as ClapParser;
use std::{error::Error, path::PathBuf, time::Instant};
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

    env_logger::builder().format_timestamp(None).init();

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
    log::info!("Scanned {} dependencies", report.total);
    log::info!("ESM: {}", report.esm.len());
    log::info!("CommonJS: {}", report.cjs.len());
    log::info!(
        "Faux ESM with CommonJS transitive dependencies: {}",
        report.faux_esm.with_commonjs_dependencies.len()
    );
    log::info!(
        "Faux ESM with missing JS file extensions: {}",
        report.faux_esm.with_missing_js_file_extensions.len()
    );
    log::info!("Resolve errors: {}", report.resolve_errors.len());
    log::info!("Parse errors: {}", report.parse_errors.len());

    println!("Done in {:#?}", duration);

    Ok(())
}
