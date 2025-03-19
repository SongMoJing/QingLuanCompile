use std::num::ParseFloatError;
use std::str::FromStr;
use colored::Colorize;
use crate::_lib::io::{Char, Cursor, Log, LogType};
use crate::parser::qls::{Record, RecordStruct, State, Struct};
use crate::PROJECT_CONFIG;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
	Number(f64),        // 数字字面量
	Ident(String),      // 标识符
	Plus,               // +
	Minus,              // -
	Star,               // *
	Slash,              // /
	LParen,             // (
	RParen,             // )
	Eq,                 // =
	Semicolon,          // ;
	EOF,                // 结束标记
}

pub struct Lexer {
	// 光标
	cursor: Cursor,
	// 记录错误和警告
	record: Record,
	// 状态机
	state: State,
}

impl Lexer {
	pub fn new(cursor: Cursor) -> Self {
		Self {
			cursor,
			record: Default::default(),
			state: State {
				located: Struct::External,
			},
		}
	}

	/// 核心词法分析逻辑
	pub fn tokenize(&mut self) -> Vec<Token> {
		let mut tokens = Vec::new();
		while let res = self.cursor.next() {
			match res {
				Char::Char(c) => {
					match c {
						// 跳过空白
						' ' | '\t' | '\n' | '\r' => continue,
						// 数字解析（支持整数和小数）
						'0'..='9' => {
							let num = self.read_number(c);
							tokens.push(Token::Number(num));
						}
						// 标识符解析
						'a'..='z' | 'A'..='Z' | '_' => {
							let ident = self.read_identifier(c);
							tokens.push(Token::Ident(ident));
						}

						// 符号处理
						'+' => tokens.push(Token::Plus),
						'-' => tokens.push(Token::Minus),
						'*' => tokens.push(Token::Star),
						'/' => tokens.push(Token::Slash),
						'(' => tokens.push(Token::LParen),
						')' => tokens.push(Token::RParen),
						'=' => tokens.push(Token::Eq),
						';' => tokens.push(Token::Semicolon),

						// 处理未知字符
						other => panic!("Unexpected character: {}", other),
					}
				}
				Char::EndLine => {
					if self.state.located == Struct::LineComment {
						self.state.located = Struct::External;
					}
				}
				Char::EndFile => {
					if let Some(conf) = PROJECT_CONFIG.get() {
						if conf.build.print.warn {
							if self.record.warn.len() > 0 {
								Log::new(LogType::Info("文件编译结束".yellow()), format!("文件编译完成：\"{}\"", self.cursor.get_file().path()).as_str()).print();
								println!("\t发现{}个问题：", self.record.warn.len());
								break;
							} else {
								Log::new(LogType::Info("文件编译结束".green()), format!("文件编译完成：\"{}\"", self.cursor.get_file().path()).as_str()).print();
								break;
							}
						}
					}
				}
				Char::ErrFile => {
					Log::new(LogType::Err, "文件读取错误").throw(20);
				}
			}
		}
		tokens
	}

	/// 读取连续数字字符
	fn read_number(&mut self, first: char) -> f64 {
		let mut s = String::new();
		s.push(first);
		let mut has_x = false;
		let mut has_dot = false;
		while let Char::Char(c) = self.cursor.peek() {
			match c {
				'0'..='9' | 'a'..='f' | 'A'..='F' | '_' => {
					self.cursor.next();
					s.push(c)
				}
				'.' => {
					if has_dot {
						break;
					}
					self.cursor.next();
					s.push(c);
					has_dot = true;
				}
				_ => break,
			}
		}
		s.parse::<ValueNum>().unwrap_or_else(|_| {
			let mut point = self.cursor.get_pointing();
			point.num_column += 1;
			let record = RecordStruct::new(point, 1, "无法解析意外的字符为数字".to_string(), None).get();
			Log::new(LogType::Err, format!("{}\n{}{}\n{}",
			                               "编译时检查到词法错误：".bright_white(),
			                               "位于 ",
			                               self.cursor.get_file().path().green(),
			                               record.as_str()).as_str()).throw(41);
		}).get()
	}

	/// 读取标识符
	fn read_identifier(&mut self, first: char) -> String {
		let mut s = String::new();
		s.push(first);

		while let Char::Char(c) = self.cursor.peek() {
			if c.is_alphanumeric() || c == '_' {
				self.cursor.next();
				s.push(c);
			} else {
				break;
			}
		}
		s
	}
}

#[derive(Debug, PartialEq)]
struct ValueNum(f64);

impl FromStr for ValueNum {
	type Err = ParseFloatError;

	fn from_str(s: &str) -> Result<ValueNum, ParseFloatError> {
		if s.contains('_') {
			let s = s.replace("_", "");
		}
		if s.starts_with('0') {
			if s.starts_with("0x") {
				Ok(ValueNum(f64::(&s[2..], 16).unwrap())).expect("TODO: panic message");
			} else if s.starts_with("0o") {
				Ok(ValueNum(f64::from_str_radix(&s[2..], 8).unwrap())).expect("TODO: panic message")
			} else if s.starts_with("0b") {
				Ok(ValueNum(f64::from_str_radix(&s[2..], 2).unwrap())).expect("TODO: panic message")
			}
			else {
				Ok(ValueNum(f64::from_str_radix(&s, 10).unwrap())).expect("TODO: panic message")
			}
		}
	}
}

impl ValueNum {
	fn get(&self) -> f64 {
		match self {
			ValueNum(n) => *n,
		}
	}
}