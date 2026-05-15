use crate::{
    error::{Error, loc_error},
    parser::Parser,
    structure::{ast::Value, token::Token},
};

impl<'a> Parser<'a> {
    pub(crate) fn parse_value(&self, token: Token) -> Result<Value<'a>, Error> {
        match token {
            Token::Identifier => Ok(Value::Symbol(self.lexer.slice())),
            Token::True => Ok(Value::Bool(true)),
            Token::False => Ok(Value::Bool(false)),
            Token::StringLiteral => Ok(Value::String(self.lexer.slice())),
            Token::Number => self.parse_number(),
            _ => loc_error(self.line, "Invalid value"),
        }
    }

    fn parse_number(&self) -> Result<Value<'a>, Error> {
        let num = self.lexer.slice();

        if num.contains(".") {
            let val = num
                .parse::<f64>()
                .or_else(|_| loc_error(self.line, "Invalid floating point number"))?;

            if val as f32 as f64 == val {
                return Ok(Value::Float32(val as f32));
            } else {
                return Ok(Value::Float64(val));
            }
        }

        if num.contains("-") {
            let val = num
                .parse::<i128>()
                .or_else(|_| loc_error(self.line, "Invalid signed integer"))?;

            return if i8::MIN as i128 <= val && val <= i8::MAX as i128 {
                Ok(Value::Int8(val as i8))
            } else if i16::MIN as i128 <= val && val <= i16::MAX as i128 {
                Ok(Value::Int16(val as i16))
            } else if i32::MIN as i128 <= val && val <= i32::MAX as i128 {
                Ok(Value::Int32(val as i32))
            } else if i64::MIN as i128 <= val && val <= i64::MAX as i128 {
                Ok(Value::Int64(val as i64))
            } else {
                Ok(Value::Int128(val))
            };
        }

        let val = num
            .parse::<u128>()
            .or_else(|_| loc_error(self.line, "Invalid unsigned integer"))?;

        if u8::MIN as u128 <= val && val <= u8::MAX as u128 {
            Ok(Value::UInt8(val as u8))
        } else if u16::MIN as u128 <= val && val <= u16::MAX as u128 {
            Ok(Value::UInt16(val as u16))
        } else if u32::MIN as u128 <= val && val <= u32::MAX as u128 {
            Ok(Value::UInt32(val as u32))
        } else if u64::MIN as u128 <= val && val <= u64::MAX as u128 {
            Ok(Value::UInt64(val as u64))
        } else {
            Ok(Value::UInt128(val))
        }
    }
}
