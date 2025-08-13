use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::process::exit;
use std::sync::OnceLock;

use colored::*;

use crate::_lib::io::{Log, LogType};
use crate::parser::qln::lexer::{ErrorReporter, Lexer};
use crate::parser::toml::{Config, load_config};

mod _lib;
mod parser;

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
}

static PROJECT_CONFIG: OnceLock<Config> = OnceLock::new();

fn main() {
	// CLI输出编码设置为Unicode
	#[cfg(windows)]
	enable_ansi_support();
	// 获得参数
	let args = get_args();
	// 检查开始方式
	if let Some(root_path) = &args.project_path {
		// 查看projectPath是否存在
		fs::metadata(root_path).unwrap_or_else(|e| {
			let log = Log::creat(e.kind(), root_path.as_str());
			log.throw(match e.kind() {
				ErrorKind::NotFound => { 21 }
				ErrorKind::PermissionDenied => { 22 }
				_ => { 20 }
			});
		});
		// 读取QingLuan.toml文件
		let mut toml = _lib::io::FileWrapper::new(format!("{root_path}/QingLuan.toml"));
		// 读取文件内容
		// 解析项目配置并加入全局变量
		PROJECT_CONFIG.get_or_init(|| load_config(
			toml.read_to_string().unwrap_or_else(|e| {
				let log = Log::creat(e.kind(), toml.path().as_str());
				log.throw(match e.kind() {
					ErrorKind::NotFound => { 21 }
					ErrorKind::PermissionDenied => { 22 }
					_ => { 20 }
				});
			}))
			.unwrap_or_else(|e| {
				let log = Log::creat(e.kind(), format!("{}: \n{}", toml.path(), e).as_str());
				log.throw(match e.kind() {
					ErrorKind::NotFound => { 21 }
					ErrorKind::PermissionDenied => { 22 }
					_ => { 20 }
				});
			}));

		let path = Path::new("D:\\Program\\Rust\\QingLuanCompile\\Note\\project\\src\\main.qln");
		let mut lexer = Lexer::new(path);
		let (tokens, errors) = lexer.expect("REASON").tokenize();

		// 报告错误
		if !errors.is_empty() {
			let source = fs::read_to_string(path).expect("无法读取文件");
			let mut reporter = ErrorReporter::new(&source, path);
			for (error, span) in errors {
				reporter.add_error(error.clone(), span);
			}
			reporter.report();
			Log::new(LogType::Err, "编译因错误而终止").throw(1);
		} else {
			for token in tokens {
				println!("{:?}", token);
			}
		}
	} else {
		Log::new(LogType::Err, "缺少项目路径").throw(10);
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
		println!("   [必填] <路径> 项目根路径");
		println!("{}", "选项：".yellow());
		println!("   <-h | --help>     获取帮助");
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
					help();
					exit(0);
				}
				// 未知目标
				_ => {
					Log::new(LogType::Err, format!("未知的选项 {}。", arg).as_str()).throw(11);
				}
			}
		} else {
			res.project_path = Some(arg.replace("\\", "/"));
		}
	}
	res
}
