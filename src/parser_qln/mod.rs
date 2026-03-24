use crate::_lib::io::{Log, LogType};
use crate::parser_qln::model_ast::ASTType;
use crate::parser_qln::lexer::{LexErrorReporter, Lexer};
use crate::parser_qln::parsing::{ASTParser, ParseErrorReporter};
use rust_i18n::t;
use std::collections::VecDeque;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

pub(crate) mod model_ast;
pub(crate) mod lexer;
pub(crate) mod parsing;
pub(crate) mod model_token;

pub struct Program {
	path: String,
}

impl Program {
	pub(crate) fn new(path: String) -> Program {
		Program { path }
	}

	pub(crate) fn compile_program(&self) {
		// 读取QingLuan.toml文件
		// self.compile_file((&format!("{}/build.qln", self.path)).as_path());
		// // 遍历src目录
		// for entry in fs::read_dir(format!("{}/src", self.path)).unwrap_or_else(|e| {
		// 	let log = Log::creat(e.kind(), format!("{}: \n{}", build.path(), e).as_str());
		// 	log.throw(match e.kind() {
		// 		ErrorKind::NotFound => 21,
		// 		ErrorKind::PermissionDenied => 22,
		// 		_ => 20,
		// 	})
		// }) {
		// 	if let Ok(e) = entry {
		// 		self.compile_file(&e.path());
		// 	}
		// }
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

		println!(
			"File ============================================================================\n{}",
			relative_path.to_str().unwrap()
		);

		// println!("Token ========\n{:?}", tokens);
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
			let tokens = tokens.into_iter().collect::<VecDeque<_>>();
			let mut parser = ASTParser::new(tokens, ASTType::File);
			let (ast, errors) = parser.parse();
			println!("AST ========\n{:?}", ast);
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
