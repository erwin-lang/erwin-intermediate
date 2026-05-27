use crate::{
    checker::Checker,
    error::{Error, loc_error},
    structure::{
        ast::{BinaryOp, Instruction, InstructionKind, UnaryOp},
        types::Type,
        value::Value,
    },
};

impl<'a> Checker<'a> {
    pub(super) fn check_inst(&mut self, inst: &'a Instruction<'a>) -> Result<(), Error> {
        match &inst.kind {
            InstructionKind::Import { .. } => {
                loc_error(inst.line, "Imports should not be passed to the checker")
            }
            InstructionKind::Decl { ty, id, value } => self.check_decl(inst, *ty, id, *value),
            InstructionKind::Assign { id, value } => self.check_assign(inst, id, *value),
            InstructionKind::Fn { ty, id, params } => self.check_fn(inst, *ty, id, params),
            InstructionKind::Ret { ret } => self.check_ret(inst, *ret),
            InstructionKind::Call { op, id, args } => self.check_call(inst, op, id, args),
            InstructionKind::Label { id } => self.check_label(id),
            InstructionKind::Jump { cond, label } => self.check_jump(inst, *cond, label),
            InstructionKind::UnaryOp { op, id, val } => self.check_unary(inst, *op, id, *val),
            InstructionKind::BinaryOp { op, id, a, b } => self.check_binary(inst, *op, id, *a, *b),
        }
    }

    fn check_decl(
        &mut self,
        inst: &Instruction<'a>,
        ty: Type<'a>,
        id: &'a str,
        value: Value<'a>,
    ) -> Result<(), Error> {
        let val_ty = self.get_value_ty(value, self.inst_index, inst.line)?;
        let resolved_id = self.resolve_id_logged(self.inst_index, id);

        if self.forward_jump.is_some() {
            self.jumped_symbols.insert(resolved_id);
        }

        let sym = self.symbols.get_mut(resolved_id).unwrap();

        if !val_ty.is_assignable_to(ty) {
            return loc_error(
                inst.line,
                format!(
                    "Type '{}' is not assignable to '{}'",
                    val_ty.as_str(self.arena),
                    ty.as_str(self.arena)
                )
                .as_str(),
            );
        }

        self.current_stack_offset -= ty.size(inst.line, self.arena)? as i32;
        sym.stack_offset = self.current_stack_offset;
        sym.ty = ty;

        Ok(())
    }

    fn check_assign(
        &mut self,
        inst: &Instruction<'a>,
        id: &'a str,
        value: Value<'a>,
    ) -> Result<(), Error> {
        let val_ty = self.get_value_ty(value, self.inst_index, inst.line)?;
        let resolved_id = self.resolve_id_logged(self.inst_index, id);
        let Some(sym) = self.symbols.get(resolved_id) else {
            return loc_error(
                inst.line,
                format!("Symbol '{}' never declared", id).as_str(),
            );
        };

        if !sym.is_visible {
            return loc_error(
                inst.line,
                format!("Symbol '{}' is not visible", id).as_str(),
            );
        }

        if sym.is_global {
            return loc_error(
                inst.line,
                format!("Symbol '{}' is global and cannot be assigned", id).as_str(),
            );
        }

        if sym.is_fn {
            return loc_error(
                inst.line,
                format!("Symbol '{}' is callable and cannot be assigned", id).as_str(),
            );
        }

        if !sym.is_reassignable {
            return loc_error(
                inst.line,
                format!("Symbol '{}' is not assignable", id).as_str(),
            );
        }

        if !val_ty.is_assignable_to(sym.ty) {
            return loc_error(
                inst.line,
                format!(
                    "Type '{}' is not assignable to '{}'",
                    val_ty.as_str(self.arena),
                    sym.ty.as_str(self.arena)
                )
                .as_str(),
            );
        }

        Ok(())
    }

    fn check_fn(
        &mut self,
        inst: &Instruction<'a>,
        ty: Type<'a>,
        id: &'a str,
        params: &'a [(Type<'a>, &str)],
    ) -> Result<(), Error> {
        let resolved_id = self.resolve_id_logged(self.inst_index, id);

        if self.forward_jump.is_some() {
            self.jumped_symbols.insert(resolved_id);
        }

        for (p_ty, p_id) in params {
            let resolved_p_id = self.resolve_id_logged(self.inst_index, p_id);

            if self.forward_jump.is_some() {
                self.jumped_symbols.insert(resolved_p_id);
            }

            let p_sym = self.symbols.get_mut(resolved_p_id).unwrap();

            self.current_stack_offset -= p_ty.size(inst.line, self.arena)? as i32;
            p_sym.stack_offset = self.current_stack_offset;
            p_sym.ty = *p_ty;
        }

        let sym = self.symbols.get_mut(resolved_id).unwrap();

        self.current_stack_offset -= ty.size(inst.line, self.arena)? as i32;
        sym.stack_offset = self.current_stack_offset;
        sym.ty = ty;
        self.working_funcs.push(resolved_id);

        Ok(())
    }

    fn check_ret(&mut self, inst: &Instruction<'a>, ret: Value<'a>) -> Result<(), Error> {
        let val_ty = self.get_value_ty(ret, self.inst_index, inst.line)?;
        let Some(fn_id) = self.working_funcs.last() else {
            return loc_error(inst.line, "Unmatched return");
        };
        let fn_sym = self.symbols.get(fn_id).unwrap();

        if !val_ty.is_assignable_to(fn_sym.ty) {
            return loc_error(
                inst.line,
                format!(
                    "Type '{}' is not assignable to '{}'",
                    val_ty.as_str(self.arena),
                    fn_sym.ty.as_str(self.arena)
                )
                .as_str(),
            );
        }

        self.working_funcs.pop();

        Ok(())
    }

    fn check_call(
        &mut self,
        inst: &Instruction<'a>,
        op: &'a str,
        id: &'a str,
        args: &[Value<'a>],
    ) -> Result<(), Error> {
        let resolved_op = self.resolve_id_logged(self.inst_index, op);

        let mut arg_types = Vec::new();
        for arg in args {
            arg_types.push(self.get_value_ty(*arg, self.inst_index, inst.line)?);
        }

        let fn_sym = self.symbols.get(resolved_op).unwrap();

        if !fn_sym.is_visible {
            return loc_error(
                inst.line,
                format!("Symbol '{}' is not visible", op).as_str(),
            );
        }

        if !fn_sym.is_fn {
            return loc_error(
                inst.line,
                format!("Symbol '{}' is not a function", op).as_str(),
            );
        }

        let ty = fn_sym.ty;

        for (arg_ty, mangled_param) in arg_types.iter().zip(&fn_sym.mangled_params) {
            let param_ty = self.symbols.get(mangled_param).unwrap().ty;

            if !arg_ty.is_assignable_to(param_ty) {
                return loc_error(
                    inst.line,
                    format!(
                        "Type '{}' is not assignable to '{}'",
                        arg_ty.as_str(self.arena),
                        param_ty.as_str(self.arena)
                    )
                    .as_str(),
                );
            }
        }

        let resolved_id = self.resolve_id_logged(self.inst_index, id);

        if self.forward_jump.is_some() {
            self.jumped_symbols.insert(resolved_id);
        }

        let sym = self.symbols.get_mut(resolved_id).unwrap();

        self.current_stack_offset -= ty.size(inst.line, self.arena)? as i32;
        sym.stack_offset = self.current_stack_offset;
        sym.ty = ty;

        Ok(())
    }

    fn check_label(&mut self, id: &'a str) -> Result<(), Error> {
        let resolved_id = self.resolve_id_logged(self.inst_index, id);

        if self.forward_jump.is_some() {
            self.jumped_symbols.insert(resolved_id);
        }

        self.passed_labels.insert(resolved_id);

        if let Some(resolved_label) = self.forward_jump
            && resolved_label == resolved_id
        {
            self.forward_jump = None;

            for jumped_id in &self.jumped_symbols {
                let jumped_sym = self.symbols.get_mut(jumped_id).unwrap();

                jumped_sym.is_visible = false;
            }

            self.jumped_symbols.clear();
        }

        Ok(())
    }

    fn check_jump(
        &mut self,
        inst: &Instruction<'a>,
        cond: Value<'a>,
        label: &'a str,
    ) -> Result<(), Error> {
        let cond_ty = self.get_value_ty(cond, self.inst_index, inst.line)?;

        if !matches!(cond_ty, Type::Bool) {
            return loc_error(inst.line, "Condition must have bool type");
        }

        let resolved_label = self.resolve_id_logged(self.inst_index, label);

        let Some(sym) = self.symbols.get(resolved_label) else {
            return loc_error(
                inst.line,
                format!("Symbol '{}' never declared", label).as_str(),
            );
        };

        if !sym.is_visible {
            return loc_error(
                inst.line,
                format!("Symbol '{}' is not visible", label).as_str(),
            );
        }

        if !matches!(sym.ty, Type::Label) {
            return loc_error(inst.line, "Jump target must be a label");
        }

        if self.passed_labels.contains(resolved_label) {
            self.forward_jump = None;
        } else {
            self.forward_jump = Some(resolved_label);
        }

        Ok(())
    }

    fn check_unary(
        &mut self,
        inst: &Instruction<'a>,
        op: UnaryOp,
        id: &'a str,
        val: Value<'a>,
    ) -> Result<(), Error> {
        let resolved_id = self.resolve_id_logged(self.inst_index, id);

        if self.forward_jump.is_some() {
            self.jumped_symbols.insert(resolved_id);
        }

        let val_ty = self.get_value_ty(val, self.inst_index, inst.line)?;
        let sym = self.symbols.get_mut(resolved_id).unwrap();

        let final_ty = match op {
            UnaryOp::Neg => {
                if !matches!(
                    val_ty,
                    Type::Int8
                        | Type::Int16
                        | Type::Int32
                        | Type::Int64
                        | Type::Int128
                        | Type::Float32
                        | Type::Float64
                ) {
                    return loc_error(inst.line, "Negation requires a float or integer type");
                }

                val_ty
            }
            UnaryOp::Not => {
                if !matches!(
                    val_ty,
                    Type::Bool
                        | Type::UInt8
                        | Type::UInt16
                        | Type::UInt32
                        | Type::UInt64
                        | Type::UInt128
                        | Type::Int8
                        | Type::Int16
                        | Type::Int32
                        | Type::Int64
                        | Type::Int128
                ) {
                    return loc_error(inst.line, "Logical not requires bool or integer type");
                }

                val_ty
            }
            UnaryOp::Ref => Type::Ptr(self.arena.sized_alloc(val_ty)),
            UnaryOp::Deref => {
                if let Type::Ptr(inner_ty) = val_ty {
                    *inner_ty
                } else {
                    return loc_error(inst.line, "Cannot dereference a non-pointer type");
                }
            }
            UnaryOp::Lsh | UnaryOp::Rsh => {
                if !matches!(
                    val_ty,
                    Type::UInt8
                        | Type::UInt16
                        | Type::UInt32
                        | Type::UInt64
                        | Type::UInt128
                        | Type::Int8
                        | Type::Int16
                        | Type::Int32
                        | Type::Int64
                        | Type::Int128
                ) {
                    return loc_error(inst.line, "Bitwise shift requires an integer type");
                }

                val_ty
            }
        };

        self.current_stack_offset -= final_ty.size(inst.line, self.arena)? as i32;
        sym.stack_offset = self.current_stack_offset;
        sym.ty = final_ty;

        Ok(())
    }

    fn check_binary(
        &mut self,
        inst: &Instruction<'a>,
        op: BinaryOp,
        id: &'a str,
        a: Value<'a>,
        b: Value<'a>,
    ) -> Result<(), Error> {
        let resolved_id = self.resolve_id_logged(self.inst_index, id);

        if self.forward_jump.is_some() {
            self.jumped_symbols.insert(resolved_id);
        }

        let a_ty = self.get_value_ty(a, self.inst_index, inst.line)?;
        let b_ty = self.get_value_ty(b, self.inst_index, inst.line)?;
        let sym = self.symbols.get_mut(resolved_id).unwrap();

        let final_ty = a_ty.join(b_ty, self.arena, inst.line)?;

        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mult | BinaryOp::Div => {
                if !matches!(
                    final_ty,
                    Type::UInt8
                        | Type::UInt16
                        | Type::UInt32
                        | Type::UInt64
                        | Type::UInt128
                        | Type::Int8
                        | Type::Int16
                        | Type::Int32
                        | Type::Int64
                        | Type::Int128
                        | Type::Float32
                        | Type::Float64
                ) {
                    return loc_error(
                        inst.line,
                        "Arithmetic operations require integer or float types",
                    );
                }
            }
            BinaryOp::And | BinaryOp::Or | BinaryOp::Xor => {
                if !matches!(
                    final_ty,
                    Type::Bool
                        | Type::UInt8
                        | Type::UInt16
                        | Type::UInt32
                        | Type::UInt64
                        | Type::UInt128
                        | Type::Int8
                        | Type::Int16
                        | Type::Int32
                        | Type::Int64
                        | Type::Int128
                ) {
                    return loc_error(
                        inst.line,
                        "Binary logical operations require bool or integer types",
                    );
                }
            }
        }

        self.current_stack_offset -= final_ty.size(inst.line, self.arena)? as i32;
        sym.stack_offset = self.current_stack_offset;
        sym.ty = final_ty;

        Ok(())
    }
}
