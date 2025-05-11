use crate::analyze::{types::AnalysisError, Analysis};
use report_model::{
    ParseError, Report, ResolveError, WithCommonJSDependencies, WithMissingJsFileExtensions,
};

pub fn into_report(analyses: Vec<Result<Analysis, AnalysisError>>) -> Report {
    let mut report = Report {
        total: analyses.len(),
        ..Default::default()
    };

    for analysis in analyses {
        match analysis {
            Ok(analysis) => {
                let has_cjs_dependencies = !analysis.transitive_commonjs_dependencies.is_empty();
                let has_missing_js_file_extensions =
                    !analysis.esm_missing_js_file_extensions.is_empty();

                // Faux-ESM. **Note** a dependency can have _both_ transitive cjs deps and
                // missing file extensions but we report it only as having transitive cjs in
                // this case. This avoids reporting the same dependency twice in the output.
                if analysis.is_entry_esm && has_cjs_dependencies {
                    report
                        .faux_esm
                        .with_commonjs_dependencies
                        .push(WithCommonJSDependencies {
                            package_name: analysis.package_name,
                            transitive_commonjs_dependencies: analysis
                                .transitive_commonjs_dependencies,
                        });
                    continue;
                }

                if analysis.is_entry_esm && has_missing_js_file_extensions {
                    report.faux_esm.with_missing_js_file_extensions.push(
                        WithMissingJsFileExtensions {
                            package_name: analysis.package_name,
                            transitive_deps_with_missing_js_file_extensions: analysis
                                .esm_missing_js_file_extensions,
                        },
                    );
                    continue;
                }

                // True ESM
                if analysis.is_entry_esm {
                    report.esm.push(analysis.package_name);
                    continue;
                }

                report.cjs.push(analysis.package_name);
            }
            Err(err) => match err {
                AnalysisError::ResolveError {
                    package_name,
                    import_specifier,
                    from,
                    source,
                } => report.resolve_errors.push(ResolveError {
                    package_name,
                    import_specifier,
                    from,
                    original_error_message: source.to_string(),
                }),
                AnalysisError::ParseError {
                    package_name,
                    path,
                    original_error_message,
                } => report.parse_errors.push(ParseError {
                    package_name,
                    path,
                    original_error_message,
                }),
            },
        }
    }

    report.esm.sort();
    report.cjs.sort();
    report.faux_esm.with_commonjs_dependencies.sort_by(|a, b| {
        a.package_name
            .to_lowercase()
            .cmp(&b.package_name.to_lowercase())
    });
    report
        .faux_esm
        .with_missing_js_file_extensions
        .sort_by(|a, b| {
            a.package_name
                .to_lowercase()
                .cmp(&b.package_name.to_lowercase())
        });
    report.parse_errors.sort_by(|a, b| {
        a.package_name
            .to_lowercase()
            .cmp(&b.package_name.to_lowercase())
    });

    report
}
