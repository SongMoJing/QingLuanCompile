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

#[derive(Debug, Clone, PartialEq)]
/// 结构
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
	Val
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
	/// `+` `[expression] + [expression]` 加
	Plus,
	/// `-` `[expression] - [expression]` 减
	Minus,
	/// `*` `[expression] * [expression]` 乘
	Star,
	/// `/` `[expression] / [expression]` 除
	Slash,
	/// `%` `[expression] % [expression]` 取余
	Percent,
	/// `~` `~ [expression]` 逻辑非
	Tilde,
	/// `&` `[expression] & [expression]` 逻辑与
	Ampersand,
	/// `|` `[expression] | [expression]` 逻辑或
	Pipe,
	/// `^` `[expression] ^ [expression]` 逻辑异或
	Caret,
	/// `!` `! [expression]` 取反
	Exclamation,
	/// `?` `[expression]?` 错误传播
	Question,
	/// `=` `[expression] = [expression]` 赋值
	Assign,
	/// `==` `[expression] == [expression]` 等于
	Eq,
	/// `->` 函数返回 | 闭包
	Arrow,
	/// `<` `[expression] < [expression]` 小于
	Lt,
	/// `>` `[expression] > [expression]` 大于
	Gt,
	/// `!=` `[expression] != [expression]` 不等于
	Ne,
	/// `<=` `[expression] <= [expression]` 小于等于
	Le,
	/// `>=` `[expression] >= [expression]` 大于等于
	Ge,
	/// `&&` `[expression] && [expression]` 逻辑与
	And,
	/// `||` `[expression] || [expression]` 逻辑或
	Or,
	/// `+=` `[expression] += [expression]` 加赋值
	PlusAssign,
	/// `-=` `[expression] -= [expression]` 减赋值
	MinusAssign,
	/// `*=` `[expression] *= [expression]` 乘赋值
	StarAssign,
	/// `/=` `[expression] /= [expression]` 除赋值
	SlashAssign,
	/// `%=` `[expression] %= [expression]` 取余赋值
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
}