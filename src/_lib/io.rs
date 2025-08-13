use std::fs;
use std::io;
use std::io::{BufRead, ErrorKind};
use std::process::exit;

use colored::{ColoredString, Colorize};

/// # 文件读取器
pub struct FileWrapper {
	works: bool,
	path: String,
	file: Option<io::BufReader<fs::File>>,
}

impl FileWrapper {
	pub fn new(path: String) -> Self {
		let file = fs::File::open(&path);
		Self {
			works: file.is_ok(),
			path: (&path).to_string(),
			file: Some(io::BufReader::new(match file {
				Ok(file) => file,
				Err(_) => return Self {
					works: false,
					path,
					file: None,
				},
			}
			)),
		}
	}

	/// ## 对象正常
	pub fn works(&self) -> bool {
		self.works
	}

	/// ## 获取文件名称
	pub fn path(&self) -> String {
		self.path.clone()
	}

	/// ## 读取下一行
	pub fn next(&mut self) -> Option<io::Result<String>> {
		if self.works {
			self.file.as_mut().unwrap().lines().next()
		} else {
			None
		}
	}

	/// ## 读取文件内容
	pub fn read_to_string(&mut self) -> io::Result<String> {
		fs::read_to_string(self.path())
	}
}

pub enum Char {
	Char(char),
	EndLine,
	EndFile,
	ErrFile,
}

pub struct Cursor {
	num_line: usize,
	num_column: usize,
	// 当前字节偏移量
	byte_offset: usize,
	file_wrapper: FileWrapper,
	// 改为字节数组
	line: Vec<u8>,
	// 改为u8
	point: u8,
}

impl Cursor {
	pub fn new(file: FileWrapper) -> Self {
		Cursor {
			file_wrapper: file,
			num_line: 0,
			num_column: 0,
			byte_offset: 0,
			line: Vec::new(),
			point: 0,
		}
	}

	pub fn peek(&self) -> Char {
		if self.file_wrapper.works() {
			if self.byte_offset < self.line.len() {
				// 尝试解码UTF-8字符
				let s = std::str::from_utf8(&self.line[self.byte_offset..])
					.unwrap_or("");
				if let Some(c) = s.chars().next() {
					Char::Char(c)
				} else {
					Char::EndLine
				}
			} else {
				Char::EndLine
			}
		} else {
			Char::ErrFile
		}
	}

	pub fn next(&mut self) -> Char {
		if self.file_wrapper.works() {
			if self.byte_offset < self.line.len() {
				// 获取当前字符及其字节长度
				let s = std::str::from_utf8(&self.line[self.byte_offset..]).unwrap_or("");
				let option = s.chars().next();
				if let Some(c) = option {
					let char_len = c.len_utf8();
					self.point = self.line[self.byte_offset];
					self.byte_offset += char_len;
					self.num_column += 1;
					return Char::Char(c);
				}
				return self.next_line();
			} else {
				return self.next_line();
			}
		} else {
			Char::ErrFile
		}
	}

	fn next_line(&mut self) -> Char {
		if self.file_wrapper.works() {
			// 读取下一行
			match self.file_wrapper.next() {
				Some(Ok(line)) => {
					self.line = line.into_bytes();
					self.byte_offset = 0;
					self.num_line += 1;
					self.num_column = 0;
					Char::EndLine
				}
				Some(Err(_)) => Char::ErrFile,
				None => Char::EndFile,
			}
		} else {
			Char::ErrFile
		}
	}

	pub fn get_file(&self) -> &FileWrapper {
		&self.file_wrapper
	}

	/// ## 获取当前位置
	/// return x, y, char
	pub fn get_position(&self) -> (usize, usize, u8) {
		(self.num_line, self.num_column, self.point)
	}

	pub fn get_pointing(&self) -> CursorPointing {
		CursorPointing {
			num_line: self.num_line,
			num_column: self.num_column,
			line: self.line.clone(),
		}
	}
}

#[derive(Debug)]
pub struct CursorPointing {
	/// 行号
	pub num_line: usize,
	/// 列号
	pub num_column: usize,
	/// 当前行
	pub line: Vec<u8>,
}

/// # 日志
pub struct Log {
	_type: ColoredString,
	_msg: String,
}

pub enum LogType {
	Err,
	Info(ColoredString),
	Warn,
}

impl Log {
	/// ## 创建新的日志输出
	/// `_type` 错误的类型： e:错误 i:表示信息 w:表示警告 <br />
	/// `_msg` 消息正文 <br />
	/// `stop_key` 是退出程序的代码（如果错误是致命的） <br />如果为`0`则不退出程序，继续运行
	pub fn new(_type: LogType, _msg: &str) -> Self {
		Self {
			_type: match _type {
				LogType::Info(s) => s,
				LogType::Warn => "警告".yellow(),
				LogType::Err => "错误".red(),
			},
			_msg: _msg.to_string(),
		}
	}

	pub fn creat(err: ErrorKind, _msg: &str) -> Self {
		Self {
			_type: "错误".red(),
			_msg: match err {
				ErrorKind::NotFound => format!("{} {}", _msg, "不存在"),
				ErrorKind::PermissionDenied => format!("{} {}", _msg, "权限不足"),
				ErrorKind::AlreadyExists => format!("{} {}", _msg, "已存在"),
				ErrorKind::InvalidData => format!("{} {}", _msg, "数据错误"),
				ErrorKind::BrokenPipe => format!("{} {}", _msg, "管道已损坏"),
				ErrorKind::NotConnected => format!("{} {}", _msg, "未连接"),
				ErrorKind::ConnectionRefused => format!("{} {}", _msg, "连接被拒绝"),
				ErrorKind::ConnectionReset => format!("{} {}", _msg, "连接被重置"),
				ErrorKind::ConnectionAborted => format!("{} {}", _msg, "连接被中止"),
				ErrorKind::TimedOut => format!("{} {}", _msg, "连接超时"),
				ErrorKind::Interrupted => format!("{} {}", _msg, "操作被中断"),
				ErrorKind::WriteZero => format!("{} {}", _msg, "写入零字节"),
				ErrorKind::UnexpectedEof => format!("{} {}", _msg, "未预期的文件结束"),
				ErrorKind::InvalidInput => format!("{} {}", _msg, "无效的输入"),
				ErrorKind::NotADirectory => format!("{} {}", _msg, "不是目录"),
				ErrorKind::IsADirectory => format!("{} {}", _msg, "是目录"),
				ErrorKind::ReadOnlyFilesystem => format!("{} {}", _msg, "只读文件系统"),
				ErrorKind::FileTooLarge => format!("{} {}", _msg, "文件太大"),
				ErrorKind::TooManyLinks => format!("{} {}", _msg, "链接过多"),
				ErrorKind::Other => format!("{} {}", _msg, "其他错误"),
				_ => format!("{} {}", _msg, "未知错误"),
			},
		}
	}

	/// ## 打印日志
	pub fn print(self) {
		println!("{}: {}", self._type, self._msg)
	}

	/// ## 打印错误
	pub fn throw(self, exit_code: i32) -> ! {
		eprint!("{}[{}] {}", self._type, exit_code.to_string().yellow(), self._msg);
		exit(exit_code);
	}
}