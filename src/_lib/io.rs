use std::fs;
use std::io;
use std::io::{BufRead, Error, ErrorKind};
use std::process::exit;

use colored::{ColoredString, Colorize};
use rust_i18n::t;

/// # 位置信息
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
	pub start_line: usize,
	pub start_col: usize,
	pub end_line: usize,
	pub end_col: usize,
}

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
	file_wrapper: FileWrapper,
	line: Vec<char>,
	point: char,
}

impl Cursor {
	pub fn new(mut file: FileWrapper) -> Option<Self> {
		if !file.works() {
			return None;
		}
		let option = file.next();
		if let Some(Ok(line)) = option {
			return Some(Cursor {
				file_wrapper: file,
				num_line: 1,
				num_column: 0,
				line: line.chars().collect::<Vec<_>>(),
				point: 0 as char,
			});
		}
		None
	}

	pub fn next(&mut self) -> Char {
		if !self.file_wrapper.works() {
			return Char::ErrFile;
		}
		if self.num_column < self.line.len() {
			self.point = self.line[self.num_column];
			self.num_column += 1;
			return Char::Char(self.point);
		}
		if self.num_column == self.line.len() {
			self.point = 0 as char;
			self.num_column += 1;
			return Char::EndLine;
		}
		let option = self.file_wrapper.next();
		if let Some(Ok(line)) = option {
			self.line = line.chars().collect::<Vec<_>>();
			self.num_line += 1;
			self.num_column = 0;
			if self.line.len() == 0 {
				return self.next();
			}
			self.point = self.line[self.num_column];
			self.num_column += 1;
			return Char::Char(self.point);
		}
		return Char::EndFile;
	}

	pub fn get_file(&self) -> &FileWrapper {
		&self.file_wrapper
	}

	/// ## 获取当前位置
	/// return x, y, char
	pub fn get_position(&self) -> (usize, usize, char) {
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
	pub line: Vec<char>,
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
	pub fn new(_type: LogType, _msg: &str) -> Self {
		Self {
			_type: match _type {
				LogType::Info(s) => s.white(),
				LogType::Warn => t!("log.Warn.TYPE").yellow(),
				LogType::Err => t!("log.Err.TYPE").red(),
			},
			_msg: _msg.to_string(),
		}
	}

	pub fn creat(err: ErrorKind, _msg: &str) -> Self {
		Self {
			_type: t!("log.Err.TYPE").red(),
			_msg: match err {
				ErrorKind::NotFound => format!("{} {}", _msg, t!("log.Err.NotFound")),
				ErrorKind::PermissionDenied => format!("{} {}", _msg, t!("log.Err.PermissionDenied")),
				ErrorKind::AlreadyExists => format!("{} {}", _msg, t!("log.Err.AlreadyExists")),
				ErrorKind::InvalidData => format!("{} {}", _msg, t!("log.Err.InvalidData")),
				ErrorKind::BrokenPipe => format!("{} {}", _msg, t!("log.Err.BrokenPipe")),
				ErrorKind::NotConnected => format!("{} {}", _msg, t!("log.Err.NotConnected")),
				ErrorKind::ConnectionRefused => format!("{} {}", _msg, t!("log.Err.ConnectionRefused")),
				ErrorKind::ConnectionReset => format!("{} {}", _msg, t!("log.Err.ConnectionReset")),
				ErrorKind::ConnectionAborted => format!("{} {}", _msg, t!("log.Err.ConnectionAborted")),
				ErrorKind::TimedOut => format!("{} {}", _msg, t!("log.Err.TimedOut")),
				ErrorKind::Interrupted => format!("{} {}", _msg, t!("log.Err.Interrupted")),
				ErrorKind::WriteZero => format!("{} {}", _msg, t!("log.Err.WriteZero")),
				ErrorKind::UnexpectedEof => format!("{} {}", _msg, t!("log.Err.UnexpectedEof")),
				ErrorKind::InvalidInput => format!("{} {}", _msg, t!("log.Err.InvalidInput")),
				ErrorKind::NotADirectory => format!("{} {}", _msg, t!("log.Err.NotADirectory")),
				ErrorKind::IsADirectory => format!("{} {}", _msg, t!("log.Err.IsADirectory")),
				ErrorKind::ReadOnlyFilesystem => format!("{} {}", _msg, t!("log.Err.ReadOnlyFilesystem")),
				ErrorKind::FileTooLarge => format!("{} {}", _msg, t!("log.Err.FileTooLarge")),
				ErrorKind::TooManyLinks => format!("{} {}", _msg, t!("log.Err.TooManyLinks")),
				ErrorKind::Other => format!("{} {}", _msg, t!("log.Err.Other")),
				_ => format!("{} {}", _msg, t!("log.Err.unknown")),
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
	
	pub fn parse_error(err: Error) -> Self {
		Self::creat(err.kind(), err.to_string().as_str())
	}
}