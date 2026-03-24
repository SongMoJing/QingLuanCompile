use crate::_lib::io::{Log, LogType, Span};
use crate::parser_qln::model_ast::{ASTType, Node, AST};
use crate::parser_qln::model_token::{Key, KeyStruct, Token, TokenKind};
use colored::Colorize;
use rust_i18n::t;
use std::collections::VecDeque;
use std::path::Path;
use unicode_width::UnicodeWidthChar;

pub struct ASTParser {
    tokens: VecDeque<Token>,
    ast: AST,
    pending_nodes: Vec<Node>,
    errors: Vec<(ParseError, Span)>,
}

impl ASTParser {
    pub fn new(tokens: VecDeque<Token>, ast_type: ASTType) -> Self {
        let ast = AST::new(ast_type);
        Self {
            tokens,
            ast,
            pending_nodes: vec![],
            errors: vec![],
        }
    }

    pub fn get_ast(&self) -> AST {
        self.ast.clone()
    }

    pub fn get_errors(&self) -> Vec<(ParseError, Span)> {
        self.errors.clone()
    }

    pub fn parse(&mut self) -> (AST, Vec<(ParseError, Span)>) {
        while let Some(token) = self.tokens.pop_front() {
            if let TokenKind::Key(Key::StructKey(key)) = &token.kind {
                match key {
                    KeyStruct::Import => self.parse_import(),
                    KeyStruct::Mod => self.parse_mod(),
                    KeyStruct::Fn => self.parse_fn(),
                    _ => self.errors.push((
                        ParseError::UnexpectedToken(
	                        token.kind,
                            vec![
                                TokenKind::Key(Key::StructKey(KeyStruct::Import)),
                                TokenKind::Key(Key::StructKey(KeyStruct::Mod)),
                                TokenKind::Key(Key::StructKey(KeyStruct::Fn)),
                            ],
                        ),
                        token.span,
                    ))
                }
            } else {
	            return (self.ast.clone(), self.errors.clone());
                // self.errors.push((
                //     ParseError::UnexpectedToken(
	            //         token.kind,
                //         vec![
                //             TokenKind::Key(Key::StructKey(KeyStruct::Import)),
                //             TokenKind::Key(Key::StructKey(KeyStruct::Mod)),
                //             TokenKind::Key(Key::StructKey(KeyStruct::Fn)),
                //         ],
                //     ),
                //     token.span,
                // ));
            }
        }
        (self.ast.clone(), self.errors.clone())
    }

    fn parse_import(&mut self) {
	    // import ql.std.console.out as console; -> name: 'console', path: 'ql.std.console.out'
	    // import ql.std.console.in; -> name: 'in', path: 'ql.std.console.in'
	    // import ql.lang.type {
	    //     String as str, -> name: 'str', path: 'ql.lang.type.String'
	    //     List, -> name: 'List', path: 'ql.lang.type.List'
	    //     Int.byte1 as I8 -> name: 'I8', path: 'ql.lang.type.Int.byte1'
	    // };
    }

    fn parse_mod(&mut self) {}

    fn parse_class(&mut self) {}

    fn parse_static(&mut self) {}

    fn parse_attr(&mut self) {}

    fn parse_init(&mut self) {}

    fn parse_fn(&mut self) {}
}

#[derive(Clone)]
pub enum ParseError {
    /// 意外的符号
    UnexpectedToken(TokenKind, Vec<TokenKind>),
    /// 意外终止
    UnexpectedEnd,
}

/// # 错误报告器
pub struct ParseErrorReporter {
    source: String,
    file: String,
    errors: Vec<(ParseError, Span)>,
}

impl ParseErrorReporter {
    pub fn new(source: &str, file: &Path) -> Self {
        ParseErrorReporter {
            source: source.to_string(),
            file: file.to_string_lossy().to_string().replace("\\", "/"),
            errors: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: ParseError, span: Span) {
        self.errors.push((error, span));
    }

    pub fn report(&self) {
        for (error, span) in &self.errors {
            let line_num = span.start_line;
            let line_str = self.get_line_content(line_num - 1);

            let message = match error {
                ParseError::UnexpectedToken(token, expect) => {
                    let mut str = String::new();
                    for token in expect {
                        str.push_str(&format!("'{}', ", token));
                    }
                    str.pop();
                    str.pop();
                    t!(
                        "log.Err.ParseError.UnexpectedToken",
                        token = token,
                        expect = str
                    )
                    .to_string()
                }
                ParseError::UnexpectedEnd => t!("log.Err.ParseError.UnexpectedEnd").to_string(),
            };

            let line_num = line_num.to_string();
            Log::new(
                LogType::Err,
                &format!(
                    "{} {} \n{}{} {}:{}:{}",
                    format!("[E{:04}]", self.error_code(error)).bright_cyan(),
                    message,
                    " ".repeat(line_num.len()),
                    "-->".bright_cyan(),
                    self.file,
                    line_num,
                    span.start_col
                ),
            )
            .print();
            println!(" {} {}", " ".repeat(line_num.len()), "|".bright_cyan());
            println!(
                " {} {} {}",
                line_num.bright_cyan(),
                "|".bright_cyan(),
                line_str.replace("\t", " ")
            );
            println!(
                " {} {}{}{}",
                " ".repeat(line_num.len()),
                "|".bright_cyan(),
                " ".repeat(get_len(&line_str.replace("\t", " "), span.start_col)),
                "^".repeat(span.end_col.saturating_sub(span.start_col))
                    .bright_yellow()
            );
        }

        fn get_len(line_content: &str, col: usize) -> usize {
            line_content
                .chars()
                .take(col)
                .map(|c| c.width().unwrap_or(1)) // 全角字符宽度为2
                .sum()
        }
    }

    fn get_line_content(&self, line_num: usize) -> &str {
        self.source.lines().nth(line_num).unwrap_or("")
    }

    fn error_code(&self, error: &ParseError) -> u16 {
        match error {
            ParseError::UnexpectedToken(_, _) => 2001,
            ParseError::UnexpectedEnd => 2002,
        }
    }
}
