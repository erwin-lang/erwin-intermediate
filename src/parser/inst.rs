use std::path::Path;

use crate::{
    error::{Error, loc_error},
    parser::Parser,
    structure::{
        ast::{BinaryOp, Instruction, InstructionKind, UnaryOp},
        token::Token,
    },
};

impl<'a> Parser<'a> {
    pub(super) fn parse_inst(&mut self, token: Token) -> Result<(), Error> {
        match token {
            Token::Import => self.parse_import(),
            Token::Decl => self.parse_decl(),
            Token::Assign => self.parse_assign(),
            Token::Fn => self.parse_fn(),
            Token::Ret => self.parse_ret(),
            Token::Identifier => self.parse_call(),
            Token::Label => self.parse_label(),
            Token::Jump => self.parse_jump(),
            Token::Neg | Token::Not | Token::Ref | Token::Deref | Token::Lsh | Token::Rsh => {
                self.parse_unary(token)
            }
            Token::Add
            | Token::Sub
            | Token::Mult
            | Token::Div
            | Token::And
            | Token::Or
            | Token::Xor => self.parse_binary(token),
            _ => loc_error(self.line, "Unknown instruction"),
        }
    }

    fn parse_import(&mut self) -> Result<(), Error> {
        let alias = self.next_and_consume(Token::Identifier)?;

        if alias.contains(".") {
            return loc_error(self.line, "Declaration identifier cannot have '.'");
        }

        let path = Path::new(self.next_and_consume(Token::StringLiteral)?);

        self.insts.push(Instruction {
            kind: InstructionKind::Import { alias, path },
            line: self.line,
        });

        if matches!(self.advance(), Token::Newline) {
            self.line += 1;
        }

        Ok(())
    }

    fn parse_decl(&mut self) -> Result<(), Error> {
        let ty_token = self.advance();
        let ty = self.parse_type(ty_token)?;

        let id = self.next_and_consume(Token::Identifier)?;

        if id.contains(".") {
            return loc_error(self.line, "Declaration identifier cannot have '.'");
        }

        let val_token = self.advance();
        let value = self.parse_value(val_token)?;

        self.insts.push(Instruction {
            kind: InstructionKind::Decl { ty, id, value },
            line: self.line,
        });

        if matches!(self.advance(), Token::Newline) {
            self.line += 1;
        }

        Ok(())
    }

    fn parse_assign(&mut self) -> Result<(), Error> {
        let id = self.next_and_consume(Token::Identifier)?;

        let val_token = self.advance();
        let value = self.parse_value(val_token)?;

        self.insts.push(Instruction {
            kind: InstructionKind::Assign { id, value },
            line: self.line,
        });

        if matches!(self.advance(), Token::Newline) {
            self.line += 1;
        }

        Ok(())
    }

    fn parse_fn(&mut self) -> Result<(), Error> {
        let line = self.line;

        let ty_token = self.advance();
        let ty = self.parse_type(ty_token)?;

        let id = self.next_and_consume(Token::Identifier)?;

        if id.contains(".") {
            return loc_error(self.line, "Declaration identifier cannot have '.'");
        }

        let mut params = Vec::new();
        loop {
            let token = self.advance();

            match token {
                Token::Newline => {
                    self.line += 1;
                    break;
                }
                Token::Eof => break,
                _ => {}
            }

            let p_ty = self.parse_type(token)?;
            let p_id = self.next_and_consume(Token::Identifier)?;
            params.push((p_ty, p_id));
        }

        self.insts.push(Instruction {
            kind: InstructionKind::Fn { ty, id, params },
            line,
        });

        Ok(())
    }

    fn parse_ret(&mut self) -> Result<(), Error> {
        let ret_token = self.advance();
        let ret = self.parse_value(ret_token)?;

        self.insts.push(Instruction {
            kind: InstructionKind::Ret { ret },
            line: self.line,
        });

        if matches!(self.advance(), Token::Newline) {
            self.line += 1;
        }

        Ok(())
    }

    fn parse_call(&mut self) -> Result<(), Error> {
        let line = self.line;

        let op = self.lexer.slice();
        let id = self.next_and_consume(Token::Identifier)?;

        if id.contains(".") {
            return loc_error(self.line, "Declaration identifier cannot have '.'");
        }

        let mut args = Vec::new();
        loop {
            let token = self.advance();

            match token {
                Token::Newline => {
                    self.line += 1;
                    break;
                }
                Token::Eof => break,
                _ => {}
            }

            args.push(self.parse_value(token)?);
        }

        self.insts.push(Instruction {
            kind: InstructionKind::Call { op, id, args },
            line,
        });

        Ok(())
    }

    fn parse_label(&mut self) -> Result<(), Error> {
        let id = self.next_and_consume(Token::Identifier)?;

        if id.contains(".") {
            return loc_error(self.line, "Declaration identifier cannot have '.'");
        }

        self.insts.push(Instruction {
            kind: InstructionKind::Label { id },
            line: self.line,
        });

        if matches!(self.advance(), Token::Newline) {
            self.line += 1;
        }

        Ok(())
    }

    fn parse_jump(&mut self) -> Result<(), Error> {
        let cond_token = self.advance();
        let cond = self.parse_value(cond_token)?;

        let label = self.next_and_consume(Token::Identifier)?;

        self.insts.push(Instruction {
            kind: InstructionKind::Jump { cond, label },
            line: self.line,
        });

        if matches!(self.advance(), Token::Newline) {
            self.line += 1;
        }

        Ok(())
    }

    fn parse_unary(&mut self, token: Token) -> Result<(), Error> {
        let op = match token {
            Token::Neg => UnaryOp::Neg,
            Token::Not => UnaryOp::Not,
            Token::Ref => UnaryOp::Ref,
            Token::Deref => UnaryOp::Deref,
            Token::Lsh => UnaryOp::Lsh,
            Token::Rsh => UnaryOp::Rsh,
            _ => return loc_error(self.line, "Invalid unary operator"),
        };

        let id = self.next_and_consume(Token::Identifier)?;

        if id.contains(".") {
            return loc_error(self.line, "Declaration identifier cannot have '.'");
        }

        let val_token = self.advance();
        let val = self.parse_value(val_token)?;

        self.insts.push(Instruction {
            kind: InstructionKind::UnaryOp { op, id, val },
            line: self.line,
        });

        if matches!(self.advance(), Token::Newline) {
            self.line += 1;
        }

        Ok(())
    }

    fn parse_binary(&mut self, token: Token) -> Result<(), Error> {
        let op = match token {
            Token::Add => BinaryOp::Add,
            Token::Sub => BinaryOp::Sub,
            Token::Mult => BinaryOp::Mult,
            Token::Div => BinaryOp::Div,
            Token::And => BinaryOp::And,
            Token::Or => BinaryOp::Or,
            Token::Xor => BinaryOp::Xor,
            _ => return loc_error(self.line, "Invalid binary operator"),
        };

        let id = self.next_and_consume(Token::Identifier)?;

        if id.contains(".") {
            return loc_error(self.line, "Declaration identifier cannot have '.'");
        }

        let a_token = self.advance();
        let a = self.parse_value(a_token)?;

        let b_token = self.advance();
        let b = self.parse_value(b_token)?;

        self.insts.push(Instruction {
            kind: InstructionKind::BinaryOp { op, id, a, b },
            line: self.line,
        });

        if matches!(self.advance(), Token::Newline) {
            self.line += 1;
        }

        Ok(())
    }
}
