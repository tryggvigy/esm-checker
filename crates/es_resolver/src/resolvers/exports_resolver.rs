use std::{
    borrow::Cow,
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    package_json::{ExportsLikeField, FilenameOrConditional, PackageJson},
    resolve_chain::{ChainStep, ResolveStepResult},
    utils::ImplicitFileResolver,
};

/// The name of the field that is being resolved by the [`ExportsResolver`]. Refers to the fields
/// of the same name in `package.json`.
#[derive(Debug, PartialEq)]
pub enum FieldName {
    /// The `browser` field.
    Browser,
    /// The `exports` field.
    Exports,
    /// The `main` field.
    Main,
    /// The `module` field.
    Module,
    /// The `types` field.
    Types,
}

/// Resolver that handles the `exports`-like fields in package.json.
/// Is also able to deal with `browser`, `main` and `module` fields, which may or may not use the
/// same tree-like structure of the `exports` field. Note that if the `package.json` contains
/// an `exports` field, all other fields will be ignored by this resolver:
/// <https://nodejs.org/api/packages.html#package-entry-points>
#[derive(Debug)]
pub struct ExportsResolver<'a> {
    field_name: FieldName,
    condition_names: Vec<Cow<'a, str>>,
    implicit_file_resolver: Option<ImplicitFileResolver<'a>>,
}

#[derive(Debug, PartialEq)]
enum MatchedExport<'a> {
    Filename(&'a str),
    FilenameWithPlaceholders(&'a str, Vec<&'a str>),
    Conditional(&'a HashMap<String, FilenameOrConditional>),
    ConditionalWithPlaceholders(&'a HashMap<String, FilenameOrConditional>, Vec<&'a str>),
}

impl<'a> ExportsResolver<'a> {
    /// Create a new [`ExportsResolver`]. `condition_names` is a the list of condition names that
    /// should be checked when resolving the exports, in the order that they will be checked.
    ///
    /// # Example
    ///
    /// ```
    /// use es_resolver::resolvers::{ExportsResolver, FieldName};
    ///
    /// let resolver = ExportsResolver::new(
    ///    FieldName::Exports,
    ///    vec!["import".into(), "require".into(), "default".into()],
    ///    None,
    /// );
    /// ```
    ///
    /// When resolving the import specifier `foo/bar`, and the `package.json` for `foo` contains:
    ///
    /// ```json
    /// {
    ///   "exports": {
    ///     "./bar": {
    ///       "require": "./bar.cjs",
    ///       "default": "./bar.js"
    ///     }
    ///   }
    /// }
    /// ```
    ///
    /// Then given the condition names above, it will resolve to `foo/bar.cjs`, as there is no
    /// `import` condition, and `require` is the first condition name that exists in the exports.
    pub fn new(
        field_name: FieldName,
        condition_names: Vec<Cow<'a, str>>,
        implicit_file_resolver: Option<ImplicitFileResolver<'a>>,
    ) -> Self {
        Self {
            field_name,
            condition_names,
            implicit_file_resolver,
        }
    }

    fn resolve_export(&self, entry: MatchedExport<'_>, package_root: &Path) -> Option<PathBuf> {
        match entry {
            MatchedExport::Filename(filename) => Some(package_root.join(filename)),
            MatchedExport::FilenameWithPlaceholders(filename, placeholders) => {
                Some(package_root.join(Self::replace_placeholders(filename, &placeholders)))
            }
            MatchedExport::Conditional(map) => self.resolve_condition_name(map, package_root, None),
            MatchedExport::ConditionalWithPlaceholders(map, placeholders) => {
                self.resolve_condition_name(map, package_root, Some(&placeholders))
            }
        }
    }

    fn resolve_condition_name(
        &self,
        map: &HashMap<String, FilenameOrConditional>,
        package_root: &Path,
        placeholders: Option<&[&str]>,
    ) -> Option<PathBuf> {
        for condition_name in self.condition_names.iter() {
            if let Some(value) = map.get(condition_name.as_ref()) {
                match value {
                    FilenameOrConditional::Filename(filename) => {
                        return if let Some(placeholders) = placeholders {
                            Some(
                                package_root
                                    .join(Self::replace_placeholders(filename, placeholders)),
                            )
                        } else {
                            Some(package_root.join(filename))
                        }
                    }
                    FilenameOrConditional::Conditional(map) => {
                        let path = self.resolve_condition_name(map, package_root, placeholders);
                        if path.is_some() {
                            return path;
                        }
                    }
                }
            }
        }

        None
    }

    fn match_export<'m>(
        map: &'m HashMap<String, FilenameOrConditional>,
        import_specifier: &'m str,
    ) -> Option<MatchedExport<'m>> {
        match map.get(import_specifier) {
            Some(FilenameOrConditional::Filename(filename)) => {
                return Some(MatchedExport::Filename(filename))
            }
            Some(FilenameOrConditional::Conditional(map)) => {
                return Some(MatchedExport::Conditional(map))
            }
            None => {
                // Iterate through the map to see if any of the keys match the import specifier,
                // taking wildcards into account. For example, if the import specifier is
                // `foo/bar`, and the map contains the key `foo/*`, then the value for that key
                // will be returned. Note that the wildcard may appear anywhere in the key, not
                // just at the end.
                'outer: for (key, value) in map.iter() {
                    if key.contains('*') {
                        // Just split the key on the wildcard, and check that the import specifier
                        // contains each expected part. Keep track of the captures (the parts of
                        // the import specifier that correspond to the wildcard parts of the key),
                        // and return the value for the key if there is a match.
                        let mut import_specifier_remaining = import_specifier;
                        let mut captures: Vec<&str> = Vec::new();
                        let mut ended_with_wildcard = false;
                        for (i, key_part) in key.split('*').enumerate() {
                            ended_with_wildcard = key_part.is_empty();
                            if i == 0 {
                                if !import_specifier_remaining.starts_with(key_part) {
                                    break;
                                }

                                import_specifier_remaining =
                                    &import_specifier_remaining[key_part.len()..];
                            } else if let Some(index) = import_specifier_remaining.find(key_part) {
                                captures.push(&import_specifier_remaining[..index]);
                                import_specifier_remaining =
                                    &import_specifier_remaining[index + key_part.len()..];
                            } else {
                                // No match
                                continue 'outer;
                            }
                        }

                        // If the key ended with a wildcard, then capture the rest of the import
                        // specifier.
                        if ended_with_wildcard {
                            captures.push(import_specifier_remaining);
                            import_specifier_remaining = "";
                        }

                        // If there are no more parts of the import specifier remaining, then we
                        // have a match. Now we need to replace the wildcard captures in the value
                        // with the corresponding parts of the import specifier.
                        if import_specifier_remaining.is_empty() {
                            return Some(match value {
                                // Simple case: no placeholders in string value.
                                FilenameOrConditional::Filename(s) if !s.contains('*') => {
                                    MatchedExport::Filename(s)
                                }
                                // Replace placeholders in string value.
                                FilenameOrConditional::Filename(s) => {
                                    MatchedExport::FilenameWithPlaceholders(s, captures)
                                }
                                FilenameOrConditional::Conditional(m) => {
                                    // If there are no placeholders in the map values, then we can
                                    // just return the map as-is.
                                    let any_placeholders =
                                        Self::any_placeholders_in_map_values(map);
                                    if !any_placeholders {
                                        MatchedExport::Conditional(m)
                                    } else {
                                        // Otherwise, we need to replace the placeholders in the
                                        // map values, recursively.
                                        MatchedExport::ConditionalWithPlaceholders(m, captures)
                                    }
                                }
                            });
                        }
                    }
                }
            }
        }

        None
    }

    fn replace_placeholders(str: &str, captures: &[&str]) -> String {
        let mut result = str.to_string();
        for capture in captures.iter() {
            result = result.replacen('*', capture, 1)
        }
        result
    }

    fn any_placeholders_in_map_values(map: &HashMap<String, FilenameOrConditional>) -> bool {
        map.values().any(|v| match v {
            FilenameOrConditional::Filename(s) => s.contains('*'),
            FilenameOrConditional::Conditional(m) => Self::any_placeholders_in_map_values(m),
        })
    }
}

impl<'a> ChainStep<Arc<PackageJson>, Arc<PackageJson>> for ExportsResolver<'a> {
    fn call(
        &self,
        import_specifier: String,
        _from: &Path,
        state: Arc<PackageJson>,
    ) -> ResolveStepResult<Arc<PackageJson>> {
        // If the `package.json` contains an `exports` field, all other fields are ignored.
        if self.field_name != FieldName::Exports && state.parsed_exports.is_some() {
            return ResolveStepResult::Continue(import_specifier, state);
        }

        if let Some(field) = match self.field_name {
            FieldName::Exports => state.parsed_exports.as_ref(),
            FieldName::Main => state.parsed_main.as_ref(),
            FieldName::Module => state.parsed_module.as_ref(),
            FieldName::Browser => state.parsed_browser.as_ref(),
            FieldName::Types => state.parsed_types.as_ref(),
        } {
            if let Some(entry) = match field {
                ExportsLikeField::Filename(f)
                    if state
                        .name
                        .as_ref()
                        .map(|name| name == &import_specifier)
                        .unwrap_or(false) =>
                {
                    Some(MatchedExport::Filename(f))
                }
                ExportsLikeField::Conditional(c)
                    if state
                        .name
                        .as_ref()
                        .map(|name| name == &import_specifier)
                        .unwrap_or(false) =>
                {
                    Some(MatchedExport::Conditional(c))
                }
                ExportsLikeField::Filename(_) | ExportsLikeField::Conditional(_) => None,
                ExportsLikeField::Map(m) => Self::match_export(m, &import_specifier),
            } {
                if let Some(path) = self.resolve_export(entry, state.package_root.as_path()) {
                    if path.is_file() {
                        return ResolveStepResult::Ok(path);
                    }
                    if let Some(implicit_file_resolver) = &self.implicit_file_resolver {
                        if let Some(path) = implicit_file_resolver.try_resolve_implicitly(path) {
                            return ResolveStepResult::Ok(path);
                        }
                    }
                }
            }
        }

        ResolveStepResult::Continue(import_specifier, state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcard_suffix() {
        // suffix mapping to single file
        assert_eq!(
            ExportsResolver::match_export(
                &{
                    let mut map = HashMap::new();
                    map.insert(
                        "foo/*".to_string(),
                        FilenameOrConditional::Filename("bar".to_string()),
                    );
                    map
                },
                "foo/bar"
            ),
            Some(MatchedExport::Filename("bar"))
        );
    }

    #[test]
    fn wildcard_no_match() {
        // missing extension in import specifier
        assert_eq!(
            ExportsResolver::match_export(
                &{
                    let mut map = HashMap::new();
                    map.insert(
                        "foo/*.js".to_string(),
                        FilenameOrConditional::Filename("bar".to_string()),
                    );
                    map
                },
                "foo/baz"
            ),
            None
        );

        // incorrect prefix in import specifier
        assert_eq!(
            ExportsResolver::match_export(
                &{
                    let mut map = HashMap::new();
                    map.insert(
                        "foo/*.js".to_string(),
                        FilenameOrConditional::Filename("bar".to_string()),
                    );
                    map
                },
                "baz/qux.js"
            ),
            None
        );
    }

    #[test]
    fn wildcard_infix_with_value_pattern() {
        // infix mapping to pattern
        assert_eq!(
            ExportsResolver::match_export(
                &{
                    let mut map = HashMap::new();
                    map.insert(
                        "foo/*.js".to_string(),
                        FilenameOrConditional::Filename("bar/*.js".to_string()),
                    );
                    map
                },
                "foo/baz.js"
            ),
            Some(MatchedExport::FilenameWithPlaceholders(
                "bar/*.js",
                vec!["baz"]
            ))
        );
    }

    #[test]
    fn multiple_wildcards() {
        // multiple wildcards
        assert_eq!(
            ExportsResolver::match_export(
                &{
                    let mut map = HashMap::new();
                    map.insert(
                        "foo/*/baz/*.js".to_string(),
                        FilenameOrConditional::Filename("bar/*/qux/*.js".to_string()),
                    );
                    map
                },
                "foo/one/baz/two.js"
            ),
            Some(MatchedExport::FilenameWithPlaceholders(
                "bar/*/qux/*.js",
                vec!["one", "two"]
            ))
        );
    }

    #[test]
    fn wildcard_with_condition_names() {
        // condition names with placeholders
        assert_eq!(
            ExportsResolver::match_export(
                &{
                    let mut map = HashMap::new();
                    map.insert(
                        "foo/*.js".to_string(),
                        FilenameOrConditional::Conditional({
                            let mut map = HashMap::new();
                            map.insert(
                                "node".to_string(),
                                FilenameOrConditional::Conditional({
                                    let mut map = HashMap::new();
                                    map.insert(
                                        "import".to_string(),
                                        FilenameOrConditional::Filename("bar/*.mjs".to_string()),
                                    );
                                    map.insert(
                                        "default".to_string(),
                                        FilenameOrConditional::Filename("bar/*.js".to_string()),
                                    );
                                    map
                                }),
                            );
                            map.insert(
                                "default".to_string(),
                                FilenameOrConditional::Filename("qux/*.js".to_string()),
                            );
                            map
                        }),
                    );
                    map
                },
                "foo/baz.js"
            ),
            Some(MatchedExport::ConditionalWithPlaceholders(
                &{
                    let mut map = HashMap::new();
                    map.insert(
                        "node".to_string(),
                        FilenameOrConditional::Conditional({
                            let mut map = HashMap::new();
                            map.insert(
                                "import".to_string(),
                                FilenameOrConditional::Filename("bar/*.mjs".to_string()),
                            );
                            map.insert(
                                "default".to_string(),
                                FilenameOrConditional::Filename("bar/*.js".to_string()),
                            );
                            map
                        }),
                    );
                    map.insert(
                        "default".to_string(),
                        FilenameOrConditional::Filename("qux/*.js".to_string()),
                    );
                    map
                },
                vec!["baz"]
            ))
        );

        // condition names without placeholders
        assert_eq!(
            ExportsResolver::match_export(
                &{
                    let mut map = HashMap::new();
                    map.insert(
                        "foo/*.js".to_string(),
                        FilenameOrConditional::Conditional({
                            let mut map = HashMap::new();
                            map.insert(
                                "import".to_string(),
                                FilenameOrConditional::Filename("qux/import.js".to_string()),
                            );
                            map.insert(
                                "default".to_string(),
                                FilenameOrConditional::Filename("bar/default.js".to_string()),
                            );
                            map
                        }),
                    );
                    map
                },
                "foo/bar.js"
            ),
            Some(MatchedExport::Conditional(&{
                let mut map = HashMap::new();
                map.insert(
                    "import".to_string(),
                    FilenameOrConditional::Filename("qux/import.js".to_string()),
                );
                map.insert(
                    "default".to_string(),
                    FilenameOrConditional::Filename("bar/default.js".to_string()),
                );
                map
            }))
        );
    }
}
