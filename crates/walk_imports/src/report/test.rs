use es_resolver::package_json::PackageJsonParser;
use es_resolver::prelude::*;
// cargo test -p walk_imports -- --nocapture
use pretty_assertions::assert_eq;
use std::{env, path::PathBuf, sync::Arc};

use crate::{
    analyze::analyze_package,
    report::{
        into_report,
        types::{FauxESM, WithCommonJSDependencies},
        Report,
    },
};

fn test_repo_path() -> PathBuf {
    env::current_dir().unwrap().join("../../").join("test_repo")
}

#[test]
fn create_report() {
    let package_json_parser = Arc::new(PackageJsonParser::new());
    let es_resolver =
        presets::get_default_es_resolver_with_package_json_parser(Arc::clone(&package_json_parser));
    let analyses = vec![
        analyze_package(
            &test_repo_path(),
            "react",
            &package_json_parser,
            &es_resolver,
        ),
        analyze_package(
            &test_repo_path(),
            "@loadable/component",
            &package_json_parser,
            &es_resolver,
        ),
    ];

    assert_eq!(
        into_report(analyses),
        Report {
            total: 2,
            esm: vec![],
            cjs: vec!["react".to_string()],
            faux_esm: FauxESM {
                with_commonjs_dependencies: vec![WithCommonJSDependencies {
                    package_name: "@loadable/component".to_string(),
                    transitive_commonjs_dependencies: [
                        "react",
                        "react-is",
                        "hoist-non-react-statics"
                    ]
                    .iter()
                    .map(|d| d.to_string())
                    .collect()
                }],
                with_missing_js_file_extensions: vec![],
            },
            resolve_errors: vec![],
            parse_errors: vec![],
        }
    )
}
