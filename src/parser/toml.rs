use serde::Deserialize;
use std::collections::HashMap;
use std::io;

#[derive(Debug, Deserialize)]
pub struct Config {
	pub project: Project,
	pub build: Build,
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
pub struct Build {
	#[serde(default)]
	pub package: PackageType,
	#[serde(default)]
	pub target: TargetType,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub enum PackageType {
	Debug,
	Release,
}

// 定义构建目标类型的枚举
#[derive(Debug, Deserialize, PartialEq, Clone)]
pub enum TargetType {
	App,
	Lib,
}

#[derive(Debug, Deserialize)]
pub struct Dependency {
	pub version: String,
}

impl Default for PackageType {
	fn default() -> Self {
		PackageType::Debug
	}
}

impl Default for TargetType {
	fn default() -> Self {
		TargetType::App
	}
}

pub fn load_config(content: String) -> Result<Config, io::Error> {
	toml::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}