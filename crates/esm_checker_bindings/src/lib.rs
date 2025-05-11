use napi_derive::napi;
use report_model::Report as RustReport;
use reporter::generate_report::generate_report as generate_report_rust;

#[napi(object)]
pub struct WithCommonJSDependencies {
    pub package_name: String,
    pub transitive_commonjs_dependencies: Vec<String>,
}

#[napi(object)]
pub struct WithMissingJsFileExtensions {
    pub package_name: String,
    pub transitive_deps_with_missing_js_file_extensions: Vec<String>,
}

#[napi(object)]
pub struct FauxESM {
    pub with_commonjs_dependencies: Vec<WithCommonJSDependencies>,
    pub with_missing_js_file_extensions: Vec<WithMissingJsFileExtensions>,
}

#[napi(object)]
pub struct ResolveError {
    pub package_name: String,
    pub from: String,
    pub import_specifier: String,
    pub original_error_message: String,
}

#[napi(object)]
pub struct ParseError {
    pub package_name: String,
    pub path: String,
    pub original_error_message: String,
}

#[napi(object)]
pub struct Report {
    pub total: u32,
    pub esm: Vec<String>,
    pub cjs: Vec<String>,
    pub faux_esm: FauxESM,
    pub resolve_errors: Vec<ResolveError>,
    pub parse_errors: Vec<ParseError>,
}

impl From<RustReport> for Report {
    fn from(report: RustReport) -> Self {
        Report {
            total: report.total as u32,
            esm: report.esm,
            cjs: report.cjs,
            faux_esm: FauxESM {
                with_commonjs_dependencies: report
                    .faux_esm
                    .with_commonjs_dependencies
                    .into_iter()
                    .map(|d| WithCommonJSDependencies {
                        package_name: d.package_name,
                        transitive_commonjs_dependencies: d
                            .transitive_commonjs_dependencies
                            .into_iter()
                            .collect(),
                    })
                    .collect(),
                with_missing_js_file_extensions: report
                    .faux_esm
                    .with_missing_js_file_extensions
                    .into_iter()
                    .map(|d| WithMissingJsFileExtensions {
                        package_name: d.package_name,
                        transitive_deps_with_missing_js_file_extensions: d
                            .transitive_deps_with_missing_js_file_extensions
                            .into_iter()
                            .collect(),
                    })
                    .collect(),
            },
            resolve_errors: report
                .resolve_errors
                .into_iter()
                .map(|e| ResolveError {
                    package_name: e.package_name,
                    from: e.from.to_string_lossy().into_owned(),
                    import_specifier: e.import_specifier,
                    original_error_message: e.original_error_message,
                })
                .collect(),
            parse_errors: report
                .parse_errors
                .into_iter()
                .map(|e| ParseError {
                    package_name: e.package_name,
                    path: e.path.to_string_lossy().into_owned(),
                    original_error_message: e.original_error_message,
                })
                .collect(),
        }
    }
}

#[napi]
pub fn generate_report(
    package_json_location: String,
    check: Option<Vec<String>>,
) -> napi::Result<Report> {
    let report = generate_report_rust(&package_json_location, check)
        .map_err(|e| napi::Error::from_reason(format!("Failed to generate report: {}", e)))?;

    Ok(Report::from(report))
}
