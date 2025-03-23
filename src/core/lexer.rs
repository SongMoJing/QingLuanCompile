use std::str::FromStr;
use colored::Colorize;
use crate::_lib::io::{Char, Cursor, Log, LogType};
use crate::parser::qls::{Record, RecordStruct, State, Struct};
use crate::PROJECT_CONFIG;

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
		let mut tokens: Vec<Token> = Vec::new();
		while let res = self.cursor.next() {
			match res {
				Char::Char(c) => {
					match self.state.located {
						Struct::LineComment => continue,
						_ => {}
					}
					match c {
						// 跳过空白
						' ' | '\t' | '\n' | '\r' => continue,
						// 数字解析（支持整数和小数）
						'0'..='9' => {
							tokens.push(self.read_number(c));
						}
						// 标识符解析
						'a'..='z' | 'A'..='Z' | '_' => {
							tokens.push(self.read_identifier(c));
						}
						// 符号处理
						'+' => tokens.push(Token::Operator(Operator::Plus)),
						'-' => tokens.push(Token::Operator(Operator::Minus)),
						'*' => tokens.push(Token::Operator(Operator::Star)),
						'/' => {
							if let Char::Char('/') = self.cursor.peek() {
								self.state.located = Struct::LineComment;
								self.cursor.next();
							} else {
								tokens.push(Token::Operator(Operator::Slash));
							}
						}
						'(' => tokens.push(Token::StructFlag(StructFlag::OpenParen)),
						'{' => tokens.push(Token::StructFlag(StructFlag::OpenBrace)),
						')' => tokens.push(Token::StructFlag(StructFlag::CloseParen)),
						'}' => tokens.push(Token::StructFlag(StructFlag::CloseBrace)),
						'=' => tokens.push(Token::Eq),
						';' => tokens.push(Token::EndStatement),

						// 处理未知字符
						_ => {
							let point = self.cursor.get_pointing();
							let record = RecordStruct::new(point, 1, "意外的字符".to_string(), None).get();
							Log::new(LogType::Err, format!("{}\n{}{}\n{}",
														   "编译时检查到词法错误：".bright_white(),
														   "位于 ",
														   self.cursor.get_file().path().green(),
														   record.as_str()).as_str()).throw(41);
						}
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
								Log::new(LogType::Info("文件编译结束".yellow()), self.cursor.get_file().path().as_str()).print();
								println!("\t发现{}个问题：", self.record.warn.len());
								break;
							} else {
								Log::new(LogType::Info("文件编译结束".green()), self.cursor.get_file().path().as_str()).print();
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
	fn read_number(&mut self, first: char) -> Token::Value {
		fn to_number(s: String) -> Option<Token::Value> {
			let sanitized = s.replace('_', "");
			let (radix, parts) = if sanitized.len() >= 2 {
				match &sanitized[..2] {
					"0b" | "0B" => (2, &sanitized[2..]),
					"0o" | "0O" => (8, &sanitized[2..]),
					"0x" | "0X" => (16, &sanitized[2..]),
					_ => (10, &sanitized[..]),
				}
			} else {
				(10, &sanitized[..])
			};
			match radix {
				2 => {
					// if parts.contains('-')
					Some(Token::Value(Value::NumIsize(u64::from_str_radix(parts, radix)?)))
				}
				8 => Some(Token::Value(Value::Int(u64::from_str_radix(parts, radix)?))),
			}
		}

		let mut s = String::new();
		s.push(first);
		if first == '0' {
			if let Char::Char(c) = self.cursor.peek() {
				match c {
					'b' | 'B' => {}
					'o' | 'O' => {}
					'x' | 'X' => {}
					_ => {}
				}
			}
		}
		while let Char::Char(c) = self.cursor.peek() {
			match c {
				'0'..='9' | 'a'..='f' | 'A'..='F' => {
					self.cursor.next();
					s.push(c)
				}
				'x' | 'X' | 'o' | 'O' | 'b' | 'B' => {

				}
				'_' => {
					self.cursor.next();
				}
				_ => break,
			}
		}
		to_number(s).unwrap_or_else(|_| {
			let mut point = self.cursor.get_pointing();
			point.num_column += 1;
			let record = RecordStruct::new(point, 1, "无法解析意外的字符为数字".to_string(), None).get();
			Log::new(LogType::Err, format!("{}\n{}{}\n{}",
										   "编译时检查到词法错误：".bright_white(),
										   "位于 ",
										   self.cursor.get_file().path().green(),
										   record.as_str()).as_str()).throw(41);
		})
	}
	/// 读取标识符
	fn read_identifier(&mut self, first: char) -> Token {
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
		match s.as_str() {
			"true" => Token::Value(Value::Boolean(true)),
			"false" => Token::Value(Value::Boolean(false)),
			_ => Token::Ident(s),
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
	/// 字面量
	Value(Value),
	/// 函数调用
	Call(Token::Ident, Vec<Token>),
	/// 标识符
	Ident(String),
	/// 运算符
	Operator(Operator),
	/// 结构分隔符
	StructFlag(StructFlag),
	/// =
	Eq,
	/// 语句结束标记
	EndStatement,
}

/// 结构标识符，括号
#[derive(Debug, Clone, PartialEq)]
pub enum StructFlag {
	/// (
	OpenParen,
	/// )
	CloseParen,
	/// {
	OpenBrace,
	/// }
	CloseBrace,
	/// <
	OpenAngle,
	/// >
	CloseAngle,
}
/// 运算符
#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
	/// +
	Plus,
	/// -
	Minus,
	/// *
	Star,
	/// /
	Slash,
	/// %
	Percent,
	/// &
	And,
	/// |
	Or,
	/// !
	Not,
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
