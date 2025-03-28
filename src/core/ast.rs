use std::iter::Map;
use crate::parser::qls::Value;
///  语句
#[derive(Debug)]
pub enum Statement {
	/// 语句
	Statement(Expression),
	/// 块
	Block(Vec<Statement>),
	/// 函数
	Function(Function),
	/// 方法
	Method(Method),
	/// 条件
	If(Box<If>),
	/// 匹配
	Match(Box<Match>),
	/// 循环
	Loop(Loop),
	/// 循环
	While(Box<While>),
	/// 循环
	For(Box<For>),
}
/// 表达式
pub enum Expression {
	/// 导入
	Import(Vec<String>),
	/// 比较
	Compare(Compare),
	/// 移动
	Move(Object, Box<Expression>),
	/// 调用
	CallFn(),
	CallOp(),
	/// 内存申请
	Memory(Box<Memory>),
	/// 常量值
	Value(Value),
	/// 计算
	Compute(Compute),
}
/// 函数
pub struct Function {
	/// 函数名
	name: String,
	/// 参数
	params: Vec<String>,
	/// 函数体
	body: Vec<Statement>,
}
/// 方法
pub struct Method {
	/// 方法名
	name: String,
	/// 参数
	params: Vec<String>,
	/// 方法体
	body: Vec<Statement>,
}
/// 语句 循环
pub struct Loop {
	/// 循环体
	body: Vec<Statement>,
}
/// 语句 条件
pub struct If {
	/// 条件
	condition: Box<Expression>,
	/// 条件为真时
	body_true: Box<Statement>,
	/// 条件为假时
	body_false: Option<Box<Statement>>,
}
/// 语句 匹配
pub struct Match {
	/// 匹配对象
	object: Box<Expression>,
	/// 匹配体
	body: Map<Expression, Box<Statement>>,
}
/// 语句 while 循环
pub struct While {
	/// 条件
	condition: Box<Expression>,
	/// 循环体
	body: Statement,
}
/// 语句 for-each 循环
pub struct For {
	/// 循环对象
	object: Box<Expression>,
	/// 循环变量
	variable: Object,
	/// 循环体
	body: Statement,
}
/// 对象
pub enum Object {
	/// 对象
	Object {
		name: String,
	}
}
/// 计算表达式
pub enum Compute {
	/// `e + e`
	Plus(Box<Expression>, Box<Expression>),
	/// `o += e`
	PlusEqual(Object, Box<Expression>),
	/// `e - e`
	Minus(Box<Expression>, Box<Expression>),
	/// `o -= e`
	MinusEqual(Object, Box<Expression>),
	/// `e * e`
	By(Box<Expression>, Box<Expression>),
	/// `o *= e`
	ByEqual(Object, Box<Expression>),
	/// `e / e`
	Dividing(Box<Expression>, Box<Expression>),
	/// `o /= e`
	DividingEqual(Object, Box<Expression>),
	/// `e % e`
	Impressions(Box<Expression>, Box<Expression>),
	/// `o %= e`
	ImpressionsEqual(Object, Box<Expression>),
	/// `e && e`
	And(Box<Expression>, Box<Expression>),
	/// `e || e`
	Or(Box<Expression>, Box<Expression>),
	/// `!e`
	Not(Box<Expression>),
}
// /// & 引用
// Reference(Object),
// /// @ 结构引用
// AtStruct,
// /// $ 要求结构应用
// GetStruct,
/// 比较表达式
pub enum Compare {
	/// `e == e`
	Is,
	/// `e != e`
	NotIs,
	/// `e > e`
	Greater,
	/// `e < e`
	Less,
	/// `e >= e`
	GreaterEqual,
	/// `e <= e`
	LessEqual,
	/// `e?`
	/// 判断有效
	Exist,
}
/// 内存申请表达式
pub enum Memory {
	/// 申请
	New(Box<Expression>),
	/// 释放
	Drop(Box<Expression>)
}