use serde::Deserialize;
use std::collections::HashMap;
use std::io;

#[derive(Debug, Deserialize)]
pub struct Config {
	pub project: Project,
	pub dependencies: Option<HashMap<String, Dependency>>,
}

#[derive(Debug, Deserialize)]
pub struct Project {
	pub name: String,
	pub version: String,
	pub sdk_edition: String,
	pub authors: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct Dependency {
	pub version: String,
}

pub fn load_config(content: String) -> Result<Config, io::Error> {
	toml::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}


