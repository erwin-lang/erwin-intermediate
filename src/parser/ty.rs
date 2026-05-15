use crate::{
    error::{Error, loc_error},
    parser::Parser,
    structure::{ast::Type, token::Token},
};

impl<'a> Parser<'a> {
    pub(crate) fn parse_type(&mut self, token: Token) -> Result<Type<'a>, Error> {
        match token {
            Token::Bool => Ok(Type::Bool),
            Token::String => Ok(Type::String),
            Token::UInt8 => Ok(Type::UInt8),
            Token::UInt16 => Ok(Type::UInt16),
            Token::UInt32 => Ok(Type::UInt32),
            Token::UInt64 => Ok(Type::UInt64),
            Token::UInt128 => Ok(Type::UInt128),
            Token::Int8 => Ok(Type::Int8),
            Token::Int16 => Ok(Type::Int16),
            Token::Int32 => Ok(Type::Int32),
            Token::Int64 => Ok(Type::Int64),
            Token::Int128 => Ok(Type::Int128),
            Token::Float32 => Ok(Type::Float32),
            Token::Float64 => Ok(Type::Float64),
            Token::Ptr => {
                let inner = self.advance();
                Ok(Type::Ptr(self.arena.sized_alloc(self.parse_type(inner)?)))
            }
            _ => loc_error(self.line, "Invalid type"),
        }
    }
}
