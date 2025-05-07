use es_resolver::package_json::PackageJsonParser;
use es_resolver::prelude::*;
// cargo test -p walk_imports -- --nocapture
use pretty_assertions::assert_eq;
use std::collections::BTreeSet;
use std::env;
use std::path::PathBuf;

use crate::analyze::{analyze_package, Analysis};

fn test_repo_path() -> PathBuf {
    env::current_dir().unwrap().join("../../").join("test_repo")
}

#[test]
fn react() {
    assert_eq!(
        analyze_package(
            &test_repo_path(),
            "react",
            &PackageJsonParser::new(),
            &presets::get_default_es_resolver(),
        )
        .unwrap(),
        Analysis {
            package_name: "react".to_string(),
            is_entry_esm: false,
            esm_missing_js_file_extensions: BTreeSet::new(),
            transitive_commonjs_dependencies: BTreeSet::new(),
        }
    )
}

#[test]
fn loadable_component() {
    let transitive_commonjs_dependencies: BTreeSet<String> =
        ["react", "react-is", "hoist-non-react-statics"]
            .iter()
            .map(|d| d.to_string())
            .collect();

    assert_eq!(
        analyze_package(
            &test_repo_path(),
            "@loadable/component",
            &PackageJsonParser::new(),
            &presets::get_default_es_resolver(),
        )
        .unwrap(),
        Analysis {
            package_name: "@loadable/component".to_string(),
            is_entry_esm: true,
            esm_missing_js_file_extensions: BTreeSet::new(),
            transitive_commonjs_dependencies,
        }
    )
}

#[test]
fn murmurhash() {
    assert_eq!(
        analyze_package(
            &test_repo_path(),
            "murmurhash",
            &PackageJsonParser::new(),
            &presets::get_default_es_resolver(),
        )
        .unwrap(),
        Analysis {
            package_name: "murmurhash".to_string(),
            is_entry_esm: false,
            esm_missing_js_file_extensions: BTreeSet::new(),
            transitive_commonjs_dependencies: BTreeSet::new(),
        }
    )
}
