use colored::{ColoredString, Colorize};
use std::fs;
use std::io;
use std::io::BufRead;
use std::process::exit;

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
			file: {
				Some(io::BufReader::new({
					match file {
						Ok(file) => file,
						Err(_) => return Self {
							works: false,
							path,
							file: None,
						},
					}
				}))
			}
		}
	}

	/// ## 对象正常
	pub fn works(&mut self) -> bool {
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
}

pub struct Cursor {
	num_line: usize,
	num_column: usize,
	content: FileWrapper,
	line: String,
	point: char,
}

impl Cursor {
	pub fn new(file: FileWrapper) -> Self {
		Cursor {
			content: file,
			num_line: 0,
			num_column: 0,
			line: String::new(),
			point: '\u{0}',
		}
	}

	/// ## 读取下一个字符
	/// return char
	pub fn next(&mut self) -> Char {
		if self.num_column >= self.line.len() {
			return if let Some(i) = self.content.next() {
				if let Ok(i) = i {
					self.num_line += 1;
					self.num_column = 0;
					self.line = i;
					self.point = '\u{0}';
					Char::EndLine
				} else {
					Char::EndFile
				}
			} else {
				Char::EndFile
			}
		} else {
			self.point = self.line.chars().nth(self.num_column - 1).unwrap();
			self.num_column += 1;
		}
		Char::Char(self.point)
	}

	/// ## 获取当前位置
	/// return x, y, char
	pub fn get_position(&self) -> (usize, usize, char) {
		(self.num_line, self.num_column, self.point)
	}
}

/// # 日志
pub struct Log {
	_type: ColoredString,
	_msg: String
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

	/// ## 打印日志
	pub fn print(&self) {
		println!("{}: {}", self._type, self._msg)
	}

	/// ## 打印错误
	pub fn throw(&self, exit_code: i32) -> ! {
		eprint!("{}[{}] {}", self._type, exit_code.to_string().yellow(), self._msg);
		exit(exit_code);
	}
}