use crate::_lib::io::{Log, LogType};
use crate::parser::qln::ast::AST;
use crate::parser::qln::lexer::{ErrorReporter, Lexer};
use crate::parser::qln::parsing::ASTParser;
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
            });
        }) {
            // self.compile_file()
        }
        let qln_path = format!("{}/src/main.qln", self.path);
        let path = Path::new(&qln_path);
        let lexer = Lexer::new(path);
        let (tokens, errors) = lexer
            .unwrap_or_else(|e| {
                let log = Log::creat(e.kind(), path.to_str().unwrap());
                log.throw(match e.kind() {
                    ErrorKind::NotFound => 21,
                    ErrorKind::PermissionDenied => 22,
                    _ => 20,
                });
            })
            .tokenize();

        // 报告错误
        if !errors.is_empty() {
            let source = fs::read_to_string(path).unwrap_or_else(|e| {
                let log = Log::creat(e.kind(), path.to_str().unwrap());
                log.throw(match e.kind() {
                    ErrorKind::NotFound => 21,
                    ErrorKind::PermissionDenied => 22,
                    _ => 20,
                })
            });
            let mut reporter = ErrorReporter::new(&source, path);
            for (error, span) in errors {
                reporter.add_error(error.clone(), span);
            }
            reporter.report();
            Log::new(LogType::Err, t!("log.Err.CompileError").to_string().as_str()).throw(1);
        } else {
            let mut parser = ASTParser::new(tokens);
            let ast: AST = parser.parse();
        }
    }

    fn compile_file(&self, relative_path: &Path) {
        let path = Path::new(&relative_path);
    }
}
