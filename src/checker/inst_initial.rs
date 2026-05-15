use crate::{
    checker::Checker,
    error::{Error, loc_error},
    structure::{
        ast::{Instruction, InstructionKind, Type, Value},
        symbols::Symbol,
    },
};

impl<'a> Checker<'a> {
    pub(super) fn check_inst_initial(&mut self, inst: &'a Instruction<'a>) -> Result<(), Error> {
        match &inst.kind {
            InstructionKind::Import { .. } => {
                loc_error(inst.line, "Imports should not be passed to the checker")
            }
            InstructionKind::Decl { id, .. } => self.check_decl_initial(id),
            InstructionKind::Fn { id, params, .. } => self.check_fn_initial(inst, id, params),
            InstructionKind::Ret { .. } => self.check_ret_initial(inst),
            InstructionKind::Call { op, id, args } => self.check_call_initial(inst, op, id, args),
            InstructionKind::Label { id } => self.check_label_initial(inst, id),
            InstructionKind::UnaryOp { id, .. } => self.check_unary_initial(id),
            InstructionKind::BinaryOp { id, .. } => self.check_binary_initial(id),
            _ => Ok(()),
        }
    }

    fn check_decl_initial(&mut self, id: &'a str) -> Result<(), Error> {
        let mangled_id = self.mangle_id(id);

        self.symbols.insert(
            mangled_id,
            Symbol {
                id: mangled_id,
                ty: Type::Undeclared,
                is_fn: false,
                is_visible: true,
                mangled_params: Vec::new(),
            },
        );

        Ok(())
    }

    fn check_fn_initial(
        &mut self,
        inst: &Instruction<'a>,
        id: &'a str,
        params: &'a Vec<(Type<'a>, &str)>,
    ) -> Result<(), Error> {
        let mangled_id = self.mangle_id(id);

        if self.symbols.contains_key(mangled_id) {
            return loc_error(
                inst.line,
                format!("Symbol '{}' has already been declared", id).as_str(),
            );
        }

        let mut mangled_params = Vec::new();

        for (_, p_id) in params {
            let mangled_p_id = self.mangle_id(p_id);

            if self.symbols.contains_key(mangled_p_id) {
                return loc_error(
                    inst.line,
                    format!("Symbol '{}' has already been declared", p_id).as_str(),
                );
            }

            mangled_params.push(mangled_p_id);

            self.symbols.insert(
                mangled_p_id,
                Symbol {
                    id: mangled_p_id,
                    ty: Type::Undeclared,
                    is_fn: false,
                    is_visible: true,
                    mangled_params: Vec::new(),
                },
            );
        }

        self.symbols.insert(
            mangled_id,
            Symbol {
                id: mangled_id,
                ty: Type::Undeclared,
                is_fn: true,
                is_visible: true,
                mangled_params,
            },
        );

        self.working_funcs.push(mangled_id);

        Ok(())
    }

    fn check_ret_initial(&mut self, inst: &Instruction<'a>) -> Result<(), Error> {
        if self.working_funcs.last().is_none() {
            return loc_error(inst.line, "Unmatched return");
        }

        self.working_funcs.pop();

        Ok(())
    }

    fn check_call_initial(
        &mut self,
        inst: &Instruction<'a>,
        op: &'a str,
        id: &'a str,
        args: &Vec<Value<'a>>,
    ) -> Result<(), Error> {
        let resolved_op = self.resolve_id(op);

        let Some(fn_sym) = self.symbols.get(resolved_op) else {
            return loc_error(
                inst.line,
                format!("Symbol '{}' never declared", op).as_str(),
            );
        };

        if !fn_sym.is_fn {
            return loc_error(
                inst.line,
                format!("Symbol '{}' is not callable", op).as_str(),
            );
        }

        if args.len() != fn_sym.mangled_params.len() {
            return loc_error(
                inst.line,
                format!(
                    "Symbol '{}' expected {} arguments but {} were provided",
                    op,
                    fn_sym.mangled_params.len(),
                    args.len()
                )
                .as_str(),
            );
        }

        let mangled_id = self.mangle_id(id);

        self.symbols.insert(
            mangled_id,
            Symbol {
                id: mangled_id,
                ty: Type::Undeclared,
                is_fn: false,
                is_visible: true,
                mangled_params: Vec::new(),
            },
        );

        Ok(())
    }

    fn check_label_initial(&mut self, inst: &Instruction<'a>, id: &'a str) -> Result<(), Error> {
        let mangled_id = self.mangle_id(id);

        if self.symbols.contains_key(mangled_id) {
            return loc_error(
                inst.line,
                format!("Symbol '{}' has already been declared", id).as_str(),
            );
        }

        self.symbols.insert(
            mangled_id,
            Symbol {
                id: mangled_id,
                ty: Type::Label,
                is_fn: false,
                is_visible: true,
                mangled_params: Vec::new(),
            },
        );

        Ok(())
    }

    fn check_unary_initial(&mut self, id: &'a str) -> Result<(), Error> {
        let mangled_id = self.mangle_id(id);

        self.symbols.insert(
            mangled_id,
            Symbol {
                id: mangled_id,
                ty: Type::Undeclared,
                is_fn: false,
                is_visible: true,
                mangled_params: Vec::new(),
            },
        );

        Ok(())
    }

    fn check_binary_initial(&mut self, id: &'a str) -> Result<(), Error> {
        let mangled_id = self.mangle_id(id);

        self.symbols.insert(
            mangled_id,
            Symbol {
                id: mangled_id,
                ty: Type::Undeclared,
                is_fn: false,
                is_visible: true,
                mangled_params: Vec::new(),
            },
        );

        Ok(())
    }
}
