#[derive(Debug, Clone, PartialEq)]
pub struct Token {
	pub kind: TokenKind,
	pub span: Span, // 包含始、终位置
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
	pub start_line: usize,
	pub start_col: usize,
	pub end_line: usize,
	pub end_col: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
	// 关键字
	Key(Key),
	// 标识符
	Ident(String),
	// 字面量
	String(Vec<CharacterString>),
	Char(char),
	Bool(bool),
	NumInt(i64),
	NumFloat(f64),
	// 符号
	// (
	LParen,
	// )
	RParen,
	// {
	LBrace,
	// }
	RBrace,
	// [
	LBracket,
	// ]
	RBracket,
	// ;
	Semicolon,
	// :
	Colon,
	// ,
	Comma,
	// .
	Dot,
	// +
	Plus,
	// -
	Minus,
	// *
	Star,
	// /
	Slash,
	// %
	Percent,
	// &
	Ampersand,
	// |
	Pipe,
	// ^
	Caret,
	// !
	Exclamation,
	// ?
	Question,
	// =
	Assign,
	// ==
	Eq,
	// <
	Lt,
	// >
	Gt,
	// ->
	Arrow,
	// !=
	Ne,
	// <=
	Le,
	// >=
	Ge,
	// &&
	And,
	// ||
	Or,
	// +=
	PlusAssign,
	// -=
	MinusAssign,
	// *=
	StarAssign,
	// /=
	SlashAssign,
	// %=
	ModAssign,
	// 文件结束
	EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Key {
	// 结构
	StructKey(KeyStruct),
	// 逻辑控制
	LogicControl(KeyLogicControl),
	// 顺序控制
	SequenceControl(KeySequenceControl),
	// 对象分配
	ObjectAllocation(KeyObjectAllocation),
	// 内存分配
	MemoryAllocation(KeyMemoryAllocation),
	// 对象访问
	ObjectAccess(KeyObjectAccess),
}

/// 结构
#[derive(Debug, Clone, PartialEq)]
pub enum KeyStruct {
	/// `[any] > import`
	Import,
	/// `[file] > mod`
	Mod,
	/// `[mod] > class`
	Class,
	/// `[class] > attr`
	Attr,
	/// `[class] > static`
	Static,
	/// `[class] > init`
	Init,
	/// `[file] > fn` `[class] > fn`
	Fn,
}

/// 逻辑控制
#[derive(Debug, Clone, PartialEq)]
pub enum KeyLogicControl {
	/// `if (expression) [statement]`
	If,
	/// `else (expression)`
	Else,
	/// `for (expression) [statement]`
	For,
	/// `while (expression) [statement]`
	While,
	/// `loop [statement]`
	Loop,
	/// `match (expression) [statement]`
	Match,
}

/// 顺序控制
#[derive(Debug, Clone, PartialEq)]
pub enum KeySequenceControl {
	/// `continue;`
	Continue,
	/// `break;`
	Break,
	/// `return [any]?;`
	Return,
}

/// 对象分配
#[derive(Debug, Clone, PartialEq)]
pub enum KeyObjectAllocation {
	/// `let a`
	Let,
	/// `let mut a`
	Var,
	/// `const a`
	Const,
}

/// 内存分配
#[derive(Debug, Clone, PartialEq)]
pub enum KeyMemoryAllocation {
	/// `new [any]`
	New,
}

/// 对象访问
#[derive(Debug, Clone, PartialEq)]
pub enum KeyObjectAccess {
	/// `self`
	ToSelf,
	/// `super`
	ToParent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CharacterString {
	String(String),
	Implant(Vec<Token>),
}