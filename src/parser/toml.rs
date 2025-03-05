use std::collections::HashMap;
use std::process::{Command, Stdio};
use serde::Deserialize;
use toml::Value;
use crate::_lib::io::{Log, LogType};
use crate::PACKAGE_MANAGER;

pub fn parser_project(content: String) {
	let value: Value = content.parse::<Value>().unwrap_or_else(|_| {
		Log::new(LogType::Err, "QingLuan.toml 文件解析失败，请检查文件格式。", 23).throw();
	});
	// 获取依赖项
	let mut dependencies: HashMap<&str, HashMap<&str, Vec<&str>>> = Default::default();
	if let Some(deps) = value.get("dependencies").and_then(Value::as_table) {
		// 遍历依赖项
		for (k, v) in deps {
			let mut item: HashMap<&str, Vec<&str>> = HashMap::new();
			match v {
				// 字符串格式
				Value::String(v) => {
					item.insert("version", vec![v]);
					item.insert("include", vec!["*"]);
				}
				// 内联表格式
				Value::Table(table) => {
					let mut vec: Vec<&str> = vec![];
					for (key, val) in table {
						match key.as_str() {
							"version" => {
								if let Value::String(v) = val {
									vec.push(v);
								} else {
									Log::new(LogType::Err, format!("依赖文件解析失败，dependencies中依赖项 {} 的版本值不是字符串。", k).as_str(), 23).throw()
								}
							}
							"include" | "exclude" => {
								if let Value::Array(arr) = val {
									for v in arr {
										if let Value::String(v) = v {
											vec.push(v.as_str());
										} else {
											Log::new(LogType::Err, format!("依赖文件解析失败，dependencies中依赖项 {} 的 {} 值不是字符串。", k, key).as_str(), 23).throw()
										}
									}
								}
							}
							_ => Log::new(LogType::Err, format!("依赖文件解析失败，dependencies中依赖项 {} 包含了未知的键 {}。", k, key).as_str(), 23).throw()
						}
						item.insert(key, vec.to_vec());
						vec.clear();
					}
				}
				_ => Log::new(LogType::Err, format!("依赖文件解析失败，dependencies中依赖项 {} 包含了类型为 {} 的值。", k, v.type_str()).as_str(), 23).throw()
			}
			dependencies.insert(k, item);
		}
	}
	// 读取配置文件
	let config: Config = toml::from_str(content.as_str()).unwrap_or_else(|_| {
		Log::new(LogType::Err, "QingLuan.toml 文件解析失败，请检查文件格式。", 23).throw();
	});
	// 项目配置（TOML文件）
	let mut project: ProjectConfig = ProjectConfig {
		project: config.project,
		build: config.build,
		dependencies: HashMap::new(),
	};
	// 调用QingLuanPackageManager获取依赖库信息
	for (package_name, item) in dependencies.iter() {
		let output = Command::new(PACKAGE_MANAGER)
			.args(&["find", package_name, item.get("version").unwrap().get(0).unwrap()])
			.stdout(Stdio::piped())  // 捕获标准输出
			.stderr(Stdio::piped())  // 捕获错误输出
			.output();
		let mut path: Vec<_> = vec![];
		match output {
			Ok(output) => {
				if output.status.success() {
					let stdout = String::from_utf8_lossy(&output.stdout);
					let stderr = String::from_utf8_lossy(&output.stderr);
					if !stderr.is_empty() {
						Log::new(LogType::Err, format!("依赖文件解析失败，在读取 {} 依赖库时，包管理器返回了：\n\t {}", package_name, stderr).as_str(), 23).throw();
					} else {
						path = stdout.trim().split("\u{009C}").collect();
					}
				} else {
					Log::new(LogType::Err, format!("依赖文件解析失败，在读取 {} 依赖库时，包管理器返回了非零的返回值： {} 。", package_name, output.status.code().unwrap_or_else(|| -1)).as_str(), 23).throw();
				}
			}
			Err(_e) => {
				Log::new(LogType::Err, format!("依赖文件解析失败，没有找到包管理器，应该将包管理器{PACKAGE_MANAGER}同编译器放在同一个文件夹下，或放在环境变量内。").as_str(), 23).throw();
			}
		}
		for path in path {
			println!("{}", path);
		}
		project.dependencies.insert(package_name, item.get("include").unwrap().to_vec());
	}
}

/// ## 项目配置（TOML文件）
struct ProjectConfig {
	project: Project,
	build: Build,
	// 依赖项：
	// Hash Map
	// String      依赖库名称
	// Vec(String) 包含的依赖模块路径
	dependencies: HashMap<&'static str, Vec<String>>,
}

struct Dependency {
	name: &'static str,
	version: &'static str,
	include: Vec<String>,
}

#[derive(Deserialize)]
struct Config {
	project: Project,
	build: Build,
}

#[derive(Deserialize)]
struct Project {
	// 项目名称
	name: &'static str,
	// 项目版本
	version: &'static str,
	// SDK版本
	sdk_edition: &'static str,
	// 入口文件
	main: &'static str,
	// 作者
	authors: Vec<&'static str>,
}

#[derive(Deserialize)]
struct Build {
	// 路径
	path: &'static str,
	// 构建目标
	target: &'static str,
}
