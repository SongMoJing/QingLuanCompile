use crate::_lib::io::{Char, Cursor, CursorPointing, FileWrapper, Log, LogType};
use crate::core::{lexer, parser};
use crate::PROJECT_CONFIG;
use colored::Colorize;
use std::fs::DirEntry;

#[derive(Default)]
pub struct Record {
	error: Vec<RecordStruct>,
	pub(crate) warn: Vec<RecordStruct>,
}

#[derive(Eq, PartialEq)]
pub enum Struct {
	// 外部
	External,
	// 字符串
	String,
	// 字符
	Char,
	// 行注释
	LineComment,
	// 块注释
	BlockComment,
}

pub struct State {
	pub(crate) located: Struct,
}

pub fn parser_qls(dir: DirEntry) {
	// 游标
	let cursor = Cursor::new(FileWrapper::new(dir.path().display().to_string()));
	// 词法分析
	let mut lexer = lexer::Lexer::new(cursor);
	let tokens = lexer.tokenize();
	// 语法分析
	println!("{:?}", tokens);
	// let mut parser = parser::Parser::new(tokens);
	// let ast = parser.parse().expect("Parsing failed");
}

pub struct RecordStruct {
	pointing: CursorPointing,
	mark: usize,
	message: String,
	note: Option<String>,
}

impl RecordStruct {
	pub(crate) fn new(pointing: CursorPointing, mark: usize, message: String, note: Option<String>) -> Self {
		Self {
			pointing,
			mark,
			message,
			note,
		}
	}

	pub(crate) fn get(self) -> String {
		let space = " ".repeat(if self.pointing.num_line.to_string().len() < 4 { 5 } else { self.pointing.num_line.to_string().len() + 1 });
		format!("{:0>4} {} {}\n{}{}{} {} {}{}",
		        self.pointing.num_line.to_string().bright_cyan().bold(),
		        "|".bright_cyan(),
		        self.pointing.line,
		        space,
		        "|".bright_cyan(),
		        " ".repeat(self.pointing.num_column - 1),
		        "^".repeat(self.mark).bright_yellow().bold(),
		        self.message,
		        if let Some(note) = &self.note {
			        format!("\n{}{} {}{}",
			                space,
			                "=".bright_cyan(),
			                "描述：".bright_white(),
			                note)
		        } else {
			        "".to_string()
		        }
		)
	}
}
