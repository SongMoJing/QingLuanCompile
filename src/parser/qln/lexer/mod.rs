use std::cmp::PartialEq;
use std::io::{Error, ErrorKind};
use std::path::Path;
use colored::Colorize;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use crate::_lib::io::{Char, Cursor, FileWrapper, Log, LogType};
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

		let mut cursor = Cursor::new(file_wrapper);
		let current_char = cursor.next();

		Ok(Lexer {
			cursor,
			current_char,
			tokens: Vec::new(),
			errors: Vec::new(),
		})
	}

	pub fn tokenize(mut self) -> (Vec<Token>, Vec<(LexError, Span)>) {
		while !matches!(self.current_char, Char::EndFile | Char::ErrFile) {
			self.skip_whitespace();

			let start_pos = self.cursor.get_pointing();

			match &self.current_char {
				Char::Char(c) => {
					match c {
						// 处理标识符和关键字
						c if c.is_alphabetic() || *c == '_' => self.scan_ident_or_keyword(),

						// 处理数字字面量
						c if c.is_ascii_digit() => self.scan_number(),

						// 处理字符串字面量
						'"' => self.scan_string(),

						// 处理符号
						'(' => self.add_token(TokenKind::LParen),
						')' => self.add_token(TokenKind::RParen),
						'{' => self.add_token(TokenKind::LBrace),
						'}' => self.add_token(TokenKind::RBrace),
						'[' => self.add_token(TokenKind::LBracket),
						']' => self.add_token(TokenKind::RBracket),
						';' => self.add_token(TokenKind::Semicolon),
						':' => self.add_token(TokenKind::Colon),
						',' => self.add_token(TokenKind::Comma),
						'.' => self.add_token(TokenKind::Dot),
						'%' => self.add_token(TokenKind::Percent),
						'^' => self.add_token(TokenKind::Caret),
						'?' => self.add_token(TokenKind::Question),

						'=' => self.scan_equal(),
						'!' => self.scan_exclamation(),
						'<' => self.scan_less_than(),
						'>' => self.scan_greater_than(),
						'&' => self.scan_ampersand(),
						'|' => self.scan_pipe(),
						'+' => self.scan_plus(),
						'-' => self.scan_minus(),
						'*' => self.scan_star(),
						'/' => self.scan_comment(),

						// 错误字符处理
						_ => {
							let pos = self.cursor.get_pointing();
							self.errors.push((LexError::InvalidChar(*c), Span {
								start_line: pos.num_line,
								start_col: pos.num_column,
								end_line: pos.num_line,
								end_col: pos.num_column,
							}));
							self.next(); // 跳过无效字符继续
						}
					}
				}
				Char::EndLine => {
					self.next(); // 跳过换行符
				}
				Char::EndFile | Char::ErrFile => break,
			}

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

	fn add_token(&mut self, kind: TokenKind) {
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
		self.next();
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
			if c.is_digit(radix) || c == '_' {
				num_str.push(c);
				self.next();
			} else {
				break;
			}
		}

		// 解析数字
		let kind = match i64::from_str_radix(&num_str[2..], 16) {
			Ok(num) => TokenKind::NumInt(num),
			Err(_) => {
				let pos = self.cursor.get_pointing();
				self.errors.push((LexError::MalformedNumber, Span {
					start_line: pos.num_line,
					start_col: pos.num_column,
					end_line: pos.num_line,
					end_col: pos.num_column,
				}));
				TokenKind::NumInt(0) // 占位值
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
							if let Some(c) = self.scan_escape() {
								temp_string.push(c);
							}
							continue
						}
						if let Some(c) = self.handle_escape() {
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
						let mut implant_vec: Vec<Token> = Vec::new();
						// TODO
						while Char::Char('}') != self.current_char {
							self.next();
						}
						self.next();
						string_vec.push(CharacterString::Implant(implant_vec.clone()));
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
				Char::EndLine | Char::EndFile | Char::ErrFile => {
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

	fn scan_escape(&mut self) -> Option<char> {
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
								start_col: start_pos.num_column -1,
								end_line: pos.num_line,
								end_col: pos.num_column -1,
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
					start_col: start_pos.num_column -1,
					end_line: pos.num_line,
					end_col: pos.num_column,
				}));
			}
		} else {
			let pos = self.cursor.get_pointing();
			self.errors.push((LexError::InvalidUnicode, Span {
				start_line: start_pos.num_line,
				start_col: start_pos.num_column -1,
				end_line: pos.num_line,
				end_col: pos.num_column,
			}));
		}
		None
	}

	fn scan_equal(&mut self) {
		self.next(); // 跳过 '='
		if let Char::Char('=') = self.current_char {
			self.next(); // 跳过第二个 '='
			self.add_token(TokenKind::Eq);
		} else {
			self.add_token(TokenKind::Assign);
		}
	}

	fn scan_exclamation(&mut self) {
		self.next(); // 跳过 '!'
		if let Char::Char('=') = self.current_char {
			self.next(); // 跳过 '='
			self.add_token(TokenKind::Ne); // 需要添加 TokenKind::Ne
		} else {
			self.add_token(TokenKind::Exclamation);
		}
	}

	fn scan_less_than(&mut self) {
		self.next(); // 跳过 '<'
		if let Char::Char('=') = self.current_char {
			self.next(); // 跳过 '='
			self.add_token(TokenKind::Le); // 需要添加 TokenKind::Le
		} else {
			self.add_token(TokenKind::Lt);
		}
	}

	fn scan_greater_than(&mut self) {
		self.next(); // 跳过 '>'
		if let Char::Char('=') = self.current_char {
			self.next(); // 跳过 '='
			self.add_token(TokenKind::Ge); // 需要添加 TokenKind::Ge
		} else {
			self.add_token(TokenKind::Gt);
		}
	}

	fn scan_ampersand(&mut self) {
		self.next(); // 跳过 '&'
		if let Char::Char('&') = self.current_char {
			self.next(); // 跳过第二个 '&'
			self.add_token(TokenKind::And); // 需要添加 TokenKind::And
		} else {
			self.add_token(TokenKind::Ampersand);
		}
	}

	fn scan_pipe(&mut self) {
		self.next(); // 跳过 '|'
		if let Char::Char('|') = self.current_char {
			self.next(); // 跳过第二个 '|'
			self.add_token(TokenKind::Or); // 需要添加 TokenKind::Or
		} else {
			self.add_token(TokenKind::Pipe);
		}
	}

	fn scan_plus(&mut self) {
		self.next(); // 跳过 '+'
		if let Char::Char('=') = self.current_char {
			self.next(); // 跳过 '='
			self.add_token(TokenKind::PlusAssign); // 需要添加 TokenKind::PlusAssign
		} else {
			self.add_token(TokenKind::Plus);
		}
	}

	fn scan_minus(&mut self) {
		self.next(); // 跳过 '-'
		if let Char::Char('>') = self.current_char {
			self.next(); // 跳过 '>'
			self.add_token(TokenKind::Arrow);
		} else if let Char::Char('=') = self.current_char {
			self.next(); // 跳过 '='
			self.add_token(TokenKind::MinusAssign); // 需要添加 TokenKind::MinusAssign
		} else {
			self.add_token(TokenKind::Minus);
		}
	}

	fn scan_star(&mut self) {
		self.next(); // 跳过 '*'
		if let Char::Char('=') = self.current_char {
			self.next(); // 跳过 '='
			self.add_token(TokenKind::StarAssign); // 需要添加 TokenKind::StarAssign
		} else {
			self.add_token(TokenKind::Star);
		}
	}

	fn handle_escape(&mut self) {
		let start_pos = self.cursor.get_pointing();
		match &self.current_char {
			Char::Char('n') => self.add_token(TokenKind::Char('\n')),
			Char::Char('r') => self.add_token(TokenKind::Char('\r')),
			Char::Char('t') => self.add_token(TokenKind::Char('\t')),
			Char::Char('\\') => self.add_token(TokenKind::Char('\\')),
			Char::Char('"') => self.add_token(TokenKind::Char('"')),
			Char::Char('\'') => self.add_token(TokenKind::Char('\'')),
			c => {
				let pos = self.cursor.get_pointing();
				if let Char::Char(e) = c {
					self.errors.push((LexError::InvalidEscape(*e), Span {
						start_line: start_pos.num_line,
						start_col: start_pos.num_column - 1,
						end_line: pos.num_line,
						end_col: pos.num_column,
					}));
				}
			}
		}
	}

	fn scan_comment(&mut self) {
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
								start_line: pos.num_line,
								start_col: pos.num_column,
								end_line: pos.num_line,
								end_col: pos.num_column,
							}));
							return;
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

			// 单个 '/' (不是注释)
			_ => {
				self.tokens.push(Token {
					kind: TokenKind::Slash,
					span: Span {
						start_line: self.cursor.get_pointing().num_line,
						start_col: self.cursor.get_pointing().num_column,
						end_line: self.cursor.get_pointing().num_line,
						end_col: self.cursor.get_pointing().num_column,
					},
				});
			}
		}
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
			println!(" {} {}", " ".repeat(line_num.len()), "|".bright_cyan());
			println!(" {} {} {}", line_num.bright_cyan(), "|".bright_cyan(), line_str.replace("\t", " "));
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
		}
	}
}