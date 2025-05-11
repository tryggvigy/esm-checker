use crate::prelude::*;
use std::path::PathBuf;

#[test]
fn relative() {
    let mut fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fixtures.push("fixtures");

    let expected = fixtures.join("foo.mjs");
    let actual = crate::presets::get_default_es_resolver()
        .resolve("./foo.mjs".to_string(), &fixtures.join("index.mjs"))
        .unwrap();

    // Canonincalize paths to avoid windows extended-length path prefix
    assert_eq!(
        actual.canonicalize().unwrap(),
        expected.canonicalize().unwrap()
    );
}
