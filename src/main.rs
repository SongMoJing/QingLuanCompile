use std::{env, fmt, fs};
use std::env::consts::{ARCH, OS};
use std::io::ErrorKind;
use std::path::Path;
use std::sync::OnceLock;
use clap::{arg, Command, CommandFactory, Parser};
use colored::*;
use lazy_static::lazy::Lazy;
use rust_i18n::{i18n, t};
use lazy_static::lazy_static;
use crate::_lib::io::{Log, LogType};
use crate::parser::qln::lexer::{ErrorReporter, Lexer};
use crate::parser::toml::{Config, load_config};
use target_lexicon::HOST;
use regex::Regex;

mod _lib;
mod parser;

// 假设 t! 宏返回 String，创建一个静态引用
lazy_static::lazy_static! {
    static ref VERSION: String = t!("VERSION");
    static ref DESCRIPTION: String = t!("DESCRIPTION");
	static ref AUTHOR: String = t!("AUTHOR");
}

i18n!("locales");

static PROJECT_CONFIG: OnceLock<Config> = OnceLock::new();

fn main() {
	// CLI输出编码设置为Unicode
	#[cfg(windows)]
	enable_ansi_support();
	set_language("zh-CN");
	// 获得参数
	let args = CLI::parse();
	// 查看projectPath是否存在
	fs::metadata(&args.project_path).unwrap_or_else(|e| {
		let log = Log::creat(e.kind(), args.project_path.as_str());
		log.throw(match e.kind() {
			ErrorKind::NotFound => { 21 }
			ErrorKind::PermissionDenied => { 22 }
			_ => { 20 }
		});
	});
	// 读取QingLuan.toml文件
	let mut toml = _lib::io::FileWrapper::new(format!("{}/QingLuan.toml", args.project_path.as_str()));
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
		})
	);
	let qln_path = args.project_path.clone().as_str().to_owned() + "/src/main.qln";
	let path = Path::new(&qln_path);
	let lexer = Lexer::new(path);
	let (tokens, errors) = lexer.unwrap_or_else(|e| {
		let log = Log::creat(e.kind(), path.to_str().unwrap());
		log.throw(match e.kind() {
			ErrorKind::NotFound => { 21 }
			ErrorKind::PermissionDenied => { 22 }
			_ => { 20 }
		});
	}).tokenize();
	
	// 报告错误
	if !errors.is_empty() {
		let source = fs::read_to_string(path).unwrap_or_else(|e| {
			let log = Log::creat(e.kind(), path.to_str().unwrap());
			log.throw(match e.kind() {
				ErrorKind::NotFound => { 21 }
				ErrorKind::PermissionDenied => { 22 }
				_ => { 20 }
			})
		});
		let mut reporter = ErrorReporter::new(&source, path);
		for (error, span) in errors {
			reporter.add_error(error.clone(), span);
		}
		reporter.report();
		Log::new(LogType::Err, t!("log.Err.CompileError").as_str()).throw(1);
	} else {
		for token in tokens {
			println!("{:?}\n\t{:?}", token.kind, token.span);
		}
		// let mut parser = Parser::new(tokens);
		// let ast = parser.parse_expr();
		// println!("{:?}", ast);
	}
}

/// ## 启用 ANSI 支持
fn enable_ansi_support() {
	let _ = control::set_virtual_terminal(true);
}

fn set_language(lang: &str) {
	rust_i18n::set_locale(lang);
}

#[derive(Parser, Debug)]
#[command(
	version = VERSION.as_str(),
	about = DESCRIPTION.as_str(),
	author = AUTHOR.as_str(),
	long_about = None,
	arg_required_else_help = true,
	disable_help_flag = true,
	disable_version_flag = true,
	help_template = CLI::custom_help_template()
)]
struct CLI {
	/// 项目路径
	#[arg(index = 1, value_name = "PATH", help = t!("ARGS.project_path"), required = true)]
	project_path: String,
	/// 编译目标<Debug|Release>
	#[arg(
		short,
		long,
		help = format!("{}{}]", t!("ARGS.package"), "release"),
		value_name = "debug|release",
		default_value = "release",
		hide_possible_values = true,
		hide_default_value = true
	)]
	package: Package,
	/// 编译目标系统
	#[arg(
		short,
		long,
		help = format!("{}{}]", t!("ARGS.os"), "x86_64-pc-windows-msvc"),
		value_name = "os",
		default_value = "x86_64-pc-windows-msvc",
		hide_possible_values = true,
		hide_default_value = true
	)]
	os: Option<String>,
	/// 输出
	#[arg(
		short,
		long,
		help = format!("{}{}]", t!("ARGS.language"), "zh-CN"),
		value_name = "lang",
	)]
	language: Option<String>,
	/// 帮助
	#[arg(
		short,
		long,
		help = t!("ARGS.help"),
		action = clap::ArgAction::Help,
	)]
	help: Option<bool>,
	/// 版本
	#[arg(
		short,
		long,
		help = t!("ARGS.version"),
		action = clap::ArgAction::Version,
	)]
	version: Option<bool>,
}

impl CLI {
	fn custom_help_template() -> String {
		format!(r#"{name} {version}
{{before-help}}{{about-section}}
{usage}
  {{usage}}

{arguments}
{{positionals}}

{options}
{{options}}

{{after-help}}"#,
		        name = t!("NAME").bright_green().bold(),
		        version = VERSION.as_str().bright_red(),
		        usage = t!("ARGS.usage").bright_yellow().bold(),
		        arguments = t!("ARGS.params").bright_yellow().bold(),
		        options = t!("ARGS.options").bright_yellow().bold()
		)
	}
	
	pub fn parse() -> Self {
		let mut cli = <Self as Parser>::parse();
		if cli.os.is_none() {
			cli.os = Some(HOST.to_string());
		}
		if let Some(language) = &cli.language {
			let re = Regex::new(r"^(?P<lang>\w{2})(?:-(?P<region>\w{2}))?$").unwrap_or_else(|_| panic!("Invalid regex"));
			let caps = re.captures(language).unwrap_or_else(|| panic!("Invalid language format"));
			set_language(caps.name("lang").unwrap().as_str());
		}
		cli
	}
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum, Debug)]
enum Package {
	#[value(name = "debug", alias = "d")]
	Debug,
	#[value(name = "release", alias = "r")]
	Release,
}
