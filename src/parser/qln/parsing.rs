use crate::parser::qln::token::Token;
use crate::parser::qln::ast::AST;
use crate::parser::qln::ast::ASTType::File;

pub struct ASTParser {
	pub tokens: Vec<Token>,
	current: i128,
	ast: AST,
	errors: Vec<(Token, Token)>,
}

impl ASTParser {
	pub fn new(tokens: Vec<Token>) -> Self {
		let parser = ASTParser {
			tokens,
			current: 0,
			ast: AST::new(File),
			errors: vec![],
		};
		parser
	}
	
	pub fn parse(&mut self) -> AST {
		self.ast.clone()
	}
}