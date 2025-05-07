use rayon::prelude::*;
use std::{error::Error, fs::canonicalize, sync::Arc};

use es_resolver::package_json::PackageJsonParser;
use es_resolver::prelude::*;

use walk_imports::{
    analyze::analyze_package,
    report::{into_report, Report},
};

use crate::pkg_json::PackageJson;

pub fn generate_report(
    package_json_location: &str,
    check: Option<Vec<String>>,
) -> Result<Report, Box<dyn Error>> {
    let abs_pkg_json_path = canonicalize(package_json_location)?;

    let pkg = PackageJson::load(&abs_pkg_json_path)?;
    log::debug!("Analysing {:?}", abs_pkg_json_path);
    log::trace!("Package.json dependencies {:?}", pkg.dependencies);

    let pkg_json_repo = abs_pkg_json_path.parent().unwrap_or_else(|| {
        panic!(
            "Unable to get the directory of package.json from {:?}",
            &package_json_location
        )
    });

    let mut dependency_names: Vec<_> = pkg.dependencies.keys().collect();

    if let Some(check) = check {
        dependency_names.retain(|n| check.contains(n));
    }

    let package_json_parser = Arc::new(PackageJsonParser::new());
    let node_resolver =
        presets::get_default_es_resolver_with_package_json_parser(Arc::clone(&package_json_parser));
    let analyses = dependency_names
        .par_iter()
        .filter(|dependency_name| !dependency_name.starts_with("@types/"))
        .map(|dependency_name| {
            analyze_package(
                pkg_json_repo,
                dependency_name,
                &package_json_parser,
                &node_resolver,
            )
        })
        .collect::<Vec<_>>();

    Ok(into_report(analyses))
}

// RUST_BACKTRACE=1 cargo test -p is_esm_ready_yet --  --show-output --nocapture
#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use std::env;
    use walk_imports::report::types::FauxESM;
    use walk_imports::report::Report;

    use crate::generate_report;

    fn pkg_json() -> String {
        let test_repo_path = env::current_dir()
            .unwrap()
            .join("../../")
            .join("test_repo")
            .join("package.json");
        test_repo_path.into_os_string().into_string().unwrap()
    }

    #[test]
    fn react() {
        assert_eq!(
            generate_report(&pkg_json(), Some(vec![String::from("react")])).unwrap(),
            Report {
                total: 1,
                cjs: vec![String::from("react")],
                esm: vec![],
                faux_esm: FauxESM {
                    with_commonjs_dependencies: vec![],
                    with_missing_js_file_extensions: vec![],
                },
                resolve_errors: vec![],
                parse_errors: vec![],
            }
        )
    }

    #[test]
    fn alloc_quick_lru() {
        assert_eq!(
            generate_report(&pkg_json(), Some(vec![String::from("@alloc/quick-lru")])).unwrap(),
            Report {
                total: 1,
                cjs: vec![String::from("@alloc/quick-lru")],
                esm: vec![],
                faux_esm: FauxESM {
                    with_commonjs_dependencies: vec![],
                    with_missing_js_file_extensions: vec![],
                },
                resolve_errors: vec![],
                parse_errors: vec![],
            }
        )
    }
}
