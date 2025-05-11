use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::PathBuf};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithCommonJSDependencies {
    pub package_name: String,
    pub transitive_commonjs_dependencies: BTreeSet<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithMissingJsFileExtensions {
    pub package_name: String,
    pub transitive_deps_with_missing_js_file_extensions: BTreeSet<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FauxESM {
    pub with_commonjs_dependencies: Vec<WithCommonJSDependencies>,
    pub with_missing_js_file_extensions: Vec<WithMissingJsFileExtensions>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveError {
    pub package_name: String,
    pub from: PathBuf,
    pub import_specifier: String,
    pub original_error_message: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseError {
    pub package_name: String,
    pub path: PathBuf,
    pub original_error_message: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub total: usize,
    pub esm: Vec<String>,
    pub cjs: Vec<String>,
    pub faux_esm: FauxESM,
    pub resolve_errors: Vec<ResolveError>,
    pub parse_errors: Vec<ParseError>,
}
