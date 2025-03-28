use crate::_lib::io::{Char, Cursor, CursorPointing, Log, LogType};
use crate::parser::qls::{RecordStruct, State, Struct, Value};
use colored::Colorize;
use std::str::FromStr;

pub struct Lexer {
	// 光标
	cursor: Cursor,
	// 状态机
	state: State,
}

pub enum Err {
	EndFile,
}

impl Lexer {
	pub fn new(cursor: Cursor) -> Self {
		Self {
			cursor,
			state: State {
				located: Struct::External,
			},
		}
	}
	/// 报错
	fn throw_error(&self, message: String, note: Option<String>, len: usize, point: CursorPointing) -> ! {
		let record = RecordStruct::new(point, len, message, note).get();
		Log::new(LogType::Err, format!("{}\n{}{}\n{}",
		                               "编译时检查到词法错误：".bright_white(),
		                               "位于 ",
		                               self.cursor.get_file().path().green(),
		                               record.as_str()).as_str()).throw(41);
	}
	/// 核心词法分析逻辑
	pub fn next(&mut self) -> Result<Vec<Token>, Err> {
		let mut tokens: Vec<Token> = Vec::new();
		loop {
			match self.cursor.next() {
				Char::Char(c) => {
					match self.state.located {
						Struct::LineComment => continue,
						Struct::BlockComment => {
							if '*' == c {
								if let Char::Char('/') = self.cursor.peek() {
									self.state.located = Struct::External;
									self.cursor.next();
									continue;
								}
							}
							continue;
						}
						_ => {}
					}
					match c {
						// 跳过空白
						' ' | '\t' | '\n' | '\r' | '\0' => continue,
						// 数字解析（支持整数和小数）
						'0'..='9' => {
							tokens.push(self.read_number(c));
						}
						// 标识符解析
						'a'..='z' | 'A'..='Z' | '_' => {
							tokens.push(self.read_identifier(c));
						}
						// 符号处理
						'+' => {
							if let Char::Char('=') = self.cursor.peek() {
								self.cursor.next();
								tokens.push(Token::Operator(Operator::PlusEqual));
								continue;
							}
							tokens.push(Token::Operator(Operator::Plus));
						}
						'-' => {
							if let Char::Char('=') = self.cursor.peek() {
								self.cursor.next();
								tokens.push(Token::Operator(Operator::MinusEqual));
								continue;
							}
							tokens.push(Token::Operator(Operator::Minus));
						}
						'*' => {
							if let Char::Char('=') = self.cursor.peek() {
								self.cursor.next();
								tokens.push(Token::Operator(Operator::StarEqual));
								continue;
							}
							tokens.push(Token::Operator(Operator::Star));
						}
						'/' => {
							if let Char::Char('/') = self.cursor.peek() {
								self.state.located = Struct::LineComment;
								self.cursor.next();
							} else if let Char::Char('*') = self.cursor.peek() {
								self.state.located = Struct::BlockComment;
								self.cursor.next();
							} else if let Char::Char('=') = self.cursor.peek() {
								self.cursor.next();
								tokens.push(Token::Operator(Operator::SlashEqual));
							} else {
								tokens.push(Token::Operator(Operator::Slash));
							}
						}
						'%' => {
							if let Char::Char('=') = self.cursor.peek() {
								self.cursor.next();
								tokens.push(Token::Operator(Operator::PercentEqual));
								continue;
							}
							tokens.push(Token::Operator(Operator::Percent));
						}
						// 对称符号
						'(' => tokens.push(Token::StructFlag(StructFlag::OpenParen)),
						'{' => {
							tokens.push(Token::StructFlag(StructFlag::OpenBrace));
							return Ok(tokens);
						}
						'<' => {
							if let Char::Char('=') = self.cursor.peek() {
								self.cursor.next();
								tokens.push(Token::Operator(Operator::LessEqual));
								continue;
							}
							tokens.push(Token::Temp(Temp::OpenAngle))
						}
						'[' => tokens.push(Token::StructFlag(StructFlag::OpenBracket)),
						')' => tokens.push(Token::StructFlag(StructFlag::CloseParen)),
						'}' => {
							tokens.push(Token::StructFlag(StructFlag::CloseBrace));
							return Ok(tokens);
						}
						'>' => {
							if let Char::Char('=') = self.cursor.peek() {
								self.cursor.next();
								tokens.push(Token::Operator(Operator::GreaterEqual));
								continue;
							}
							tokens.push(Token::Temp(Temp::CloseAngle))
						}
						']' => tokens.push(Token::StructFlag(StructFlag::CloseBracket)),
						'"' | '\'' | '`' => {
							let mut s = String::new();
							loop {
								if let Char::Char(char) = self.cursor.next() {
									match char {
										'\0' => {}
										'\'' => {
											if c == '\'' {
												tokens.push(Token::Value(Value::Char(s.chars().next().unwrap_or('\0'))));
												break;
											} else {
												s.push('\'');
											}
										}
										'"' => {
											if c == '"' {
												tokens.push(Token::Value(Value::String(s)));
												break;
											} else {
												s.push('"');
											}
										}
										'`' => {
											if c == '`' {
												tokens.push(Token::Value(Value::String(s)));
												break;
											} else {
												s.push('`');
											}
										}
										'\\' => {
											if let Char::Char(c) = self.cursor.next() {
												match c {
													'\\' => {
														self.cursor.next();
														s.push('\\');
													}
													'"' => {
														self.cursor.next();
														s.push('"');
													}
													'n' => {
														self.cursor.next();
														s.push('\n');
													}
													'r' => {
														self.cursor.next();
														s.push('\r');
													}
													't' => {
														self.cursor.next();
														s.push('\t');
													}
													'u' => {
														if let Char::Char('{') = self.cursor.next() {
															let mut temp = String::new();
															while let Char::Char(c) = self.cursor.next() {
																match c {
																	'}' => {
																		if let Ok(num) = u32::from_str_radix(&temp, 16) {
																			s.push(char::from_u32(num).unwrap());
																		} else {
																			self.throw_error("Unicode十六进制数解析失败".to_string(), None, 1, self.cursor.get_pointing());
																		}
																		break;
																	}
																	'0'..='9' | 'a'..='f' | 'A'..='F' => {
																		temp.push(c);
																	}
																	_ => {
																		self.throw_error(format!("'{}'不是十六进制字符", c), None, 1, self.cursor.get_pointing());
																	}
																}
															}
														} else {
															self.throw_error("'\\u'之后应该有大括号包裹的四位Unicode十六进制数".to_string(), None, 1, self.cursor.get_pointing());
														}
													}
													o => {
														self.throw_error(format!("未知的'\\{}'", o).to_string(), None, 1, self.cursor.get_pointing());
													}
												}
											}
										}
										_ => {
											s.push(char);
										}
									}
								} else if let Char::EndLine = self.cursor.next() {
									if c == '`' {
										s.push_str("\n\r");
										continue;
									}
								}
							}
						}
						// 其他符号
						':' => {
							if let Char::Char(':') = self.cursor.peek() {
								self.cursor.next();
								tokens.push(Token::StructFlag(StructFlag::Index));
							} else {
								tokens.push(Token::StructFlag(StructFlag::Colon));
							}
						}
						'&' => {
							if let Char::Char('&') = self.cursor.peek() {
								self.cursor.next();
								tokens.push(Token::Operator(Operator::And));
								continue;
							}
							tokens.push(Token::Operator(Operator::Reference));
						}
						'|' => {
							if let Char::Char('|') = self.cursor.peek() {
								self.cursor.next();
								tokens.push(Token::Operator(Operator::Or));
								continue;
							}
							tokens.push(Token::Operator(Operator::Vertical));
						}
						'@' => tokens.push(Token::Operator(Operator::AtStruct)),
						'$' => tokens.push(Token::Operator(Operator::GetStruct)),
						'?' => tokens.push(Token::Operator(Operator::Exist)),
						'!' => {
							if let Char::Char('=') = self.cursor.peek() {
								self.cursor.next();
								tokens.push(Token::Operator(Operator::NotIs));
								continue;
							}
							tokens.push(Token::Operator(Operator::Not));
						}
						'.' => tokens.push(Token::StructFlag(StructFlag::Dot)),
						'=' => {
							if let Char::Char('=') = self.cursor.peek() {
								tokens.push(Token::Operator(Operator::Is));
								self.cursor.next();
								continue;
							}
							tokens.push(Token::Operator(Operator::Move))
						}
						',' => tokens.push(Token::StructFlag(StructFlag::Comma)),
						';' => {
							tokens.push(Token::EndStatement);
							return Ok(tokens);
						}

						// 处理未知字符
						_ => {
							self.throw_error(format!("未知字符 '{}'", c), None, 1, self.cursor.get_pointing())
						}
					}
				}
				Char::EndLine => {
					if self.state.located == Struct::LineComment {
						self.state.located = Struct::External;
					}
				}
				Char::EndFile => {
					Log::new(LogType::Info("文件词法分析完成".green()), self.cursor.get_file().path().as_str()).print();
					return if tokens.len() == 0 {
						Result::Err(Err::EndFile)
					} else {
						Ok(tokens)
					};
				}
				Char::ErrFile => {
					Log::new(LogType::Err, format!("文件 {} 读取错误", self.cursor.get_file().path()).as_str()).throw(20);
				}
			}
		}
	}
	/// 读取连续数字字符
	fn read_number(&mut self, first: char) -> Token {
		fn to_number(s: &str, radix: u32) -> Result<Value, String> {
			match radix {
				2 | 8 | 16 => {
					u64::from_str_radix(s, radix)
						.map(|n| Value::NumIsize(n as isize))
						.map_err(|_| format!("无法将 '{}' 解析为 {} 进制数", s, radix))
				}
				10 => {
					f64::from_str(s)
						.map_err(|_| format!("无法将 '{}' 解析为浮点数", s))
						.and_then(|n| {
							if n.fract() == 0.0 && n >= isize::MIN as f64 && n <= isize::MAX as f64 {
								Ok(Value::NumIsize(n as isize))
							} else {
								Err(format!("'{}' 超出了 isize 的范围", s))
							}
						})
				}
				_ => Err(format!("不支持 {} 进制", radix)),
			}
		}

		let mut s = String::new();
		let start = self.cursor.get_pointing();
		let mut base = 10;
		if first == '0' {
			if let Char::Char(c) = self.cursor.peek() {
				match c {
					'b' | 'B' => base = 2,
					'o' | 'O' => base = 8,
					'x' | 'X' => base = 16,
					_ => s.push(first)
				}
			}
			if base != 10 {
				self.cursor.next();
			}
		}
		let mut result = true;
		let mut message = String::from("意外的变故，无法匹配为数字");
		while let Char::Char(c) = self.cursor.peek() {
			match c {
				'0' | '1' => {
					if base >= 2 {
						self.cursor.next();
						s.push(c)
					} else {
						message = format!("'{}'超出了{}进制的最大数字，应该进位", c, base);
						result = false;
						break;
					}
				}
				'2'..='7' => {
					if base >= 8 {
						self.cursor.next();
						s.push(c)
					} else {
						message = format!("'{}'超出了{}进制的最大数字，应该进位", c, base);
						result = false;
						break;
					}
				}
				'8' | '9' => {
					if base >= 10 {
						self.cursor.next();
						s.push(c)
					} else {
						message = format!("'{}'超出了{}进制的最大数字，应该进位", c, base);
						result = false;
						break;
					}
				}
				'a'..='d' | 'A'..='D' | 'f' | 'F' => {
					if base >= 16 {
						self.cursor.next();
						s.push(c)
					} else {
						message = format!("'{}'超出了{}进制的最大数字，应该进位", c, base);
						result = false;
						break;
					}
				}
				'e' | 'E' => {
					if base >= 15 || base == 10 {
						self.cursor.next();
						s.push(c)
					} else {
						if base < 15 {
							message = format!("'{}'超出了{}进制的最大数字，应该进位", c, base);
						} else {
							message = "除十进制外，其他进制不支持指数".to_string();
						}
						result = false;
						break;
					}
				}
				'_' => {
					self.cursor.next();
				}
				'.' => {
					if base == 10 {
						self.cursor.next();
						s.push(c)
					} else {
						message = "除十进制外，其他进制不支持小数点".to_string();
						result = false;
						break;
					}
				}
				'-' => {
					if base == 10 {
						self.cursor.next();
						s.push(c)
					} else {
						message = "除十进制外，其他进制不支持减号".to_string();
						result = false;
						break;
					}
				}
				_ => {
					break;
				}
			}
		}
		let end = self.cursor.get_pointing();
		if result {
			match to_number(s.as_str(), base) {
				Ok(value) => Token::Value(value),
				Err(err_msg) => {
					self.throw_error(message, Some(err_msg), end.num_column - start.num_column, start);
				}
			}
		} else {
			self.throw_error(message, None, end.num_column - start.num_column, start);
		}
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

/// 词法分析结果
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
	/// 字面量
	Value(Value),
	/// 标识符
	Ident(String),
	/// 操作符
	Operator(Operator),
	/// 结构分隔符
	StructFlag(StructFlag),
	/// 暂定标记
	Temp(Temp),
	/// 语句结束标记
	EndStatement,
}
/// 暂定标记
#[derive(Debug, Clone, PartialEq)]
pub enum Temp {
	/// <
	OpenAngle,
	/// >
	CloseAngle,
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
	/// [
	OpenBracket,
	/// ]
	CloseBracket,
	/// ::
	Index,
	/// :
	Colon,
	/// ,
	Comma,
	/// .
	Dot,
}
/// 操作符
#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
	/// +
	Plus,
	/// +=
	PlusEqual,
	/// -
	Minus,
	/// -=
	MinusEqual,
	/// *
	Star,
	/// *=
	StarEqual,
	/// /
	Slash,
	/// /=
	SlashEqual,
	/// %
	Percent,
	/// %=
	PercentEqual,
	/// = 移动
	Move,
	/// & 引用
	Reference,
	/// @ 结构引用
	AtStruct,
	/// $ 要求结构应用
	GetStruct,
	/// | 竖线
	Vertical,
	/// &&
	And,
	/// ||
	Or,
	/// !
	Not,
	/// !=
	NotIs,
	/// ==
	Is,
	/// >
	Greater,
	/// <
	Less,
	/// >=
	GreaterEqual,
	/// <=
	LessEqual,
	/// ? 判断有效
	Exist,
}