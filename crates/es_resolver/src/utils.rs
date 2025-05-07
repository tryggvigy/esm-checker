//! Utility functions for the resolvers.

use std::{borrow::Cow, ffi::OsStr, path::PathBuf};

/// Given an import specifier, return the name of the package it belongs to.
///
/// # Examples
///
/// ```
/// use es_resolver::utils::get_npm_package_name;
///
/// assert_eq!(get_npm_package_name("foo"), "foo");
/// assert_eq!(get_npm_package_name("foo/bar"), "foo");
/// assert_eq!(get_npm_package_name("@foo/bar"), "@foo/bar");
/// assert_eq!(get_npm_package_name("@foo/bar/baz"), "@foo/bar");
/// ```
pub fn get_npm_package_name(import_specifier: &str) -> &str {
    let up_to_slash = if import_specifier.starts_with('@') {
        // Scoped package, find everything up to the second forward slash, or
        // the end of the string if there is only one.
        let mut slashes = 0;
        let mut second_slash = None;
        for (i, c) in import_specifier.char_indices() {
            if c == '/' {
                slashes += 1;
            }
            if slashes == 2 {
                second_slash = Some(i);
                break;
            }
        }

        second_slash
    } else {
        import_specifier.find('/')
    };

    match up_to_slash {
        Some(i) => &import_specifier[..i],
        None => import_specifier,
    }
}

/// A utility struct for resolving implicit files. This is used by the resolvers to
/// resolve import specifiers that don't have an extension or a file name. For
/// example, `import 'foo'` could resolve to `foo.js` or `foo/index.js` if it exists.
#[derive(Clone, Debug)]
pub struct ImplicitFileResolver<'a> {
    implicit_extensions: Vec<Cow<'a, str>>,
    implicit_indexes: Vec<Cow<'a, str>>,
}

impl<'a> ImplicitFileResolver<'a> {
    /// Create a new implicit file resolver with the given extensions and indexes.
    pub fn new(
        implicit_extensions: Vec<Cow<'a, str>>,
        implicit_indexes: Vec<Cow<'a, str>>,
    ) -> Self {
        Self {
            implicit_extensions,
            implicit_indexes,
        }
    }

    /// Try to resolve the given input path to a file, taking implicit file
    /// resolution into account. Will first try the input path using the given
    /// extensions, then try the input path using the given index filenames.
    pub fn try_resolve_implicitly(&self, mut path: PathBuf) -> Option<PathBuf> {
        let original_file_name = path.file_name().map(|name| name.to_owned());

        if let Some(original_file_name) = original_file_name {
            for implicit_extension in self.implicit_extensions.iter() {
                let mut with_extension = original_file_name.clone();
                with_extension.push(OsStr::new(implicit_extension.as_ref()));
                path.set_file_name(with_extension);
                if path.is_file() {
                    return Some(path);
                }
            }

            path.set_file_name(original_file_name);
        }

        for implicit_index in self.implicit_indexes.iter() {
            path.push(OsStr::new(implicit_index.as_ref()));
            if path.is_file() {
                return Some(path);
            }
            path.pop();
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::get_npm_package_name;
    #[test]
    fn npm_package_name() {
        assert_eq!("foo", get_npm_package_name("foo"));
        assert_eq!("foo", get_npm_package_name("foo/bar"));
        assert_eq!("foo", get_npm_package_name("foo/bar/baz.mjs"));
        assert_eq!("@foo/bar", get_npm_package_name("@foo/bar"));
        assert_eq!("@foo/bar", get_npm_package_name("@foo/bar/baz.mjs"));
    }
}
