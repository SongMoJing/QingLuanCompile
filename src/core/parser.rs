use crate::core::ast::*;
use crate::core::lexer::Token;

pub struct Parser {
	current: usize,
}

impl Parser {
	pub fn new() -> Self {
		Self {current: 0 }
	}

	pub fn parse(&mut self, tokens: Vec<Token>) -> Result<Statement, ParserResult> {
		let mut iter = tokens.iter();
		while let Some(token) = iter.next() {
			match token {
				_ => ,
			}
		}
		!
	}
}

pub enum ParserResult {
	WaitNext,
	Unexpected(Token)
}