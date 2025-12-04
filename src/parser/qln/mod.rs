use std::collections::VecDeque;
use crate::_lib::io::{Log, LogType};
use crate::parser::qln::ast::ASTType;
use crate::parser::qln::lexer::{LexErrorReporter, Lexer};
use crate::parser::qln::parsing::{ASTParser, ParseErrorReporter};
use crate::parser::toml::load_config;
use crate::{_lib, PROJECT_CONFIG};
use rust_i18n::t;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

pub(crate) mod ast;
pub(crate) mod lexer;
pub(crate) mod parsing;
pub(crate) mod token;

pub struct Program {
    path: String,
}

impl Program {
    pub(crate) fn new(path: String) -> Program {
        Program { path }
    }

    pub(crate) fn compile_program(&self) {
        // 读取QingLuan.toml文件
        let mut toml = _lib::io::FileWrapper::new(format!("{}/QingLuan.toml", self.path));
        // 解析项目配置并加入全局变量
        let _ = PROJECT_CONFIG.set({
            load_config(toml.read_to_string().unwrap_or_else(|e| {
                let log = Log::creat(e.kind(), toml.path().as_str());
                log.throw(match e.kind() {
                    ErrorKind::NotFound => 21,
                    ErrorKind::PermissionDenied => 22,
                    _ => 20,
                });
            }))
            .unwrap_or_else(|e| {
                let log = Log::creat(e.kind(), format!("{}: \n{}", toml.path(), e).as_str());
                log.throw(match e.kind() {
                    ErrorKind::NotFound => 21,
                    ErrorKind::PermissionDenied => 22,
                    _ => 20,
                });
            })
        });
        // 遍历src目录
        for entry in fs::read_dir(format!("{}/src", self.path)).unwrap_or_else(|e| {
            let log = Log::creat(e.kind(), format!("{}: \n{}", toml.path(), e).as_str());
            log.throw(match e.kind() {
                ErrorKind::NotFound => 21,
                ErrorKind::PermissionDenied => 22,
                _ => 20,
            })
        }) {
            if let Ok(e) = entry {
                self.compile_file(e.path().as_path());
            }
        }
    }

    fn compile_file(&self, relative_path: &Path) {
        let lexer = Lexer::new(relative_path);
        let (tokens, errors) = lexer
            .unwrap_or_else(|e| {
                let log = Log::creat(e.kind(), relative_path.to_str().unwrap());
                log.throw(match e.kind() {
                    ErrorKind::NotFound => 21,
                    ErrorKind::PermissionDenied => 22,
                    _ => 20,
                });
            })
            .tokenize();

        println!("File ============================================================================\n{}", relative_path.to_str().unwrap());

        // 报告错误
        if !errors.is_empty() {
            let source = fs::read_to_string(relative_path).unwrap_or_else(|e| {
                let log = Log::creat(e.kind(), relative_path.to_str().unwrap());
                log.throw(match e.kind() {
                    ErrorKind::NotFound => 21,
                    ErrorKind::PermissionDenied => 22,
                    _ => 20,
                })
            });
            let mut reporter = LexErrorReporter::new(&source, relative_path);
            for (error, span) in errors {
                reporter.add_error(error.clone(), span);
            }
            reporter.report();
            Log::new(
                LogType::Err,
                t!("log.Err.CompileError").to_string().as_str(),
            )
            .throw(1);
        } else {
            // Vec 转 VecDeque
            let tokens = tokens.into_iter().collect::<VecDeque<_>>();
            let mut parser = ASTParser::new(tokens, ASTType::File);
            let (ast, errors) = parser.parse();
            println!("Tokens ========\n{:?}", ast);
            if !errors.is_empty() {
                let source = fs::read_to_string(relative_path).unwrap_or_else(|e| {
                    let log = Log::creat(e.kind(), relative_path.to_str().unwrap());
                    log.throw(match e.kind() {
                        ErrorKind::NotFound => 21,
                        ErrorKind::PermissionDenied => 22,
                        _ => 20,
                    })
                });
                let mut reporter = ParseErrorReporter::new(&source, relative_path);
                for (error, span) in errors {
                    reporter.add_error(error.clone(), span);
                }
                reporter.report();
                Log::new(
                    LogType::Err,
                    t!("log.Err.CompileError").to_string().as_str(),
                )
                    .throw(1);
            }
        }
    }
}
