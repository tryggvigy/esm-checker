use is_esm_ready_yet::generate_report::generate_report;
use napi_derive::napi;

#[napi]
pub fn generate_report_js(
    package_json_location: String,
    check: Option<Vec<String>>,
) -> napi::Result<String> {
    let report = generate_report(&package_json_location, check)
        .map_err(|e| napi::Error::from_reason(format!("Failed to generate report: {}", e)))?;

    serde_json::to_string(&report)
        .map_err(|e| napi::Error::from_reason(format!("Failed to serialize report: {}", e)))
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
