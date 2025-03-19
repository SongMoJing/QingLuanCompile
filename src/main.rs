use crate::_lib::io::{Log, LogType};
use crate::parser::toml::ProjectConfig;
use colored::*;
use std::io::ErrorKind;
use std::process::exit;
use std::sync::OnceLock;

mod _lib;
mod parser;
mod core;

/// ## 程序版本
const VERSION: &str = "t.0.1";
/// ## 程序名称
const NAME: &str = "\"青鸾\" 编译器";
/// ## 程序作者
const AUTHOR: &str = "PRC.松蓦箐 <Song_Mojing@outlook.com>";
/// ## 包管理器
const PACKAGE_MANAGER: &str = "QingLuanPackageManager";

/// ## 可接受的参数列表
struct Args {
	project_path: Option<String>,
	_help: bool,
	_compile: &'static str,
}

static PROJECT_CONFIG: OnceLock<ProjectConfig> = OnceLock::new();
static PROJECT_ROOT: OnceLock<String> = OnceLock::new();

fn main() {
	#[cfg(windows)]
	enable_ansi_support();
	// 获得参数
	let args = get_args();
	// 检查开始方式
	if let Some(root_path) = &args.project_path {
		// 查看projectPath是否存在
		std::fs::metadata(root_path).unwrap_or_else(|e| match e.kind() {
			ErrorKind::NotFound => {
				Log::new(LogType::Err, format!("路径 {} 不存在。", root_path).as_str()).throw(21);
			}
			ErrorKind::PermissionDenied => {
				Log::new(LogType::Err, format!("路径 {} 读取失败，请检查文件权限。", root_path).as_str()).throw(22);
			}
			_ => {
				Log::new(LogType::Err, format!("路径 {} 产生未知错误。", root_path).as_str()).throw(20);
			}
		});
		// 读取QingLuan.toml文件
		let mut toml = _lib::io::FileWrapper::new(format!("{root_path}/QingLuan.toml"));
		// 读取文件内容
		let content: String = toml.read_to_string().unwrap_or_else(|e| match e.kind() {
			ErrorKind::NotFound => {
				Log::new(LogType::Err, "QingLuan.toml 文件读取失败，请检查文件是否存在。").throw(21);
			}
			ErrorKind::PermissionDenied => {
				Log::new(LogType::Err, "QingLuan.toml 文件读取失败，请检查文件权限。").throw(22);
			}
			_ => {
				Log::new(LogType::Err, "QingLuan.toml 产生未知错误。").throw(20);
			}
		});
		// 解析项目配置并加入全局变量
		PROJECT_CONFIG.get_or_init(|| parser::toml::parser_config(content));
		PROJECT_ROOT.get_or_init(|| root_path.replace("\\", "/"));
		Log::new(LogType::Info("编译".green()), "项目开始解析").print();
		parser::start();
	} else {
		Log::new(LogType::Err, "缺少项目路径。").throw(10);
	}
}

/// ## 启用 ANSI 支持
fn enable_ansi_support() {
	let _ = control::set_virtual_terminal(true);
}

/// ## 获取命令行参数
/// 读入操作和必要参数<br>
/// scriptPath. 路径
fn get_args() -> Args {
	let mut res: Args = Args {
		_help: false,
		_compile: "debug",
		project_path: None,
	};

	// 打印帮助
	fn help() {
		println!("{}", "帮助".yellow());
		println!("   {} 版本：{}", NAME.green(), VERSION.green());
		println!("   {}", AUTHOR);
		println!("{}", "用法：".yellow());
		println!("   QingLuanCompile [参数] [选项]");
		println!("{}", "参数：".yellow());
		println!("   [必填] <脚本路径> 填入用于解释的脚本路径");
		println!("{}", "选项：".yellow());
		println!("   <-h | --help>     获取帮助");
		println!("   <-c | --compile>  打包方式 可选：debug|release");
		exit(0);
	}

	// 获取参数
	let mut args = std::env::args();
	if args.len() < 2 {
		help();
	} else {
		args.next();
	}

	while let Some(arg) = args.next() {
		if arg.starts_with("-") {
			match arg.as_str() {
				// 显示帮助
				"-h" | "--help" => {
					res._help = true;
					help();
				}
				// 编译方式
				"-c" | "--compile" => {
					if let Some(path) = args.next() {
						match path.as_str() {
							"debug" => res._compile = "debug",
							"release" => res._compile = "release",
							_ => {
								Log::new(LogType::Err, format!("未知的编译方式 {}。", path).as_str()).throw(12);
							}
						}
					} else {
						res._compile = "debug";
						Log::new(LogType::Warn, "缺少编译方式，默认为Debug").print();
					}
				}
				// 未知目标
				_ => {
					Log::new(LogType::Err, format!("未知的选项 {}。", arg).as_str()).throw(11);
				}
			}
		} else {
			res.project_path = Some(arg);
		}
	}
	res
}
