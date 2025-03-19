use std::io;
use std::io::{Read, Write};

/// ## 接受用户输入内容
/// 返回输入内容
/// ## Example
/// ```rust
/// let mut input = String::new();
/// input("请输入内容：", &mut input);
/// // input: hello
/// assert_eq!(input, "hello");
/// ```
pub(crate) fn input(tip: &str, res: &mut String) {
	print!("{}", tip);
	io::stdout().flush().expect("Err: Refresh failed");
	io::stdin()
		.read_line(res)
		.expect("Err: Read input failed");
}

/// ## 接受用户确认
/// 返回确认结果，非'y'即为否定
/// ## Example
/// ```rust
/// let mut res = false;
/// identify("是否继续？(y/n): ", &mut res);
/// // input: y
/// assert_eq!(res, true);
/// ```
pub(crate) fn identify(tip: &str, res: &mut bool) {
	let mut _input = String::new();
	input(tip, &mut _input);
	*res = _input.trim() == "y";
}

/// ## 接受用户输入字符
fn get_char() -> char {
	let mut buf = vec![0; 1];
	io::stdin().lock().read_exact(&mut buf).unwrap();
	buf[0] as _
}