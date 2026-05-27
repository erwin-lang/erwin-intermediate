use crate::{
    assembler::AssemblerAMD64,
    error::{Error, loc_error},
    structure::{
        ast::{Instruction, InstructionKind},
        sizes::Size,
        value::Value,
    },
};

impl<'a> AssemblerAMD64<'a> {
    pub(super) fn assemble_inst(&mut self, inst: &Instruction<'a>) -> Result<(), Error> {
        match &inst.kind {
            InstructionKind::Import { .. } => {
                loc_error(inst.line, "Imports should not be passed to the assembler")
            }
            InstructionKind::Decl { id, value, .. } => self.assemble_decl(inst, id, *value),
            _ => todo!(),
        }
    }

    fn assemble_decl(
        &mut self,
        inst: &Instruction<'a>,
        id: &str,
        value: Value<'a>,
    ) -> Result<(), Error> {
        let sym = self.resolve_sym(id);
        let sym_id = sym.id;
        let sym_stack_offset = sym.stack_offset;

        if sym.is_global {
            self.data.insert(sym_id, (value, inst.line));
        } else {
            match value {
                Value::Symbol(val_id) => {
                    let val_sym = self.resolve_sym(val_id);
                    let size = val_sym.ty.size(inst.line, self.arena)?;
                    let reg = match size {
                        Size::B1 => "al",
                        Size::B2 => "ax",
                        Size::B4 => "eax",
                        _ => "rax",
                    };

                    self.assembly_output.push_str(
                        format!(
                            "\tmov {}, [rbp - {}]\n\tmov {} [rbp - {}], {}\n",
                            reg,
                            val_sym.stack_offset,
                            val_sym.ty.size(inst.line, self.arena)?.size_str(),
                            sym_stack_offset,
                            reg
                        )
                        .as_str(),
                    );
                }
                Value::Bool(v) => {
                    self.assembly_output.push_str(
                        format!("\tmov byte [rbp - {}], {}\n", sym_stack_offset, v as u8).as_str(),
                    );
                }
                Value::String(_) => {
                    let str_id = self.arena.dyn_alloc(format!("str_{}", self.string_count));
                    self.rodata.insert(str_id, (value, inst.line));
                    self.assembly_output.push_str(
                        format!(
                            "\tlea rax, [rel {}]\n\tmov qword [rbp - {}], rax\n",
                            str_id, sym_stack_offset
                        )
                        .as_str(),
                    );
                    self.string_count += 1;
                }
                Value::UInt8(v) => {
                    self.assembly_output.push_str(
                        format!("\tmov byte [rbp - {}], {}\n", sym_stack_offset, v).as_str(),
                    );
                }
                Value::UInt16(v) => {
                    self.assembly_output.push_str(
                        format!("\tmov word [rbp - {}], {}\n", sym_stack_offset, v).as_str(),
                    );
                }
                Value::UInt32(v) => {
                    self.assembly_output.push_str(
                        format!("\tmov dword [rbp - {}], {}\n", sym_stack_offset, v).as_str(),
                    );
                }
                Value::UInt64(v) => {
                    self.assembly_output.push_str(
                        format!("\tmov qword [rbp - {}], {}\n", sym_stack_offset, v).as_str(),
                    );
                }
                Value::UInt128(_) => {
                    let const_id = self.arena.dyn_alloc(format!("u128_{}", self.const_count));
                    self.rodata.insert(const_id, (value, inst.line));
                    self.assembly_output.push_str(
                        format!(
                            "\tlea rax [rel {}]\n\tmov qword [rbp - {}], rax\n",
                            const_id, sym_stack_offset
                        )
                        .as_str(),
                    );
                    self.const_count += 1;
                }
                Value::Int8(v) => {
                    self.assembly_output.push_str(
                        format!("\tmov byte [rbp - {}], {}\n", sym_stack_offset, v).as_str(),
                    );
                }
                Value::Int16(v) => {
                    self.assembly_output.push_str(
                        format!("\tmov word [rbp - {}], {}\n", sym_stack_offset, v).as_str(),
                    );
                }
                Value::Int32(v) => {
                    self.assembly_output.push_str(
                        format!("\tmov dword [rbp - {}], {}\n", sym_stack_offset, v).as_str(),
                    );
                }
                Value::Int64(v) => {
                    self.assembly_output.push_str(
                        format!("\tmov qword [rbp - {}], {}\n", sym_stack_offset, v).as_str(),
                    );
                }
                Value::Int128(_) => {
                    let const_id = self.arena.dyn_alloc(format!("i128_{}", self.const_count));
                    self.rodata.insert(const_id, (value, inst.line));
                    self.assembly_output.push_str(
                        format!(
                            "\tlea rax [rel {}]\n\tmov qword [rbp - {}], rax\n",
                            const_id, sym_stack_offset
                        )
                        .as_str(),
                    );
                    self.const_count += 1;
                }
                Value::Float32(_) => {
                    let const_id = self.arena.dyn_alloc(format!("f32_{}", self.const_count));
                    self.rodata.insert(const_id, (value, inst.line));
                    self.assembly_output.push_str(
                        format!(
                            "\tmovss xmm0, [rel {}]\n\tmovss dword [rbp - {}], xmm0\n",
                            const_id, sym_stack_offset
                        )
                        .as_str(),
                    );
                    self.const_count += 1;
                }
                Value::Float64(_) => {
                    let const_id = self.arena.dyn_alloc(format!("f64_{}", self.const_count));
                    self.rodata.insert(const_id, (value, inst.line));
                    self.assembly_output.push_str(
                        format!(
                            "\tmovsd xmm0, [rel {}]\n\tmovsd qword [rbp - {}], xmm0\n",
                            const_id, sym_stack_offset
                        )
                        .as_str(),
                    );
                    self.const_count += 1;
                }
            }
        }

        Ok(())
    }
}
