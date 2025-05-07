use std::{borrow::Cow, collections::HashMap, path::PathBuf};

use crate::{errors::ResolveError, prelude::Resolve};

/// A parsed `package.json` file.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawPackageJson {
    /// The name of the package.
    pub name: Option<String>,
    /// <https://nodejs.org/dist/latest-v18.x/docs/api/packages.html#exports>
    pub exports: Option<serde_json::Value>,
    /// <https://docs.npmjs.com/cli/v9/configuring-npm/package-json#files>
    pub files: Option<Vec<String>>,
    /// <https://nodejs.org/dist/latest-v18.x/docs/api/packages.html#main>
    pub main: Option<serde_json::Value>,
    /// <https://docs.npmjs.com/cli/v9/configuring-npm/package-json#browser>
    pub browser: Option<serde_json::Value>,
    /// Like `main` and `browser`, but for ESM.
    pub module: Option<serde_json::Value>,
    /// Like `main`, `browser`, and `module`, but for type definitions.
    pub types: Option<serde_json::Value>,
    /// <https://docs.npmjs.com/cli/v9/configuring-npm/package-json#peerdependencies>
    pub peer_dependencies: Option<HashMap<String, String>>,
    /// <https://docs.npmjs.com/cli/v9/configuring-npm/package-json#peerdependenciesmeta>
    pub peer_dependencies_meta: Option<HashMap<String, PeerDependencyMeta>>,
}

/// The value of a `peerDependenciesMeta` field in a `package.json` file.
#[derive(Debug, serde::Deserialize)]
pub struct PeerDependencyMeta {
    /// Whether the peer dependency is optional.
    pub optional: bool,
}

/// The value of an `exports` field, or similar, in a `package.json` file.
#[derive(Clone, Debug, PartialEq)]
pub enum ExportsLikeField {
    /// A simple string value, denoting the file that is the entrypoint.
    Filename(String),
    /// A map where the keys are entrypoint names, and the values are either filenames or
    /// conditionals.
    Map(HashMap<String, FilenameOrConditional>),
    /// A map of condition names, e.g. `default`, `import`, `module`, etc., to either filenames or
    /// more conditionals.
    Conditional(HashMap<String, FilenameOrConditional>),
}

/// The value of a filename or a conditional mapping, in an `exports` field, or similar, in a
/// `package.json` file.
#[derive(Clone, Debug, PartialEq)]
pub enum FilenameOrConditional {
    /// A simple string value, denoting the file that is the entrypoint.
    Filename(String),
    /// A map of condition names, e.g. `default`, `import`, `module`, etc., to either filenames or
    /// more conditionals.
    Conditional(HashMap<String, FilenameOrConditional>),
}

/// A parsed `package.json` file, with the `exports`, `main`, `module`, and `browser` fields parsed
/// into a [`StringOrMap`]. Also contains the path to the package root.
#[derive(Debug)]
pub struct PackageJson {
    /// The name of the package.
    pub name: Option<String>,
    /// The path to the package root (the directory that contains the `package.json` file).
    pub package_root: PathBuf,
    /// The raw (parsed) `package.json` file.
    pub raw: RawPackageJson,
    /// The parsed and normalized `exports` field.
    pub parsed_exports: Option<ExportsLikeField>,
    /// The parsed and normalized `main` field.
    pub parsed_main: Option<ExportsLikeField>,
    /// The parsed and normalized `module` field.
    pub parsed_module: Option<ExportsLikeField>,
    /// The parsed and normalized `browser` field.
    pub parsed_browser: Option<ExportsLikeField>,
    /// The parsed and normalized `types` field.
    pub parsed_types: Option<ExportsLikeField>,
}

impl PackageJson {
    /// Get the detected entrypoints (files) for this package.
    pub fn get_entrypoints(
        &self,
        condition_names: &[Cow<str>],
        resolver: &impl Resolve,
    ) -> Result<Vec<PathBuf>, ResolveError> {
        if let Some(exports) = &self.parsed_exports {
            match exports {
                ExportsLikeField::Filename(filename) => Ok(vec![self
                    .package_root
                    .join(filename)
                    .canonicalize()
                    .unwrap()]),
                ExportsLikeField::Map(map) => Ok(map
                    .values()
                    .filter_map(|v| match v {
                        FilenameOrConditional::Filename(filename) if !filename.contains('*') => {
                            Some(self.package_root.join(filename).canonicalize().unwrap())
                        }
                        FilenameOrConditional::Filename(_) => None,
                        FilenameOrConditional::Conditional(conditional) => {
                            self.pick_conditional_entrypoint(condition_names, conditional)
                        }
                    })
                    .collect()),
                ExportsLikeField::Conditional(conditional) => Ok(self
                    .pick_conditional_entrypoint(condition_names, conditional)
                    .into_iter()
                    .collect()),
            }
        } else if let Some(name) = &self.name {
            Ok(vec![resolver.resolve(name.clone(), &self.package_root)?])
        } else {
            log::trace!(
                "Could not find an entrypoint for package {} and package.json {:?}",
                self.name.as_ref().unwrap_or(&"unknown".to_owned()),
                self
            );
            Err(ResolveError::FailedToResolve(
                self.name
                    .as_ref()
                    .cloned()
                    .unwrap_or("<unknown>".to_owned()),
                self.package_root.clone(),
            ))
        }
    }

    fn pick_conditional_entrypoint(
        &self,
        condition_names: &[Cow<str>],
        conditional: &HashMap<String, FilenameOrConditional>,
    ) -> Option<PathBuf> {
        for condition_name in condition_names {
            if let Some(entrypoint) = conditional.get(condition_name.as_ref()) {
                match entrypoint {
                    FilenameOrConditional::Filename(filename) => {
                        if !filename.contains('*') {
                            return Some(self.package_root.join(filename).canonicalize().unwrap());
                        }
                    }
                    FilenameOrConditional::Conditional(conditional) => {
                        return self.pick_conditional_entrypoint(condition_names, conditional);
                    }
                };
            }
        }

        log::trace!(
            "Could not find an entrypoint for package {} in conditional {:?} with condition names {:?}",
            self.name.as_ref().unwrap_or(&"unknown".to_owned()),
            conditional,
            condition_names
        );
        None
    }
}
