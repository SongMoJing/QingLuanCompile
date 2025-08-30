use std::cmp::PartialEq;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::str::FromStr;
use colored::Colorize;
use unicode_width::{UnicodeWidthChar};
use crate::_lib::io::{Char, Cursor, CursorPointing, FileWrapper, Log, LogType};
use crate::parser::qln::lexer::token::*;

pub(crate) mod token;

// 词法错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum LexError {
	InvalidEscape(char),
	InvalidChar(char),
	UnclosedString,
	InvalidUnicode,
	MalformedNumber,
	UnexpectedEof,
	MalformedChar,
}

pub struct Lexer {
	cursor: Cursor,
	current_char: Char,
	tokens: Vec<Token>,
	errors: Vec<(LexError, Span)>,
}

impl PartialEq for Char {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Char::Char(c1), Char::Char(c2)) => c1 == c2,
			(Char::EndFile, Char::EndFile) => true,
			(Char::ErrFile, Char::ErrFile) => true,
			(Char::EndLine, Char::EndLine) => true,
			_ => false,
		}
	}
}

impl Lexer {
	pub fn new(file_path: &Path) -> Result<Lexer, Error> {
		let file_wrapper = FileWrapper::new(file_path.to_string_lossy().into_owned());

		if !file_wrapper.works() {
			return Err(Error::from(ErrorKind::NotFound));
		}

		if let Some(mut cursor) = Cursor::new(file_wrapper) {
			let current_char = cursor.next();

			Ok(Lexer {
				cursor,
				current_char,
				tokens: Vec::new(),
				errors: Vec::new(),
			})
		} else {
			Err(Error::from(ErrorKind::Other))
		}
	}

	pub fn tokenize(mut self) -> (Vec<Token>, Vec<(LexError, Span)>) {
		while !matches!(self.current_char, Char::EndFile | Char::ErrFile) {
			self.skip_whitespace();

			let start_pos = self.cursor.get_pointing();

			match self.current_char {
				Char::Char(_) => {
					if self.tokenize_marching(&start_pos) {
						// 记录token的位置信息
						if let Some(last_token) = self.tokens.last_mut() {
							let end_pos = self.cursor.get_pointing();
							last_token.span = Span {
								start_line: start_pos.num_line,
								start_col: start_pos.num_column,
								end_line: end_pos.num_line,
								end_col: end_pos.num_column,
							};
						}
					}
				}
				Char::EndLine => {
					self.next(); // 跳过换行符
					continue;
				}
				Char::EndFile | Char::ErrFile => break,
			}
		}

		// 添加文件结束标记
		let pos = self.cursor.get_pointing();
		self.tokens.push(Token {
			kind: TokenKind::EOF,
			span: Span {
				start_line: pos.num_line,
				start_col: pos.num_column,
				end_line: pos.num_line,
				end_col: pos.num_column,
			},
		});

		(self.tokens, self.errors)
	}

	fn tokenize_block(&mut self) -> (Vec<Token>, Vec<(LexError, Span)>) {
		let save = (self.tokens.clone(), self.errors.clone());
		self.tokens.clear();
		self.errors.clear();
		while !matches!(self.current_char, Char::EndFile | Char::ErrFile) {
			self.skip_whitespace();
			let mut can_break: u8 = 1;
			let start_pos = self.cursor.get_pointing();

			match self.current_char {
				Char::Char(c) => {
					match c {
						'{' => {
							can_break += 1;
						}
						'}' => {
							can_break -= 1;
							if can_break == 0 {
								break;
							}
						}
						_ => {}
					}
					if self.tokenize_marching(&start_pos) {
						// 记录token的位置信息
						if let Some(last_token) = self.tokens.last_mut() {
							let end_pos = self.cursor.get_pointing();
							last_token.span = Span {
								start_line: start_pos.num_line,
								start_col: start_pos.num_column,
								end_line: end_pos.num_line,
								end_col: end_pos.num_column,
							};
						}
					}
				}
				Char::EndLine => {
					self.next(); // 跳过换行符
					continue;
				}
				Char::EndFile | Char::ErrFile => break,
			}
		}

		// 添加文件结束标记
		let pos = self.cursor.get_pointing();
		self.tokens.push(Token {
			kind: TokenKind::EOF,
			span: Span {
				start_line: pos.num_line,
				start_col: pos.num_column,
				end_line: pos.num_line,
				end_col: pos.num_column,
			},
		});
		let res = (self.tokens.clone(), self.errors.clone());
		self.tokens = save.0;
		self.errors = save.1;
		res
	}

	fn tokenize_marching(&mut self, start_pos: &CursorPointing) -> bool {
		if let Char::Char(c) = self.current_char {
			match c {
				// 处理标识符和关键字
				c if c.is_alphabetic() || c == '_' => self.scan_ident_or_keyword(),

				// 处理数字字面量
				c if c.is_ascii_digit() => self.scan_number(),

				// 处理字符串字面量
				'"' => self.scan_string(),
				'\'' => self.scan_char(),

				// 处理符号
				'(' => self.add_token(TokenKind::LParen, false),
				')' => self.add_token(TokenKind::RParen, false),
				'{' => self.add_token(TokenKind::LBrace, false),
				'}' => self.add_token(TokenKind::RBrace, false),
				'[' => self.add_token(TokenKind::LBracket, false),
				']' => self.add_token(TokenKind::RBracket, false),
				';' => self.add_token(TokenKind::Semicolon, false),
				':' => self.add_token(TokenKind::Colon, false),
				',' => self.add_token(TokenKind::Comma, false),
				'.' => self.add_token(TokenKind::Dot, false),
				'^' => self.add_token(TokenKind::Caret, false),
				'?' => self.add_token(TokenKind::Question, false),

				'=' => self.scan_double(vec![('=', TokenKind::Eq)], TokenKind::Assign),
				'!' => self.scan_double(vec![('=', TokenKind::Ne)], TokenKind::Exclamation),
				'<' => self.scan_double(vec![('=', TokenKind::Le)], TokenKind::Lt),
				'>' => self.scan_double(vec![('=', TokenKind::Ge)], TokenKind::Gt),
				'&' => self.scan_double(vec![('&', TokenKind::And)], TokenKind::Ampersand),
				'|' => self.scan_double(vec![('|', TokenKind::Or)], TokenKind::Pipe),
				'+' => self.scan_double(vec![('=', TokenKind::PlusAssign)], TokenKind::Plus),
				'-' => self.scan_double(vec![('=', TokenKind::MinusAssign), ('>', TokenKind::Arrow)], TokenKind::Minus),
				'*' => self.scan_double(vec![('=', TokenKind::StarAssign)], TokenKind::Star),
				'%' => self.scan_double(vec![('=', TokenKind::ModAssign)], TokenKind::Percent),
				'/' => {
					return self.scan_comment(start_pos);
				}

				// 错误字符处理
				_ => {
					let pos = self.cursor.get_pointing();
					self.errors.push((LexError::InvalidChar(c), Span {
						start_line: start_pos.num_line,
						start_col: start_pos.num_column,
						end_line: pos.num_line,
						end_col: pos.num_column,
					}));
					self.next(); // 跳过无效字符继续
				}
			}
		}
		true
	}

	fn skip_whitespace(&mut self) {
		while let Char::Char(c) = self.current_char {
			if c.is_whitespace() {
				self.next();
			} else {
				break;
			}
		}
	}

	fn next(&mut self) {
		self.current_char = self.cursor.next();
	}

	fn add_token(&mut self, kind: TokenKind, keep_char: bool) {
		let pos = self.cursor.get_pointing();
		self.tokens.push(Token {
			kind,
			span: Span {
				start_line: pos.num_line,
				start_col: pos.num_column,
				end_line: pos.num_line,
				end_col: pos.num_column,
			},
		});
		if !keep_char {
			self.next();
		}
	}

	fn scan_ident_or_keyword(&mut self) {
		let start_pos = self.cursor.get_pointing();
		let mut ident = String::new();

		while let Char::Char(c) = self.current_char {
			if c.is_alphanumeric() || c == '_' {
				ident.push(c);
				self.next();
			} else {
				break;
			}
		}

		// 识别关键字
		let kind = match ident.as_str() {
			"import" => TokenKind::Key(Key::StructKey(KeyStruct::Import)),
			"mod" => TokenKind::Key(Key::StructKey(KeyStruct::Mod)),
			"class" => TokenKind::Key(Key::StructKey(KeyStruct::Class)),
			"attr" => TokenKind::Key(Key::StructKey(KeyStruct::Attr)),
			"static" => TokenKind::Key(Key::StructKey(KeyStruct::Static)),
			"init" => TokenKind::Key(Key::StructKey(KeyStruct::Init)),
			"fn" => TokenKind::Key(Key::StructKey(KeyStruct::Fn)),

			"true" => TokenKind::Bool(true),
			"false" => TokenKind::Bool(false),

			"if" => TokenKind::Key(Key::LogicControl(KeyLogicControl::If)),
			"else" => TokenKind::Key(Key::LogicControl(KeyLogicControl::Else)),
			"for" => TokenKind::Key(Key::LogicControl(KeyLogicControl::For)),
			"while" => TokenKind::Key(Key::LogicControl(KeyLogicControl::While)),
			"loop" => TokenKind::Key(Key::LogicControl(KeyLogicControl::Loop)),
			"match" => TokenKind::Key(Key::LogicControl(KeyLogicControl::Match)),

			"continue" => TokenKind::Key(Key::SequenceControl(KeySequenceControl::Continue)),
			"break" => TokenKind::Key(Key::SequenceControl(KeySequenceControl::Break)),
			"return" => TokenKind::Key(Key::SequenceControl(KeySequenceControl::Return)),

			"let" => TokenKind::Key(Key::ObjectAllocation(KeyObjectAllocation::Let)),
			"var" => TokenKind::Key(Key::ObjectAllocation(KeyObjectAllocation::Var)),
			"const" => TokenKind::Key(Key::ObjectAllocation(KeyObjectAllocation::Const)),

			"new" => TokenKind::Key(Key::MemoryAllocation(KeyMemoryAllocation::New)),

			"self" => TokenKind::Key(Key::ObjectAccess(KeyObjectAccess::ToSelf)),
			"super" => TokenKind::Key(Key::ObjectAccess(KeyObjectAccess::ToParent)),

			_ => TokenKind::Ident(ident),
		};

		self.tokens.push(Token {
			kind,
			span: Span {
				start_line: start_pos.num_line,
				start_col: start_pos.num_column,
				end_line: self.cursor.get_pointing().num_line,
				end_col: self.cursor.get_pointing().num_column,
			},
		});
	}

	fn scan_number(&mut self) {
		let start_pos = self.cursor.get_pointing();
		let mut num_str = String::new();
		let mut radix = 10;
		let mut is_float = false;

		// 处理进制前缀
		if let Char::Char('0') = self.current_char {
			num_str.push('0');
			self.next();
			// 2
			if let Char::Char('b') = self.current_char {
				num_str.push('b');
				radix = 2;
				self.next();
			}
			// 8
			if let Char::Char('o') = self.current_char {
				num_str.push('o');
				radix = 8;
				self.next();
			}
			// 16
			if let Char::Char('x') = self.current_char {
				num_str.push('x');
				radix = 16;
				self.next();
			}
		}

		while let Char::Char(c) = self.current_char {
			// 字母数字下划线
			if c.is_ascii_alphabetic() || c.is_ascii_digit() || c == '_' || c == '.' {
				if c == '.' && !is_float {
					is_float = true;
				}
				num_str.push(c);
				self.next();
			} else {
				break;
			}
		}

		// 解析数字
		let kind = if is_float {
			match f64::from_str(if radix != 10 { &num_str[2..] } else { &num_str }) {
				Ok(num) => TokenKind::NumFloat(num),
				Err(_) => {
					let pos = self.cursor.get_pointing();
					self.errors.push((LexError::MalformedNumber, Span {
						start_line: start_pos.num_line,
						start_col: start_pos.num_column,
						end_line: pos.num_line,
						end_col: pos.num_column,
					}));
					TokenKind::NumFloat(0.0)
				}
			}
		} else {
			match i64::from_str_radix(if radix != 10 { &num_str[2..] } else { &num_str }, radix) {
				Ok(num) => TokenKind::NumInt(num),
				Err(_) => {
					let pos = self.cursor.get_pointing();
					self.errors.push((LexError::MalformedNumber, Span {
						start_line: start_pos.num_line,
						start_col: start_pos.num_column,
						end_line: pos.num_line,
						end_col: pos.num_column,
					}));
					TokenKind::NumInt(0) // 占位值
				}
			}
		};

		self.tokens.push(Token {
			kind,
			span: Span {
				start_line: start_pos.num_line,
				start_col: start_pos.num_column,
				end_line: self.cursor.get_pointing().num_line,
				end_col: self.cursor.get_pointing().num_column,
			},
		});
	}

	fn scan_string(&mut self) {
		let start_pos = self.cursor.get_pointing();
		self.next(); // 跳过起始引号
		// 扫描字符串拼接结果
		let mut string_vec: Vec<CharacterString> = Vec::new();
		// 暂存字符串扫描结果
		let mut temp_string = String::new();
		while !matches!(self.current_char, Char::EndFile | Char::ErrFile) {
			match &self.current_char {
				Char::Char('"') => {
					self.next(); // 跳过结束引号
					string_vec.push(CharacterString::String(temp_string.clone()));
					temp_string.clear();
					break;
				}
				Char::Char('\\') => {
					self.next();
					if let Char::Char(c) = self.current_char {
						if c == 'u' {
							if let Some(c) = self.scan_unicode() {
								temp_string.push(c);
								continue;
							}
						} else if let Some(c) = self.scan_escape() {
							temp_string.push(c);
						}
						self.next();
					}
				}
				Char::Char('$') => {
					string_vec.push(CharacterString::String(temp_string.clone()));
					temp_string.clear();
					self.next();
					if let Char::Char('{') = self.current_char {
						self.next(); // 跳过 '{'
						let mut res = self.tokenize_block();
						if !res.1.is_empty() {
							self.errors.append(&mut res.1);
						}
						self.next(); // 跳过 '}'
						string_vec.push(CharacterString::Implant(res.0));
					} else {
						let pos = self.cursor.get_pointing();
						self.errors.push((LexError::InvalidChar('$'), Span {
							start_line: pos.num_line,
							start_col: pos.num_column,
							end_line: pos.num_line,
							end_col: pos.num_column,
						}));
					}
				}
				Char::Char(c) => {
					temp_string.push(*c);
					self.next();
				}
				Char::EndLine => {
					temp_string.push_str("\r\n");
					self.next();
				}
				Char::EndFile | Char::ErrFile => {
					let pos = self.cursor.get_pointing();
					self.errors.push((LexError::UnclosedString, Span {
						start_line: pos.num_line,
						start_col: pos.num_column,
						end_line: pos.num_line,
						end_col: pos.num_column,
					}));
					break;
				}
			}
		}
		self.tokens.push(Token {
			kind: TokenKind::String(string_vec),
			span: Span {
				start_line: start_pos.num_line,
				start_col: start_pos.num_column,
				end_line: self.cursor.get_pointing().num_line,
				end_col: self.cursor.get_pointing().num_column,
			},
		});
	}

	fn scan_char(&mut self) {
		let start_pos = self.cursor.get_pointing();
		self.next();
		let mut char_str = String::new();
		while let Char::Char(c) = self.current_char {
			if c == '\'' {
				self.next(); // 跳过结束引号
				break;
			} else if c == '\\' {
				self.next();
				if let Char::Char(c) = self.current_char {
					if c == 'u' {
						if let Some(c) = self.scan_unicode() {
							char_str.push(c);
							continue;
						}
					} else if let Some(c) = self.scan_escape() {
						char_str.push(c);
					}
				}
			} else {
				char_str.push(c);
			}
			self.next();
		}
		if char_str.chars().count() > 1 {
			let pos = self.cursor.get_pointing();
			self.errors.push((LexError::MalformedChar, Span {
				start_line: start_pos.num_line,
				start_col: start_pos.num_column,
				end_line: pos.num_line,
				end_col: pos.num_column,
			}))
		}
		self.tokens.push(Token {
			kind: TokenKind::Char(char_str.chars().next().unwrap_or('\0')),
			span: Span {
				start_line: start_pos.num_line,
				start_col: start_pos.num_column,
				end_line: self.cursor.get_pointing().num_line,
				end_col: self.cursor.get_pointing().num_column,
			},
		});
	}

	fn scan_unicode(&mut self) -> Option<char> {
		let start_pos = self.cursor.get_pointing();

		if let Char::Char('u') = self.current_char {
			self.next(); // 跳过 'u'

			if let Char::Char('{') = self.current_char {
				self.next(); // 跳过 '{'

				let mut hex_str = String::new();

				while let Char::Char(c) = self.current_char {
					if c == '}' {
						self.next(); // 跳过 '}'
						// 添加Unicode转义标记
						return char::from_u32(u32::from_str_radix(&hex_str, 16).unwrap_or_else(|_| {
							let pos = self.cursor.get_pointing();
							self.errors.push((LexError::InvalidUnicode, Span {
								start_line: start_pos.num_line,
								start_col: start_pos.num_column,
								end_line: pos.num_line,
								end_col: pos.num_column,
							}));
							0
						}));
					} else {
						hex_str.push(c);
						self.next();
					}
				}
			} else {
				let pos = self.cursor.get_pointing();
				self.errors.push((LexError::InvalidUnicode, Span {
					start_line: start_pos.num_line,
					start_col: start_pos.num_column,
					end_line: pos.num_line,
					end_col: pos.num_column,
				}));
			}
		} else {
			let pos = self.cursor.get_pointing();
			self.errors.push((LexError::InvalidUnicode, Span {
				start_line: start_pos.num_line,
				start_col: start_pos.num_column,
				end_line: pos.num_line,
				end_col: pos.num_column,
			}));
		}
		None
	}

	/// 扫描1~2个字符的键
	fn scan_double(&mut self, key_token_last: Vec<(char, TokenKind)>, def: TokenKind) {
		self.next();
		let mut iter = key_token_last.iter();
		if let Char::Char(c) = self.current_char {
			while let Some(kv) = iter.next() {
				if kv.0 == c {
					self.add_token(kv.clone().1, false);
					return;
				}
			}
		}
		self.add_token(def, true);
	}

	fn scan_escape(&mut self) -> Option<char> {
		let start_pos = self.cursor.get_pointing();
		return match &self.current_char {
			Char::Char('n') => Some('\n'),
			Char::Char('r') => Some('\r'),
			Char::Char('t') => Some('\t'),
			Char::Char('\\') => Some('\\'),
			Char::Char('"') => Some('"'),
			Char::Char('\'') => Some('\''),
			c => {
				let pos = self.cursor.get_pointing();
				if let Char::Char(e) = c {
					self.errors.push((LexError::InvalidEscape(*e), Span {
						start_line: start_pos.num_line,
						start_col: start_pos.num_column,
						end_line: pos.num_line,
						end_col: pos.num_column,
					}));
				}
				None
			}
		};
	}

	fn scan_comment(&mut self, start_pos: &CursorPointing) -> bool {
		// 跳过第一个 '/' (当前字符)
		self.next();

		match self.current_char {
			// 行注释 "//"
			Char::Char('/') => {
				self.next(); // 跳过第二个 '/'

				// 跳过直到行尾或文件结束
				while !matches!(
                    self.current_char,
                    Char::EndLine | Char::EndFile | Char::ErrFile
                ) {
					self.next();
				}
			}

			// 块注释 "/*"
			Char::Char('*') => {
				self.next(); // 跳过 '*'
				let mut nesting_level = 1; // 嵌套层级计数器

				while nesting_level > 0 {
					match self.current_char {
						Char::EndFile | Char::ErrFile => {
							let pos = self.cursor.get_pointing();
							self.errors.push((LexError::UnexpectedEof, Span {
								start_line: start_pos.num_line,
								start_col: start_pos.num_column,
								end_line: pos.num_line,
								end_col: pos.num_column,
							}));
							return false;
						}

						Char::Char('/') => {
							self.next();
							if let Char::Char('*') = self.current_char {
								nesting_level += 1; // 遇到 "/*" 嵌套增加
								self.next(); // 跳过 '*'
							}
						}

						Char::Char('*') => {
							self.next();
							if let Char::Char('/') = self.current_char {
								nesting_level -= 1; // 遇到 "*/" 嵌套减少
								self.next(); // 跳过 '/'
							}
						}

						// 跳过其他字符
						_ => self.next(),
					}
				}
			}

			// /=
			Char::Char('=') => {
				self.tokens.push(Token {
					kind: TokenKind::SlashAssign,
					span: Span {
						start_line: start_pos.num_line,
						start_col: start_pos.num_column,
						end_line: self.cursor.get_pointing().num_line,
						end_col: self.cursor.get_pointing().num_column,
					},
				});
				return true;
			}

			// 单个 '/' (不是注释)
			_ => {
				self.tokens.push(Token {
					kind: TokenKind::Slash,
					span: Span {
						start_line: start_pos.num_line,
						start_col: start_pos.num_column,
						end_line: self.cursor.get_pointing().num_line,
						end_col: self.cursor.get_pointing().num_column,
					},
				});
				return true;
			}
		}
		false
	}
}

/// # 错误报告器
pub struct ErrorReporter {
	source: String,
	file: String,
	errors: Vec<(LexError, Span)>,
}

impl ErrorReporter {
	pub fn new(source: &str, file: &Path) -> Self {
		ErrorReporter {
			source: source.to_string(),
			file: file.to_string_lossy().to_string().replace("\\", "/"),
			errors: Vec::new(),
		}
	}

	pub fn add_error(&mut self, error: LexError, span: Span) {
		self.errors.push((error, span));
	}

	pub fn report(&self) {
		for (error, span) in &self.errors {
			let line_num = span.start_line;
			let line_str = self.get_line_content(line_num - 1);

			// Rust 风格错误格式
			let message = match error {
				LexError::InvalidChar(c) => format!("意外的字符: '{}'", c),
				LexError::UnclosedString => "字符串未闭合".to_string(),
				LexError::InvalidUnicode => "无效的Unicode转义序列".to_string(),
				LexError::MalformedNumber => "数字格式错误".to_string(),
				LexError::UnexpectedEof => "文件意外结束".to_string(),
				LexError::InvalidEscape(c) => format!("无法转义: '{}'", c),
				LexError::MalformedChar => "无效的字符".to_string(),
			};

			let line_num = line_num.to_string();
			Log::new(
				LogType::Err,
				&format!(
					"[E{:04}] {} \n{}{} {}:{}:{}",
					self.error_code(error),
					message,
					" ".repeat(line_num.len()),
					"-->".bright_cyan(),
					self.file,
					line_num,
					span.start_col
				),
			).print();
			println!(" {} {}",
					 " ".repeat(line_num.len()),
					 "|".bright_cyan()
			);
			println!(" {} {} {}",
					 line_num.bright_cyan(),
					 "|".bright_cyan(),
					 line_str.replace("\t", " ")
			);
			println!(" {} {}{}{}",
					 " ".repeat(line_num.len()),
					 "|".bright_cyan(),
					 " ".repeat(get_len(&line_str.replace("\t", " "), span.start_col)),
					 "^".repeat(span.end_col.saturating_sub(span.start_col) + 1).bright_yellow()
			);
		}

		fn get_len(line_content: &str, col: usize) -> usize {
			line_content.chars()
				.take(col)
				.map(|c| c.width().unwrap_or(1)) // 全角字符宽度为2
				.sum()
		}
	}

	fn get_line_content(&self, line_num: usize) -> &str {
		self.source.lines().nth(line_num).unwrap_or("")
	}

	fn error_code(&self, error: &LexError) -> u16 {
		match error {
			LexError::InvalidChar(_) => 1001,
			LexError::UnclosedString => 1002,
			LexError::InvalidUnicode => 1003,
			LexError::MalformedNumber => 1004,
			LexError::UnexpectedEof => 1005,
			LexError::InvalidEscape(_) => 1006,
			LexError::MalformedChar => 1007,
		}
	}
}