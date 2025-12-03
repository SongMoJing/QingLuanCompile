use crate::_lib::io::Span;
use crate::parser::qln::ast::{ASTType, Node, AST};
use crate::parser::qln::token::{Key, KeyStruct, Token, TokenKind};
use std::rc::Rc;

pub struct ASTParser {
    tokens: Vec<Token>,
    current: i128,
    ast: AST,
    stack: Vec<Rc<Node>>,
    errors: Vec<(TokenKind, Span)>,
}

impl ASTParser {
    pub fn new(tokens: Vec<Token>, ast_type: ASTType) -> Self {
        let ast = AST::new(ast_type);
        let root = Rc::new(ast.get_root().clone());

        Self {
            tokens,
            current: 0,
            ast,
            stack: vec![root],
            errors: vec![],
        }
    }

    pub fn parse(&mut self) {
        for token in self.tokens.iter() {
	        if let TokenKind::Key(Key::StructKey(key)) = token.kind.clone() {
		        match key {
			        KeyStruct::Import => {

			        }
			        KeyStruct::Mod => {

			        }
			        KeyStruct::Class => {
			        }
			        KeyStruct::Static => {}
			        KeyStruct::Attr => {}
			        KeyStruct::Init => {}
			        KeyStruct::Fn => {}
		        }
	        }
        }
    }

    pub fn get_ast(&self) -> AST {
        self.ast.clone()
    }

    pub fn get_errors(&self) -> Vec<(TokenKind, Span)> {
        self.errors.clone()
    }
}
