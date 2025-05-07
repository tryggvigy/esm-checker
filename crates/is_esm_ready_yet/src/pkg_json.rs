use serde::Deserialize;
use serde_json::{Map, Value};
use std::{error::Error, fs::File, path::PathBuf};

#[derive(Deserialize, Debug)]
pub struct PackageJson {
    pub name: String,
    pub dependencies: Map<String, Value>,
}

impl PackageJson {
    pub fn load(path: &PathBuf) -> Result<PackageJson, Box<dyn Error>> {
        let pkg_json_file = File::open(path)?;
        let parsed_json: PackageJson = serde_json::from_reader(pkg_json_file)?;
        Ok(parsed_json)
    }
}
