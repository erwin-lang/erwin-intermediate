pub(super) mod inst;
pub(super) mod ty;
pub(super) mod value;

use logos::Lexer;

use crate::{
    arena::Arena,
    error::{Error, loc_error},
    structure::{ast::Instruction, token::Token},
};

pub(crate) struct Parser<'a> {
    arena: &'a Arena<'a>,
    lexer: Lexer<'a, Token>,

    insts: Vec<Instruction<'a>>,
    line: usize,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(arena: &'a Arena<'a>, lexer: Lexer<'a, Token>) -> Self {
        Self {
            arena,
            lexer,

            insts: Vec::new(),
            line: 1,
        }
    }

    pub(crate) fn parse(mut self) -> Result<Vec<Instruction<'a>>, Error> {
        while let Some(Ok(token)) = self.lexer.next() {
            self.parse_inst(token)?;
        }

        Ok(self.insts)
    }

    pub(super) fn advance(&mut self) -> Token {
        if let Some(Ok(token)) = self.lexer.next() {
            return token;
        }

        Token::Eof
    }

    pub(super) fn next_and_consume(&mut self, expected: Token) -> Result<&'a str, Error> {
        let token = self.advance();

        if token != expected {
            return loc_error(self.line, format!("Expected '{:?}'", expected).as_str());
        }

        Ok(self.lexer.slice())
    }
}
