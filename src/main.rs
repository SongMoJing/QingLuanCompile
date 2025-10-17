use crate::_lib::io::{Log, LogType};
use crate::parser::qln::Program;
use crate::parser::toml::Config;
use clap::{Arg, ArgAction, Command};
use colored::*;
use regex::Regex;
use rust_i18n::{i18n, t};
use std::fs;
use std::io::ErrorKind;
use std::sync::OnceLock;
use sys_locale::get_locale;
use target_lexicon::HOST;

mod _lib;
mod parser;

lazy_static::lazy_static! {
    static ref VERSION: String = String::from("t-0.1.0");
    static ref NAME: String = t!("NAME").to_string();
    static ref AUTHOR: String = String::from("PRC.松蓦箐 <Song_Mojing@outlook.com>");
    static ref APPLICATION_NAME: String = String::from("青鸾编译器");
}

i18n!("locales", fallback = "zh-CN");

static PROJECT_CONFIG: OnceLock<Config> = OnceLock::new();

fn main() {
    // CLI输出编码设置为Unicode
    #[cfg(windows)]
    enable_ansi_support();
    set_language("zh-CN");
    // 获得参数
    let args = CLI::parse();
    println!("{:?}", args.os);
    // 查看projectPath是否存在
    fs::metadata(&args.project_path).unwrap_or_else(|e| {
        let log = Log::creat(e.kind(), args.project_path.as_str());
        log.throw(match e.kind() {
            ErrorKind::NotFound => 21,
            ErrorKind::PermissionDenied => 22,
            _ => 20,
        });
    });
    Program::new(args.project_path);
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
    help: Option<bool>,
    version: Option<bool>,
}

impl CLI {
    fn command() -> Command {
        Command::new(APPLICATION_NAME.as_str())
            .version(VERSION.as_str())
            .about(t!("DESCRIPTION").to_string())
            .author(AUTHOR.as_str())
            .arg_required_else_help(true)
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
                    .help(format!("{}{}]", t!("ARGS.package"), "release"))
                    .value_name("debug|release")
                    .default_value("release")
                    .hide_possible_values(true)
                    .hide_default_value(true),
            )
            // 目标操作系统
            .arg(
                Arg::new("os")
                    .short('o')
                    .long("os")
                    .help(format!("{}{}]", t!("ARGS.os"), HOST))
                    .value_name("os")
                    .hide_possible_values(true)
                    .hide_default_value(true),
            )
            // 语言
            .arg(
                Arg::new("language")
                    .short('l')
                    .long("language")
                    .help(format!(
                        "{}{}]",
                        t!("ARGS.language"),
                        get_locale().unwrap_or(String::from("zh-CN"))
                    ))
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
        if let Some(lang) = get_locale() {
            set_language(lang.as_str());
        }

        let cmd = Self::command();
        let matches = cmd.get_matches();

        // 处理语言设置
        if let Some(language) = matches.get_one::<String>("language") {
            let re = Regex::new(r"^(?P<lang>\w{2})(?:-(?P<region>\w{2}))?$").unwrap_or_else(|_| {
                Log::new(
                    LogType::Err,
                    t!("log.Err.Invalid.ARGS.LanguageFormat")
                        .to_string()
                        .as_str(),
                )
                .throw(13)
            });
            let caps = re.captures(language).unwrap_or_else(|| {
                Log::new(
                    LogType::Err,
                    t!("log.Err.Invalid.ARGS.LanguageFormat")
                        .to_string()
                        .as_str(),
                )
                .throw(13)
            });
            set_language(caps.name("lang").unwrap().as_str());
        }

        // 项目路径
        let project_path = matches
            .get_one::<String>("project_path")
            .map(|s| s.to_string());

        if project_path.is_none() {
            if matches.get_flag("version") {
                println!("{} {}", APPLICATION_NAME.as_str(), VERSION.as_str());
                std::process::exit(0);
            }
            let mut cmd = Self::command();
            cmd.print_help().unwrap();
            std::process::exit(0);
        }

        Self {
            project_path: project_path.unwrap(),
            package: match matches.get_one::<String>("package").map(|s| s.as_str()) {
                Some("debug") => Package::Debug,
                Some("release") => Package::Release,
                None => Package::Release,
                _ => Log::new(
                    LogType::Err,
                    t!("log.Err.Invalid.ARGS.Package").to_string().as_str(),
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
            help: Option::from(matches.get_flag("help")),
            version: Option::from(matches.get_flag("version")),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Package {
    Debug,
    Release,
}
