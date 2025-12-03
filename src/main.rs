#![allow(dead_code)]

use crate::_lib::io::{Log, LogType};
use crate::parser::qln::Program;
use crate::parser::toml::Config;
use clap::{Arg, ArgAction, Command};
use colored::*;
use regex::Regex;
use rust_i18n::{i18n, t};
use std::ffi::OsStr;
use std::fmt::Formatter;
use std::io::ErrorKind;
use std::path::Path;
use std::sync::OnceLock;
use std::{env, fmt, fs, process};
use sys_locale::get_locale;
use target_lexicon::HOST;

mod _lib;
mod parser;

const VERSION: &str = "t-0.1.0";

lazy_static::lazy_static! {
    static ref FILE_NAME: String = env::args().next()
        .as_ref()
        .map(Path::new)
        .and_then(Path::file_name)
        .and_then(OsStr::to_str)
        .map(String::from).unwrap_or(String::from("qlc"));
}

i18n!("locales", fallback = "zh-CN");

static PROJECT_CONFIG: OnceLock<Config> = OnceLock::new();

fn main() {
    // CLI输出编码设置为Unicode
    #[cfg(windows)]
    enable_ansi_support();
    // 设置默认语言
    if let Some(lang) = get_locale() {
        set_language(lang.as_str());
    } else {
        set_language("zh-CN");
    }
    // 获得参数
    let args = CLI::parse();
    println!("项目路径：{}", args.project_path);
    println!("输出到目标平台：{}", args.os);
    println!("控制台输出语言：{}", args.language);
    println!("输出的打包方式：{}", args.package);
    // 查看projectPath是否存在
    fs::metadata(&args.project_path).unwrap_or_else(|e| {
        let log = Log::creat(
            e.kind(),
            t!("log.Err.Args.ProjectPath", path = args.project_path)
                .to_string()
                .as_str(),
        );
        log.throw(match e.kind() {
            ErrorKind::NotFound => 21,
            ErrorKind::PermissionDenied => 22,
            _ => 20,
        });
    });
    let program = Program::new(args.project_path);
    program.compile_program();
}

/// ## 启用 ANSI 支持
fn enable_ansi_support() {
    let _ = control::set_virtual_terminal(true);
}

fn set_language(lang: &str) {
    rust_i18n::set_locale(lang);
}

#[derive(Debug)]
struct CLI {
    project_path: String,
    package: Package,
    os: String,
    language: String,
}

impl CLI {
    fn command() -> Command {
        Command::new(FILE_NAME.as_str())
            .version(VERSION)
            .about(t!("DESCRIPTION").to_string())
            .disable_help_flag(true)
            .disable_version_flag(true)
            .help_template(Self::custom_help_template())
            // 项目路径
            .arg(
                Arg::new("project_path")
                    .index(1)
                    .value_name("PATH")
                    .help(t!("ARGS.project_path").to_string()),
            )
            // 打包方式
            .arg(
                Arg::new("package")
                    .short('p')
                    .long("package")
                    .help(t!("ARGS.package", package = Package::Debug.as_atr()).to_string())
                    .value_name("debug|release")
                    .default_value(Package::Debug.as_atr())
                    .hide_possible_values(true)
                    .hide_default_value(true),
            )
            // 目标操作系统
            .arg(
                Arg::new("os")
                    .short('o')
                    .long("os")
                    .help(t!("ARGS.os", os = HOST).to_string())
                    .value_name("os")
                    .hide_possible_values(true)
                    .hide_default_value(true),
            )
            // 语言
            .arg(
                Arg::new("language")
                    .short('l')
                    .long("language")
                    .help(t!("ARGS.language", language = "zh-CN").to_string())
                    .value_name("lang"),
            )
            // 帮助
            .arg(
                Arg::new("help")
                    .short('h')
                    .long("help")
                    .help(t!("ARGS.help").to_string())
                    .action(ArgAction::SetTrue),
            )
            // 版本
            .arg(
                Arg::new("version")
                    .short('v')
                    .long("version")
                    .help(t!("ARGS.version").to_string())
                    .action(ArgAction::SetTrue),
            )
    }

    fn custom_help_template() -> String {
        format!(
            r#"{name} {version}
{{before-help}}{{about-section}}
{about_author}
  {author}

{usage_title}
  {usage_content}

{arguments}
{{positionals}}

{options}
{{options}}

{{after-help}}"#,
            name = t!("NAME").bright_green().bold(),
            version = VERSION.bright_red(),
            about_author = t!("ARGS.about_author").bright_yellow().bold(),
            author = t!("AUTHOR"),
            usage_title = t!("ARGS.usage_title").bright_yellow().bold(),
            usage_content = t!("ARGS.usage_content", app = FILE_NAME.as_str()).to_string(),
            arguments = t!("ARGS.params").bright_yellow().bold(),
            options = t!("ARGS.options").bright_yellow().bold()
        )
    }

    pub fn parse() -> Self {
        let cmd = Self::command();
        let matches = match cmd.try_get_matches() {
            Ok(matches) => matches,
            Err(err) => {
                // let error_message = match err.kind() {
                //     _ => {
                //         // 对于其他类型的错误，尝试将其转换为本地化的错误信息
                //         let err_msg = err.to_string();
                //         // 这里可以根据需要进一步细化错误信息的本地化
                //         t!("log.Err.Args.Error", message = err_msg).to_string()
                //     }
                // };
                Log::new(LogType::Err, err.to_string().as_str()).throw(13)
            }
        };

        // 处理语言设置（保持原有逻辑）
        if let Some(language) = matches.get_one::<String>("language") {
            let re = Regex::new(r"^[a-zA-Z]+(?:-[a-zA-Z]+)?$").unwrap();
            let caps = re.captures(language).unwrap_or_else(|| {
                Log::new(
                    LogType::Err,
                    t!("log.Err.Args.LanguageFormat", language = language)
                        .to_string()
                        .as_str(),
                )
                .throw(13)
            });
            set_language(caps.get(0).unwrap().as_str());
        }

        let mut cli = Self {
            project_path: String::new(),
            package: match matches.get_one::<String>("package").map(|s| s.as_str()) {
                Some("debug") => Package::Debug,
                Some("release") => Package::Release,
                None => Package::Release,
                Some(package) => Log::new(
                    LogType::Err,
                    t!("log.Err.Args.Package", package = package)
                        .to_string()
                        .as_str(),
                )
                .throw(13),
            },
            os: match matches.get_one::<String>("os").map(|s| s) {
                None => HOST.to_string(),
                s => s.unwrap().clone(),
            },
            language: match matches.get_one::<String>("language").map(|s| s) {
                None => get_locale().unwrap_or(String::from("zh-CN")),
                s => s.unwrap().clone(),
            },
        };

        if let Some(project_path) = matches
            .get_one::<String>("project_path")
            .map(|s| s.to_string())
        {
            cli.project_path = project_path;
        } else {
            if matches.get_flag("version") {
                println!("{} {}", t!("NAME"), VERSION);
                process::exit(0);
            }
            let mut cmd = Self::command();
            cmd.print_help().unwrap();
            process::exit(0);
        }

        cli
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Package {
    Debug,
    Release,
}

impl Package {
    pub(crate) fn as_atr(&self) -> &str {
        match self {
            Package::Debug => "debug",
            Package::Release => "release",
        }
    }
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Package::Debug => write!(f, "debug"),
            Package::Release => write!(f, "release"),
        }
    }
}
