use crate::_lib::io::{Cursor, CursorPointing, FileWrapper};
use crate::core::{lexer, parser};
use colored::Colorize;
use std::fs::DirEntry;
use unicode_width::UnicodeWidthChar;
use crate::core::parser::ParserResult;

#[derive(Default)]
pub struct Record {
	error: Vec<RecordStruct>
}

#[derive(Eq, PartialEq)]
pub enum Struct {
	// 外部
	External,
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
	let mut parser = parser::Parser::new();
	// 词法分析
	while let Ok(tokens) = lexer.next() {
		// 语法分析
		let ast = parser.parse(tokens).unwrap_or_else(|e| match e {
				ParserResult::WaitNext => {
					println!("WaitNext");
				}
				ParserResult::Unexpected(t) => {
					RecordStruct::new(
						lexer.pointing.clone(),
						0,
						format!("Unexpected {}", t),
						None,
					)
				}
			});
		println!("{:?}", ast);
	}
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
		        " ".repeat({
			        let mut length = 0;
			        let mut index = 0;
			        for c in self.pointing.line.chars() {
				        index += 1;
				        length += UnicodeWidthChar::width(c).unwrap_or(1);
				        if index == self.pointing.num_column {
					        break;
				        }
			        }
			        if length > 0 { length - 1 } else { 0 }
		        }),
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

/// 字面量
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
	NumI8(i8),
	NumI16(i16),
	NumI32(i32),
	NumI64(i64),
	NumI128(i128),
	NumIsize(isize),
	NumU8(u8),
	NumU16(u16),
	NumU32(u32),
	NumU64(u64),
	NumU128(u128),
	NumUsize(usize),
	NumF32(f32),
	NumF64(f64),
	String(String),
	Char(char),
	Boolean(bool),
}
