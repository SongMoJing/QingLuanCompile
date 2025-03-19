use crate::_lib::io::{Log, LogType};
use crate::parser::toml::DependencyType::{Exclude, Include, IncludeAll};
use crate::PACKAGE_MANAGER;
use colored::Colorize;
use std::process::{Command, Stdio};
use toml::Value;

pub fn parser_config(content: String) -> ProjectConfig {
	let value: Value = content.parse::<Value>().unwrap_or_else(|_| {
		Log::new(LogType::Err, "QingLuan.toml 文件解析失败，请检查文件格式。").throw(23);
	});

	let mut project: Project = Default::default();
	if let Some(table) = value.get("project").and_then(Value::as_table) {
		if let Some(name) = table.get("name").and_then(Value::as_str) {
			project.name = name.to_string();
		} else {
			Log::new(LogType::Err, "QingLuan.toml 文件解析失败：project缺少name键，请检查文件格式。").throw(23);
		}
		if let Some(version) = table.get("version").and_then(Value::as_str) {
			project.version = version.to_string();
		} else {
			Log::new(LogType::Err, "QingLuan.toml 文件解析失败：project缺少version键，请检查文件格式。").throw(23);
		}
		if let Some(sdk_edition) = table.get("sdk_edition").and_then(Value::as_str) {
			project.sdk_edition = sdk_edition.to_string();
		} else {
			Log::new(LogType::Err, "QingLuan.toml 文件解析失败：project缺少sdk_edition键，请检查文件格式。").throw(23);
		}
		if let Some(authors) = table.get("authors").and_then(Value::as_array) {
			for author in authors {
				if let Value::String(author) = author {
					project.authors.push(author.to_string());
				}
			}
		} else {
			Log::new(LogType::Err, "QingLuan.toml 文件解析失败：project缺少authors键，请检查文件格式。").throw(23);
		}
	} else {
		Log::new(LogType::Err, "QingLuan.toml 文件解析失败：缺少project键，请检查文件格式。").throw(23);
	}
	let mut build: Build = Default::default();
	if let Some(table) = value.get("build").and_then(Value::as_table) {
		if let Some(path) = table.get("path").and_then(Value::as_str) {
			build.path = path.to_string();
		} else {
			Log::new(LogType::Err, "QingLuan.toml 文件解析失败：build缺少path键，请检查文件格式。").throw(23);
		}
		if let Some(print) = table.get("print").and_then(Value::as_table) {
			for (key, val) in print {
				match key.as_str() {
					// 文件完成后提示
					"file_ok" => {
						if let Value::Boolean(v) = val {
							build.print.file_ok = *v;
						} else {
							Log::new(LogType::Err, format!("依赖文件解析失败，build中print项 {} 的值不是布尔值。", key).as_str()).throw(23)
						}
					}
					// 警告提示
					"warn" => {
						if let Value::Boolean(v) = val {
							build.print.warn = *v;
						} else {
							Log::new(LogType::Err, format!("依赖文件解析失败，build中print项 {} 的值不是布尔值。", key).as_str()).throw(23)
						}
					}
					_ => Log::new(LogType::Err, format!("依赖文件解析失败，build中print项包含了未知的键 {}。", key).as_str()).throw(23)
				}
			}
		} else {

		}
		if let Some(target) = table.get("target").and_then(Value::as_str) {
			if target == "Lib" {
				build.target = Target::Lib;
			} else {
				build.target = Target::App;
			}
		} else {
			Log::new(LogType::Err, "QingLuan.toml 文件解析失败：build缺少target键，请检查文件格式。").throw(23);
		}
	} else {
		Log::new(LogType::Err, "QingLuan.toml 文件解析失败：缺少build键，请检查文件格式。").throw(23);
	}
	let mut dependencies: Vec<Dependency> = Default::default();
	if let Some(table) = value.get("dependencies").and_then(Value::as_table) {
		// 遍历依赖项
		for (k, v) in table {
			let mut dependency = Dependency {
				name: k.to_string(),
				version: "-1".to_string(),
				include: IncludeAll,
			};
			match v {
				// 字符串格式
				Value::String(v) => {
					dependency.version = v.to_string();
				}
				// 内联表格式
				Value::Table(table) => {
					for (key, val) in table {
						match key.as_str() {
							"version" => {
								if let Value::String(v) = val {
									dependency.version = v.to_string();
								} else {
									Log::new(LogType::Err, format!("依赖文件解析失败，dependencies中依赖项 {} 的版本值不是字符串。", k).as_str()).throw(23)
								}
							}
							"include" | "exclude" => {
								if let Value::Array(arr) = val {
									let mut include: Vec<String> = vec![];
									for v in arr {
										if let Value::String(v) = v {
											include.push(v.to_string());
										} else {
											Log::new(LogType::Err, format!("依赖文件解析失败，dependencies中依赖项 {} 的 {} 值不是字符串。", k, key).as_str()).throw(23)
										}
									}
									if key == "include" {
										dependency.include = Include(include);
									} else {
										dependency.include = Exclude(include);
									}
								} else {
									Log::new(LogType::Err, format!("依赖文件解析失败，dependencies中依赖项 {} 的 {} 值不是数组。", k, key).as_str()).throw(23)
								}
							}
							_ => Log::new(LogType::Err, format!("依赖文件解析失败，dependencies中依赖项 {} 包含了未知的键 {}。", k, key).as_str()).throw(23)
						}
					}
				}
				_ => Log::new(LogType::Err, format!("依赖文件解析失败，dependencies中依赖项 {} 包含了类型为 {} 的值。", k, v.type_str()).as_str()).throw(23)
			}
			dependencies.push(dependency);
		}
	}

	let project_config: ProjectConfig = ProjectConfig {
		project,
		build,
		dependencies,
	};


	// 调用QingLuanPackageManager获取依赖库信息
	for dep in project_config.dependencies.iter() {

		let mut args = vec![
			"find",
			dep.name.as_str(),
			"-v",
			dep.version.as_str(),
		];

		match &dep.include {
			Include(v) => {
				args.push("-include");
				for i in v {
					args.push(i);
				}
			}
			Exclude(v) => {
				args.push("-exclude");
				for i in v {
					args.push(i);
				}
			}
			IncludeAll => {}
		}

		let output = Command::new(&PACKAGE_MANAGER)
			.args(&args)
			.stdout(Stdio::piped())  // 捕获标准输出
			.stderr(Stdio::piped())  // 捕获错误输出
			.output();
		match &output {
			Ok(output) => {
				if output.status.success() {
					let _stdout = String::from_utf8_lossy(&output.stdout);
					let stderr = String::from_utf8_lossy(&output.stderr);
					if !stderr.is_empty() {
						Log::new(LogType::Err, format!("依赖文件解析失败，在读取 {} 依赖库时，包管理器返回了：\n\t {}", dep.name, stderr).as_str()).throw(23);
					} else {
						// let path: Vec<_> = _stdout.trim().split("\u{009C}").collect();
					}
				} else {
					Log::new(LogType::Err, format!("依赖文件解析失败，在读取 {} 依赖库时，包管理器返回了非零的返回值： {} 。", dep.name, output.status.code().unwrap_or_else(|| -1)).as_str()).throw(23);
				}
			},
			Err(e) => {
				Log::new(LogType::Err, format!("包管理器调用失败，请检查包管理器的状态：{}。", e.to_string().red()).as_str()).throw(31);
			}
		}
	}

	project_config
}

/// ## 项目配置（TOML文件）
pub struct ProjectConfig {
	pub project: Project,
	pub build: Build,
	// 依赖项：
	// Hash Map
	// String      依赖库名称
	// Vec(String) 包含的依赖模块路径
	pub dependencies: Vec<Dependency>,
}

#[derive(Default)]
pub struct Build {
	// 路径
	pub path: String,
	// 输出提示
	pub print: Print,
	// 构建目标
	pub target: Target,
}

#[derive(Default)]
pub struct Print {
	pub file_ok: bool,
	pub warn: bool,
}

#[derive(Default)]
pub enum Target {
	#[default]
	App,
	Lib
}

#[derive(Default)]
pub struct Project {
	// 项目名称
	pub name: String,
	// 项目版本
	pub version: String,
	// SDK版本
	pub sdk_edition: String,
	// 作者
	pub authors: Vec<String>,
}

#[derive(Default)]
pub struct Dependency {
	pub name: String,
	pub version: String,
	pub include: DependencyType,
}

#[derive(Default)]
pub enum DependencyType {
	#[default]
	IncludeAll,
	Include(Vec<String>),
	Exclude(Vec<String>),
}