//! Preset resolvers for common use cases.

use std::borrow::Cow;
use std::sync::Arc;

use crate::prelude::*;
use crate::resolvers::*;
use crate::utils::ImplicitFileResolver;
use crate::{
    package_json::PackageJsonParser, resolve_chain::new_chain, resolve_chain_container::Resolver,
};

/// Get a default [`Resolve`] implementation that should be able to resolve most ES module imports.
/// This function returns the same resolver as [`get_default_es_resolver_with_package_json_parser`],
/// but will create a [`PackageJsonParser`] for you. The other function allows you to pass in your
/// own [`PackageJsonParser`]
pub fn get_default_es_resolver() -> impl Resolve {
    let package_json_parser = Arc::new(PackageJsonParser::new());
    get_default_es_resolver_with_package_json_parser(package_json_parser)
}

/// Get a default [`Resolve`] implementation that should be able to resolve most ES module imports.
/// This function allows you to pass in your own [`PackageJsonParser`]. If you don't have one, you
/// can use [`get_default_es_resolver`], which will create one for you.
///
/// The resolver returned by this function will use the following resolvers, in order:
/// - [`RelativePathResolver`]
/// - [`HandleOptionalPeerDependenciesResolver`]
/// - [`PackageJsonResolver`]
/// - [`PseudoNamespaceResolver`]
/// - [`ExportsResolver`] (with `exports` as the field name)
/// - [`ExportsResolver`] (with `browser` as the field name)
/// - [`ExportsResolver`] (with `module` as the field name)
/// - [`ExportsResolver`] (with `default` as the field name)
/// - [`files_resolver`]
/// - [`index_resolver`]
/// - [`FileResolver`]
pub fn get_default_es_resolver_with_package_json_parser(
    package_json_parser: Arc<PackageJsonParser>,
) -> impl Resolve {
    let condition_names = get_default_condition_names();
    let implicit_file_resolver = Some(ImplicitFileResolver::new(
        vec![".js".into(), ".cjs".into(), ".json".into()],
        vec!["index.js".into(), "index.cjs".into(), "index.json".into()],
    ));

    Resolver::new(
        new_chain
            .chain(RelativePathResolver::new(
                Arc::clone(&package_json_parser),
                implicit_file_resolver.clone(),
            ))
            .chain(HandleOptionalPeerDependenciesResolver::new(Arc::clone(
                &package_json_parser,
            )))
            .chain(PackageJsonResolver::new(Arc::clone(&package_json_parser)))
            .chain(PseudoNamespaceResolver::new(Arc::clone(
                &package_json_parser,
            )))
            .chain(ExportsResolver::new(
                FieldName::Exports,
                condition_names.clone(),
                implicit_file_resolver.clone(),
            ))
            .chain(ExportsResolver::new(
                FieldName::Module,
                condition_names.clone(),
                implicit_file_resolver.clone(),
            ))
            .chain(ExportsResolver::new(
                FieldName::Browser,
                condition_names.clone(),
                implicit_file_resolver.clone(),
            ))
            .chain(ExportsResolver::new(
                FieldName::Main,
                condition_names,
                implicit_file_resolver.clone(),
            ))
            .chain(files_resolver as ResolveFunction<_, _>)
            .chain(index_resolver as ResolveFunction<_, _>)
            .chain(FileResolver::new(implicit_file_resolver)),
    )
}

/// Gets a [`Resolve`] implementation, similar to the one returned by [`get_default_es_resolver`],
/// but with support for TypeScript files.
pub fn get_typescript_resolver() -> impl Resolve {
    let package_json_parser = Arc::new(PackageJsonParser::new());
    get_typescript_resolver_with_package_json_parser(package_json_parser)
}

/// Gets a [`Resolve`] implementation, similar to the one returned by [`get_default_es_resolver`],
/// but with support for TypeScript files. Allows you to pass in your own [`PackageJsonParser`].
pub fn get_typescript_resolver_with_package_json_parser(
    package_json_parser: Arc<PackageJsonParser>,
) -> impl Resolve {
    let condition_names = vec![
        "import".into(),
        "module".into(),
        "default".into(),
        "types".into(),
    ];
    let implicit_file_resolver = Some(ImplicitFileResolver::new(
        vec![
            ".js".into(),
            ".cjs".into(),
            ".json".into(),
            ".ts".into(),
            ".tsx".into(),
            ".d.ts".into(),
        ],
        vec![
            "index.js".into(),
            "index.cjs".into(),
            "index.json".into(),
            "index.ts".into(),
            "index.tsx".into(),
            "index.d.ts".into(),
        ],
    ));

    Resolver::new(
        new_chain
            .chain(RelativePathResolver::new(
                Arc::clone(&package_json_parser),
                implicit_file_resolver.clone(),
            ))
            .chain(HandleOptionalPeerDependenciesResolver::new(Arc::clone(
                &package_json_parser,
            )))
            .chain(PackageJsonResolver::new(Arc::clone(&package_json_parser)))
            .chain(PseudoNamespaceResolver::new(package_json_parser))
            .chain(ExportsResolver::new(
                FieldName::Exports,
                condition_names.clone(),
                implicit_file_resolver.clone(),
            ))
            .chain(ExportsResolver::new(
                FieldName::Module,
                condition_names.clone(),
                implicit_file_resolver.clone(),
            ))
            .chain(ExportsResolver::new(
                FieldName::Browser,
                condition_names.clone(),
                implicit_file_resolver.clone(),
            ))
            .chain(ExportsResolver::new(
                FieldName::Main,
                condition_names.clone(),
                implicit_file_resolver.clone(),
            ))
            .chain(ExportsResolver::new(
                FieldName::Types,
                condition_names,
                implicit_file_resolver.clone(),
            ))
            .chain(files_resolver as ResolveFunction<_, _>)
            .chain(index_resolver as ResolveFunction<_, _>)
            .chain(FileResolver::new(implicit_file_resolver)),
    )
}

/// Gets a [`Resolve`] implementation, similar to the one returned by [`get_default_es_resolver`],
/// but that only strictly follows the ES Module resolution algorithm. In other words, it does not
/// implicitly resolve to `.js` or `.json` files, and it only resolves using relative paths, and
/// using a `package.json`'s `exports` field for the condition names `import` and `default`, and
/// the `module` field.
pub fn get_strict_esm_resolver() -> impl Resolve {
    let package_json_parser = Arc::new(PackageJsonParser::new());
    get_strict_esm_resolver_with_package_json_parser(package_json_parser)
}

/// Like [`get_strict_esm_resolver`], but allows you to pass in your own [`PackageJsonParser`].
pub fn get_strict_esm_resolver_with_package_json_parser(
    package_json_parser: Arc<PackageJsonParser>,
) -> impl Resolve {
    let condition_names = vec!["import".into(), "default".into()];

    Resolver::new(
        new_chain
            .chain(RelativePathResolver::new(
                Arc::clone(&package_json_parser),
                None,
            ))
            .chain(PackageJsonResolver::new(package_json_parser))
            .chain(ExportsResolver::new(
                FieldName::Exports,
                condition_names.clone(),
                None,
            ))
            .chain(ExportsResolver::new(
                FieldName::Module,
                condition_names.clone(),
                None,
            ))
            .chain(ExportsResolver::new(
                FieldName::Main,
                condition_names.clone(),
                None,
            ))
            .chain(files_resolver as ResolveFunction<_, _>)
            .chain(FileResolver::new(None)),
    )
}

/// Get the ordered default condition names for the `exports` field.
pub fn get_default_condition_names() -> Vec<Cow<'static, str>> {
    vec!["import".into(), "module".into(), "default".into()]
}
