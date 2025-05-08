use std::{
    collections::{hash_map::RandomState, HashMap},
    fs,
    hash::BuildHasher,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use crate::errors::ResolveError;

use super::{ExportsLikeField, FilenameOrConditional, PackageJson, RawPackageJson};

use tracing::warn;

const SHARDS: usize = 8;

/// Parses package.json files and caches the results.
#[derive(Debug, Default)]
pub struct PackageJsonParser {
    parsed: [RwLock<HashMap<PathBuf, Arc<PackageJson>>>; SHARDS],
    hasher: RandomState,
}

impl PackageJsonParser {
    /// The name of the `node_modules` directory.
    pub const NODE_MODULES: &'static str = "node_modules";

    /// The name of the `package.json` file.
    pub const PACKAGE_JSON: &'static str = "package.json";

    /// Create a new [`PackageJsonParser`].
    pub fn new() -> Self {
        Default::default()
    }

    /// Given a [`Path`], find the nearest `node_modules` directory.
    /// Crawls up first until it encounters a `node_modules` directory, then as a last resort
    /// checks if there's a `node_modules` directory in the current directory.
    pub(crate) fn find_node_modules(&self, from: &Path) -> Result<PathBuf, ResolveError> {
        let mut current = from.to_owned();
        loop {
            if current.file_name() == Some(Self::NODE_MODULES.as_ref()) {
                return Ok(current);
            }

            if !current.pop() {
                break;
            }
        }

        // Last attempt: check if there's a node_modules in the current directory.
        let potential_path = from.join(Self::NODE_MODULES);
        if potential_path.is_dir() {
            return Ok(potential_path);
        }

        Err(ResolveError::NodeModulesNotFound)
    }

    /// Find the nearest `package.json` file in the given directory. Crawls up until it finds one,
    /// or returns an error if it reaches the filesystem root.
    pub(crate) fn find_package_json(&self, from_directory: &Path) -> Result<PathBuf, ResolveError> {
        let mut current = from_directory.to_owned();
        loop {
            let package_json_path = current.join(Self::PACKAGE_JSON);
            if package_json_path.is_file() {
                return Ok(package_json_path);
            }

            if !current.pop() {
                break;
            }
        }

        Err(ResolveError::PackageJsonNotFound(from_directory.to_owned()))
    }

    /// Get the previously parsed `package.json` in the given directory, or parse it if it hasn't
    /// been parsed yet.
    pub fn get_or_parse_package_json(
        &self,
        // TODO: Make this take a `package.json` path, not a module path.
        module_path: PathBuf,
        package_name: Option<String>,
    ) -> Result<Arc<PackageJson>, ResolveError> {
        let shard_index = self.hasher.hash_one(&module_path) as usize % SHARDS;
        let parsed = self.parsed[shard_index].read().unwrap();
        if let Some(package_json) = parsed.get(&module_path) {
            return Ok(package_json.clone());
        }

        drop(parsed);

        let package_json = Self::parse_package_json_file(module_path.clone(), package_name)?;
        let package_json = Arc::new(package_json);
        self.parsed[shard_index]
            .write()
            .unwrap()
            .insert(module_path, package_json.clone());
        Ok(package_json)
    }

    /// Parse the `package.json` file in the given directory.
    pub(crate) fn parse_package_json_file(
        module_path: PathBuf,
        package_name: Option<String>,
    ) -> Result<PackageJson, ResolveError> {
        let package_json_path = module_path.join(Self::PACKAGE_JSON);
        let file_contents = fs::read_to_string(&package_json_path)
            .map_err(|e| ResolveError::IoError(package_json_path.clone(), e))?;

        Self::parse_package_json_string(module_path, package_name, &file_contents)
            .map_err(|e| ResolveError::ParsePackageJsonFailed(package_json_path, e))
    }

    /// Parse a `package.json` string.
    pub(crate) fn parse_package_json_string(
        module_path: PathBuf,
        package_name: Option<String>,
        file_contents: &str,
    ) -> Result<PackageJson, serde_json::Error> {
        let raw = Self::parse_raw_package_json(file_contents)?;

        Ok(PackageJson {
            package_root: module_path,
            parsed_exports: raw
                .name
                .as_ref()
                .or(package_name.as_ref())
                .and_then(|package_name| {
                    Self::parse_exports_like_field(package_name, raw.exports.as_ref())
                }),
            parsed_main: raw
                .name
                .as_ref()
                .or(package_name.as_ref())
                .and_then(|package_name| {
                    Self::parse_exports_like_field(package_name, raw.main.as_ref())
                }),
            parsed_module: raw
                .name
                .as_ref()
                .or(package_name.as_ref())
                .and_then(|package_name| {
                    Self::parse_exports_like_field(package_name, raw.module.as_ref())
                }),
            parsed_browser: raw
                .name
                .as_ref()
                .or(package_name.as_ref())
                .and_then(|package_name| {
                    Self::parse_exports_like_field(package_name, raw.browser.as_ref())
                }),
            parsed_types: raw
                .name
                .as_ref()
                .or(package_name.as_ref())
                .and_then(|package_name| {
                    Self::parse_exports_like_field(package_name, raw.types.as_ref())
                }),
            name: raw.name.clone().or(package_name),
            raw,
        })
    }

    fn parse_raw_package_json(file_contents: &str) -> Result<RawPackageJson, serde_json::Error> {
        let parsed = serde_json::from_str::<RawPackageJson>(file_contents);
        match parsed {
            Ok(object) => Ok(object),
            Err(e) => {
                warn!("Failed to parse package.json: {}", e);
                Err(e)
            }
        }
    }

    fn parse_exports_like_field(
        package_name: &str,
        input: Option<&serde_json::Value>,
    ) -> Option<ExportsLikeField> {
        input.and_then(|value| {
            match value {
                serde_json::Value::String(s) => Some(ExportsLikeField::Filename(s.clone())),
                serde_json::Value::Object(o) if o.keys().any(|k| k.starts_with('.')) => {
                    let mut map = HashMap::new();
                    Self::parse_export_names(&mut map, o, package_name)?;
                    Some(ExportsLikeField::Map(map))
                }
                serde_json::Value::Object(o) => {
                    let mut map = HashMap::new();
                    Self::parse_exports_conditions(&mut map, o, package_name)?;
                    Some(ExportsLikeField::Conditional(map))
                }
                // The other values are unexpected, let's not deal with them
                // (e.g. null, boolean, arrays, and so forth).
                _ => None,
            }
        })
    }

    fn parse_export_names(
        hash_map: &mut HashMap<String, FilenameOrConditional>,
        object: &serde_json::Map<String, serde_json::Value>,
        parent_name: &str,
    ) -> Option<()> {
        for (key, value) in object {
            let parsed_key = Self::parse_export_key(key, parent_name);
            match value {
                serde_json::Value::String(s) => {
                    hash_map.insert(parsed_key, FilenameOrConditional::Filename(s.clone()));
                }
                // If any of the keys start with a '.', that means we should
                // recurse, as we're looking at nested exports. If not, this
                // is a map with condition names.
                serde_json::Value::Object(o) if o.keys().any(|k| k.starts_with('.')) => {
                    Self::parse_export_names(hash_map, o, &parsed_key)?;
                }
                serde_json::Value::Object(o) => {
                    let mut map = HashMap::new();
                    Self::parse_exports_conditions(&mut map, o, &parsed_key)?;
                    hash_map.insert(parsed_key, FilenameOrConditional::Conditional(map));
                }
                // The other values are unexpected, let's not deal with them
                // (e.g. null, boolean, arrays, and so forth).
                _ => {}
            }
        }

        Some(())
    }

    fn parse_exports_conditions(
        hash_map: &mut HashMap<String, FilenameOrConditional>,
        object: &serde_json::Map<String, serde_json::Value>,
        parent_name: &str,
    ) -> Option<()> {
        for (key, value) in object {
            let parsed_key = Self::parse_export_key(key, parent_name);

            match value {
                serde_json::Value::String(s) => {
                    hash_map.insert(parsed_key, FilenameOrConditional::Filename(s.clone()));
                }
                serde_json::Value::Object(_) => {
                    let mut map = HashMap::new();
                    Self::parse_condition_value(&mut map, value, parent_name)?;
                    hash_map.insert(parsed_key, FilenameOrConditional::Conditional(map));
                }
                _ => {
                    // Propagate errors to not end up with a partially parsed `exports` field.
                    return None;
                }
            }
        }

        Some(())
    }

    fn parse_condition_value(
        map: &mut HashMap<String, FilenameOrConditional>,
        value: &serde_json::Value,
        parent_name: &str,
    ) -> Option<()> {
        match value {
            serde_json::Value::String(s) => {
                map.insert(
                    parent_name.to_string(),
                    FilenameOrConditional::Filename(s.clone()),
                );
            }
            serde_json::Value::Object(o) => {
                Self::parse_exports_conditions(map, o, parent_name)?;
            }
            _ => {
                return None;
            }
        }

        Some(())
    }

    fn parse_export_key(key: &str, parent_name: &str) -> String {
        if let Some(trailing) = key.strip_prefix('.') {
            format!("{parent_name}{trailing}")
        } else {
            key.to_owned()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::PathBuf};

    use crate::package_json::{ExportsLikeField, FilenameOrConditional};

    use super::PackageJsonParser;

    const FAKE_MODULE_PATH: &str = "/fake/module/path";
    const FAKE_PACKAGE_NAME: &str = "fake-package-name";

    #[test]
    fn test_parse_exports_string() {
        let result = PackageJsonParser::parse_package_json_string(
            PathBuf::from(FAKE_MODULE_PATH),
            Some("fake-package-name".to_owned()),
            r#"{
                "name": "fake-package-name",
                "exports": "./index.js"
            }"#,
        );
        assert!(result.is_ok(), "{:?}", result);
        assert_eq!(
            result.unwrap().parsed_exports,
            Some(ExportsLikeField::Filename("./index.js".to_owned()),)
        );
    }

    #[test]
    fn test_parse_exports_names() {
        let result = PackageJsonParser::parse_package_json_string(
            PathBuf::from(FAKE_MODULE_PATH),
            Some("fake-package-name".to_owned()),
            r#"{
                "name": "fake-package-name",
                "exports": {
                    ".": "./index.js",
                    "./foo": "./foo.js",
                    "./bar": {
                        "./baz": "./bar/baz.js",
                        "./qux": "./bar/qux.js"
                    }
                }
            }"#,
        );
        assert!(result.is_ok(), "{:?}", result);
        assert_eq!(
            result.unwrap().parsed_exports,
            Some({
                let mut map = HashMap::new();
                map.insert(
                    FAKE_PACKAGE_NAME.to_owned(),
                    FilenameOrConditional::Filename("./index.js".to_owned()),
                );
                map.insert(
                    format!("{}/foo", FAKE_PACKAGE_NAME),
                    FilenameOrConditional::Filename("./foo.js".to_owned()),
                );
                map.insert(
                    format!("{}/bar/baz", FAKE_PACKAGE_NAME),
                    FilenameOrConditional::Filename("./bar/baz.js".to_owned()),
                );
                map.insert(
                    format!("{}/bar/qux", FAKE_PACKAGE_NAME),
                    FilenameOrConditional::Filename("./bar/qux.js".to_owned()),
                );
                ExportsLikeField::Map(map)
            })
        );
    }

    #[test]
    fn test_parse_exports_conditions() {
        let result = PackageJsonParser::parse_package_json_string(
            PathBuf::from(FAKE_MODULE_PATH),
            Some("fake-package-name".to_owned()),
            r#"{
                "name": "fake-package-name",
                "exports": {
                    ".": {
                        "import": "./index.js",
                        "require": "./index.cjs"
                    },
                    "./foo": {
                        "import": "./foo.js",
                        "require": "./foo.cjs"
                    }
                }
            }"#,
        );
        assert!(result.is_ok(), "{:?}", result);
        assert_eq!(
            result.unwrap().parsed_exports,
            Some({
                let mut map = HashMap::new();
                map.insert(
                    FAKE_PACKAGE_NAME.to_owned(),
                    FilenameOrConditional::Conditional({
                        let mut map = HashMap::new();
                        map.insert(
                            "import".to_owned(),
                            FilenameOrConditional::Filename("./index.js".to_owned()),
                        );
                        map.insert(
                            "require".to_owned(),
                            FilenameOrConditional::Filename("./index.cjs".to_owned()),
                        );
                        map
                    }),
                );
                map.insert(
                    format!("{}/foo", FAKE_PACKAGE_NAME),
                    FilenameOrConditional::Conditional({
                        let mut map = HashMap::new();
                        map.insert(
                            "import".to_owned(),
                            FilenameOrConditional::Filename("./foo.js".to_owned()),
                        );
                        map.insert(
                            "require".to_owned(),
                            FilenameOrConditional::Filename("./foo.cjs".to_owned()),
                        );
                        map
                    }),
                );
                ExportsLikeField::Map(map)
            })
        );
    }

    #[test]
    fn test_parse_exports_nested_conditions() {
        let result = PackageJsonParser::parse_package_json_string(
            PathBuf::from(FAKE_MODULE_PATH),
            Some("fake-package-name".to_owned()),
            r#"{
                "name": "fake-package-name",
                "exports": {
                    ".": "./index.js",
                    "./foo": {
                        "import": {
                            "types": "./foo.d.ts",
                            "default": "./foo.js"
                        },
                        "require": {
                            "types": "./foo.d.ts",
                            "default": "./foo.cjs"
                        }
                    }
                }
            }"#,
        );
        assert!(result.is_ok(), "{:?}", result);
        assert_eq!(
            result.unwrap().parsed_exports,
            Some({
                let mut map = HashMap::new();
                map.insert(
                    FAKE_PACKAGE_NAME.to_owned(),
                    FilenameOrConditional::Filename("./index.js".to_owned()),
                );
                map.insert(
                    format!("{}/foo", FAKE_PACKAGE_NAME),
                    FilenameOrConditional::Conditional({
                        let mut map = HashMap::new();
                        map.insert(
                            "import".to_owned(),
                            FilenameOrConditional::Conditional({
                                let mut map = HashMap::new();
                                map.insert(
                                    "types".to_owned(),
                                    FilenameOrConditional::Filename("./foo.d.ts".to_owned()),
                                );
                                map.insert(
                                    "default".to_owned(),
                                    FilenameOrConditional::Filename("./foo.js".to_owned()),
                                );
                                map
                            }),
                        );
                        map.insert(
                            "require".to_owned(),
                            FilenameOrConditional::Conditional({
                                let mut map = HashMap::new();
                                map.insert(
                                    "types".to_owned(),
                                    FilenameOrConditional::Filename("./foo.d.ts".to_owned()),
                                );
                                map.insert(
                                    "default".to_owned(),
                                    FilenameOrConditional::Filename("./foo.cjs".to_owned()),
                                );
                                map
                            }),
                        );
                        map
                    }),
                );
                ExportsLikeField::Map(map)
            })
        );
    }
}
