use std::fmt::{Display, Formatter};
use crate::_lib::io::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
	pub kind: TokenKind,
	pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
	/// 关键字
	Key(Key),
	/// 标识符
	Ident(String),
	/// 字面量
	Value(Value),
	/// 运算符
	Op(Op),
	/// 符号
	Symbol(Symbol),
	/// 作用域结束
	EOS,
	/// 文件结束
	EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Key {
	/// 结构
	StructKey(KeyStruct),
	/// 逻辑控制
	LogicControl(KeyLogicControl),
	/// 顺序控制
	SequenceControl(KeySequenceControl),
	/// 对象分配
	ObjectAllocation(KeyObjectAllocation),
	/// 内存分配
	MemoryAllocation(KeyMemoryAllocation),
	/// 对象访问
	ObjectAccess(KeyObjectAccess),
}

#[derive(Debug, Clone, PartialEq)]
/// 结构
pub enum KeyStruct {
	/// `[any] > import`
	Import,
	/// `[file] > mod`
	Mod,
	/// `[mod|class] > class`
	Class,
	/// `[class] > attr`
	Attr,
	/// `[class] > static`
	Static,
	/// `[class] > init`
	Init,
	/// `[class] > fn`
	Fn,
}

#[derive(Debug, Clone, PartialEq)]
/// 逻辑控制
pub enum KeyLogicControl {
	/// `if (expression) [statement]`
	If,
	/// `else (expression)`
	Else,
	/// `for (...';' expression) [statement]`
	For,
	/// `while (expression) [statement]`
	While,
	/// `loop [statement]`
	Loop,
	/// `match (...',' expression) [statement]`
	Match,
}

#[derive(Debug, Clone, PartialEq)]
/// 顺序控制
pub enum KeySequenceControl {
	/// `continue;`
	Continue,
	/// `break;`
	Break,
	/// `return [expression]?;`
	Return,
}

#[derive(Debug, Clone, PartialEq)]
/// 引用分配
pub enum KeyObjectAllocation {
	/// `var a`
	Var,
	/// `val a`
	Val,
}

#[derive(Debug, Clone, PartialEq)]
/// 内存分配
pub enum KeyMemoryAllocation {
	/// `new [any]`
	New,
}

#[derive(Debug, Clone, PartialEq)]
/// 对象访问
pub enum KeyObjectAccess {
	/// `self`
	ToSelf,
	/// `super`
	ToParent,
}

#[derive(Debug, Clone, PartialEq)]
/// 常量值
pub enum Value {
	/// 字符串
	String(Vec<CharacterString>),
	/// 字符
	Char(char),
	/// 布尔
	Bool(bool),
	/// 数字
	NumInt(i64),
	/// 浮点数
	NumFloat(f64),
}

#[derive(Debug, Clone, PartialEq)]
/// 字符串
pub enum CharacterString {
	/// 字符串
	String(String),
	/// 嵌入代码
	Implant(Vec<Token>),
}

#[derive(Debug, Clone, PartialEq)]
/// 运算符
pub enum Op {
	/// `+`\
	/// `[expression] + [expression]` 加
	Plus,
	/// `-`\
	/// `[expression] - [expression]` 减
	Minus,
	/// `*`\
	/// `[expression] * [expression]` 乘
	Star,
	/// `/`\
	/// `[expression] / [expression]` 除
	Slash,
	/// `%`\
	/// `[expression] % [expression]` 取余
	Percent,
	/// `~`\
	/// `~ [expression]` 逻辑非
	Tilde,
	/// `&`\
	/// `[expression] & [expression]` 逻辑与
	Ampersand,
	/// `|`\
	/// `[expression] | [expression]` 逻辑或
	Pipe,
	/// `^`\
	/// `[expression] ^ [expression]` 逻辑异或
	Caret,
	/// `!`\
	/// `! [expression]` 布尔值取反
	Exclamation,
	/// `?`\
	/// `[expression]?` 错误传播
	Question,
	/// `=`\
	/// `[expression] = [expression]` 赋值
	Assign,
	/// `==`\
	/// `[expression] == [expression]` 等于
	Eq,
	/// `->`\
	/// `[fn] -> [return]` 函数返回\
	/// `[args] -> [statement]` 闭包
	Arrow,
	/// `<`\
	/// `[expression] < [expression]` 小于\
	/// `[struct] <[modifier]>` 结构修饰符
	Lt,
	/// `>`\
	/// `[expression] > [expression]` 大于\
	/// `[struct] <[modifier]>` 结构修饰符
	Gt,
	/// `!=`\
	/// `[expression] != [expression]` 不等于
	Ne,
	/// `<=`\
	/// `[expression] <= [expression]` 小于等于
	Le,
	/// `>=`\
	/// `[expression] >= [expression]` 大于等于
	Ge,
	/// `&&`\
	/// `[expression] && [expression]` 逻辑与
	And,
	/// `||`\
	/// `[expression] || [expression]` 逻辑或
	Or,
	/// `+=`\
	/// `[expression] += [expression]` 加赋值
	PlusAssign,
	/// `-=`\
	/// `[expression] -= [expression]` 减赋值
	MinusAssign,
	/// `*=`\
	/// `[expression] *= [expression]` 乘赋值
	StarAssign,
	/// `/=`\
	/// `[expression] /= [expression]` 除赋值
	SlashAssign,
	/// `%=`\
	/// `[expression] %= [expression]` 取余赋值
	ModAssign,
}

#[derive(Debug, Clone, PartialEq)]
/// 符号
pub enum Symbol {
	/// `(`
	LParen,
	/// `)`
	RParen,
	/// `{`
	LBrace,
	/// `}`
	RBrace,
	/// `[`
	LBracket,
	/// `]`
	RBracket,
	/// `;`
	Semicolon,
	/// `:`
	Colon,
	/// `,`
	Comma,
	/// `.`
	Dot,
	/// `@`
	At,
}

impl Display for Token {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.kind)
	}
}

impl Display for TokenKind {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			TokenKind::Key(key) => {
				match key {
					Key::LogicControl(key) => write!(f, "{}", key),
					Key::SequenceControl(key) => write!(f, "{}", key),
					Key::ObjectAllocation(key) => write!(f, "{}", key),
					Key::MemoryAllocation(key) => write!(f, "{}", key),
					Key::ObjectAccess(key) => write!(f, "{}", key),
					Key::StructKey(key) => write!(f, "{}", key),
				}
			}
			TokenKind::Ident(ident) => {
				write!(f, "'{}'", ident)
			}
			TokenKind::Value(value) => {
				match value {
					Value::String(string) => get_vec(string, f),
					Value::Char(char) => write!(f, "{}", char),
					Value::Bool(bool) => write!(f, "{}", bool),
					Value::NumInt(num) => write!(f, "{}", num),
					Value::NumFloat(num) => write!(f, "{}", num),
				}
			}
			TokenKind::Op(op) => {
				write!(f, "{}", op)
			}
			TokenKind::Symbol(symbol) => {
				write!(f, "{}", symbol)
			}
			TokenKind::EOS => write!(f, "EOS"),
			TokenKind::EOF => write!(f, "EOF"),
		}
	}
}

impl Display for KeyStruct {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			KeyStruct::Import => write!(f, "import"),
			KeyStruct::Mod => write!(f, "mod"),
			KeyStruct::Class => write!(f, "class"),
			KeyStruct::Attr => write!(f, "attr"),
			KeyStruct::Static => write!(f, "static"),
			KeyStruct::Init => write!(f, "init"),
			KeyStruct::Fn => write!(f, "fn"),
		}
	}
}

impl Display for KeyLogicControl {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			KeyLogicControl::If => write!(f, "if"),
			KeyLogicControl::Else => write!(f, "else"),
			KeyLogicControl::For => write!(f, "for"),
			KeyLogicControl::While => write!(f, "while"),
			KeyLogicControl::Loop => write!(f, "loop"),
			KeyLogicControl::Match => write!(f, "match"),
		}
	}
}

impl Display for KeySequenceControl {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			KeySequenceControl::Continue => write!(f, "continue"),
			KeySequenceControl::Break => write!(f, "break"),
			KeySequenceControl::Return => write!(f, "return"),
		}
	}
}

impl Display for KeyObjectAllocation {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			KeyObjectAllocation::Var => write!(f, "var"),
			KeyObjectAllocation::Val => write!(f, "val"),
		}
	}
}

impl Display for KeyMemoryAllocation {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			KeyMemoryAllocation::New => write!(f, "new"),
		}
	}
}

impl Display for KeyObjectAccess {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			KeyObjectAccess::ToSelf => write!(f, "self"),
			KeyObjectAccess::ToParent => write!(f, "super"),
		}
	}
}

impl Display for Op {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Op::Plus => write!(f, "+"),
			Op::Minus => write!(f, "-"),
			Op::Star => write!(f, "*"),
			Op::Slash => write!(f, "/"),
			Op::Percent => write!(f, "%"),
			Op::Tilde => write!(f, "~"),
			Op::Ampersand => write!(f, "&"),
			Op::Pipe => write!(f, "|"),
			Op::Caret => write!(f, "^"),
			Op::Exclamation => write!(f, "!"),
			Op::Question => write!(f, "?"),
			Op::Assign => write!(f, "="),
			Op::Eq => write!(f, "=="),
			Op::Arrow => write!(f, "->"),
			Op::Lt => write!(f, "<"),
			Op::Gt => write!(f, ">"),
			Op::Ne => write!(f, "!="),
			Op::Le => write!(f, "<="),
			Op::Ge => write!(f, ">="),
			Op::And => write!(f, "&&"),
			Op::Or => write!(f, "||"),
			Op::PlusAssign => write!(f, "+="),
			Op::MinusAssign => write!(f, "-="),
			Op::StarAssign => write!(f, "*="),
			Op::SlashAssign => write!(f, "/="),
			Op::ModAssign => write!(f, "%="),
		}
	}
}

impl Display for Symbol {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Symbol::LParen => write!(f, "("),
			Symbol::RParen => write!(f, ")"),
			Symbol::LBrace => write!(f, "{{"),
			Symbol::RBrace => write!(f, "}}"),
			Symbol::LBracket => write!(f, "["),
			Symbol::RBracket => write!(f, "]"),
			Symbol::Semicolon => write!(f, ";"),
			Symbol::Colon => write!(f, ":"),
			Symbol::Comma => write!(f, ","),
			Symbol::Dot => write!(f, "."),
			Symbol::At => write!(f, "@"),
		}
	}
}

impl Display for Value {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Value::String(string) => get_vec(string, f),
			Value::Char(char) => write!(f, "{}", char),
			Value::Bool(bool) => write!(f, "{}", bool),
			Value::NumInt(num) => write!(f, "{}", num),
			Value::NumFloat(num) => write!(f, "{}", num),
		}
	}
}

impl Display for CharacterString {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			CharacterString::String(char) => write!(f, "{}", char),
			CharacterString::Implant(vec) => get_vec(vec, f),
		}
	}
}

fn get_vec<T: Display>(vec: &Vec<T>, f: &mut Formatter<'_>) -> std::fmt::Result {
	write!(f, "[")?;
	for (i, item) in vec.iter().enumerate() {
		write!(f, "{}", item)?;
		if i != vec.len() - 1 {
			write!(f, ", ")?;
		}
	}
	write!(f, "]")
}