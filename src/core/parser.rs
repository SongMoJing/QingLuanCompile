use std::io::BufRead;
use crate::core::{
	lexer::Token,
	ast::*
};

pub struct Parser {
	tokens: Vec<Token>,
	current: usize,
}

#[derive(Debug)]
pub enum ParseError {
	UnexpectedToken(Token),
	UnexpectedEOF,
}

impl Parser {
	// pub fn new(tokens: Vec<Token>) -> Self {
	// 	Self { tokens, current: 0 }
	// }
	//
	// /// 主解析入口
	// pub fn parse(&mut self) -> Result<Vec<Statement>, ParseError> {
	// 	let mut stmts = Vec::new();
	// 	while !self.is_at_end() {
	// 		stmts.push(self.parse_statement()?);
	// 	}
	// 	Ok(stmts)
	// }
	//
	// /// 解析语句
	// fn parse_statement(&mut self) -> Result<Statement, ParseError> {
	// 	if self.match_token(Token::Ident(String::new())) {
	// 		self.parse_assignment()
	// 	} else {
	// 		self.parse_expression_statement()
	// 	}
	// }
	//
	// /// 解析赋值语句：a = 1 + 2;
	// fn parse_assignment(&mut self) -> Result<Statement, ParseError> {
	// 	let name = if let Token::Ident(s) = self.previous() {
	// 		s.clone()
	// 	} else {
	// 		unreachable!()
	// 	};
	// 	self.consume(Token::Eq)?;
	// 	let expr = self.parse_expression()?;
	// 	self.consume(Token::Semicolon)?;
	//
	// 	Ok(Statement::Assignment { name, expr })
	// }
	//
	// /// 解析表达式（运算符优先级处理）
	// fn parse_expression(&mut self) -> Result<Expr, ParseError> {
	// 	self.parse_additive()
	// }
	//
	// fn parse_additive(&mut self) -> Result<Expr, ParseError> {
	// 	let mut expr = self.parse_multiplicative()?;
	//
	// 	while self.match_tokens(&[Token::Plus, Token::Minus]) {
	// 		let op = self.previous().clone();
	// 		let right = self.parse_additive()?;
	// 		expr = Expr::Binary {
	// 			left: Box::new(expr),
	// 			op: match op {
	// 				Token::Plus => BinOp::Add,
	// 				Token::Minus => BinOp::Sub,
	// 				_ => unreachable!(),
	// 			},
	// 			right: Box::new(right),
	// 		};
	// 	}
	//
	// 	Ok(expr)
	// }
	//
	// // 更多层级解析方法...
}