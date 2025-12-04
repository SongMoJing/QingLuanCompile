use crate::parser::qln::token::*;

#[derive(Debug, Clone, PartialEq)]
pub struct AST {
	ast_type: ASTType,
	body: Node,
}

impl AST {
	pub fn new(ast_type: ASTType) -> Self {
		Self {
			ast_type,
			body: Node::File { body: vec![] },
		}
	}

	pub fn get_root(&self) -> &Node {
		&self.body
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTType {
	File,
}

/// AST 节点定义
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
	File {
		// 模块内容
		body: Vec<Node>,
	},
	Import {
		// 模块名
		name: TokenKind,
		// 模块路径
		path: Vec<String>,
	},
	Module {
		// 模块名
		name: String,
		// 模块属性
		attrs: Vec<String>,
		// 模块内容
		body: Vec<Node>,
	},
	Class {
		// 类名
		name: String,
		// 类属性
		attrs: Vec<String>,
		// 类继承
		extends: Option<String>,
		// 类实现
		implements: Option<Vec<String>>,
		// 类内容
		body: Vec<Node>,
	},
	Attr {
		// 属性名称
		name: String,
		// 属性属性
		attr: Vec<String>,
		// 属性类型
		kind: String,
		// 属性值
		value: Option<Box<Node>>,
	},
	Function {
		// 函数名称
		name: String,
		// 函数类型
		kind: FunctionKind,
		// 函数属性
		attrs: Vec<String>,
		// 函数参数
		params: Option<Vec<Param>>,
		// 函数内容
		body: Vec<Node>,
		// 函数返回类型
		return_type: Option<Expression>,
	},
	ForEach {
		// 循环变量
		variable: String,
		// 循环变量类型
		kind: String,
		// 循环集合
		collection: Box<Node>,
		// 循环体内容
		body: Vec<Node>,
	},
	ForInRange {
		// 初始化循环
		initializer: Option<Expression>,
		// 条件
		condition: Option<Expression>,
		// 迭代器
		increment: Option<Expression>,
	},
	// 表达式
	Expression(Expression),
	// 块级语句
	Block(Vec<Node>),
	If {
		// 条件
		condition: Expression,
		// 满足条件时执行
		then_body: Vec<Node>,
		// 不满足条件时执行
		else_body: Option<Vec<Node>>,
	},
	// 泛型类型
	GenericType {
		// 基础类型
		base: String,
		// 泛型参数
		params: Vec<Node>,
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
	// 二元运算
	Binary {
		left: Box<Node>,
		operator: Token,
		right: Box<Node>,
	},
	// 一元运算
	Unary {
		operator: Token,
		value: Box<Node>,
	},
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionKind {
	// 构造函数
	Constructor,
	// 实例函数
	Instance,
	// 静态函数
	Static,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
	// 参数类型
	kind: String,
	// 参数名称
	name: String,
}

/// 解析错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
	UnexpectedToken(Token),
	UnexpectedEof,
	InvalidSyntax,
}

/// 解析器
pub struct Parser {
	tokens: Vec<Token>,
	current: usize,
	errors: Vec<(ParseError, usize)>, // 错误和对应的token索引
}