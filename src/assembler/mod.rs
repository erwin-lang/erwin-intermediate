pub(super) mod inst;

use std::collections::HashMap;

use crate::{
    arena::Arena,
    error::Error,
    structure::{ast::Instruction, sizes::Size, symbol::Symbol, value::Value},
};

pub(crate) struct AssemblerAMD64<'a> {
    arena: &'a Arena<'a>,
    program: &'a [Instruction<'a>],
    symbols: &'a HashMap<&'a str, Symbol<'a>>,
    resolutions: &'a Vec<HashMap<&'a str, &'a str>>,

    inst_index: usize,                          // Index of the current instruction
    data: HashMap<&'a str, (Value<'a>, usize)>, // mangled_id, (value, line); for global symbols
    rodata: HashMap<&'a str, (Value<'a>, usize)>, // mangled_id, (value, line); for strings, floats, and large ints
    string_count: usize,                          // Counter for string names
    const_count: usize,                           // Counter for const names
    assembly_output: String,                      // Final AMD64 assembly code
}

impl<'a> AssemblerAMD64<'a> {
    pub(crate) fn new(
        arena: &'a Arena<'a>,
        program: &'a [Instruction<'a>],
        symbols: &'a HashMap<&str, Symbol<'a>>,
        resolutions: &'a Vec<HashMap<&'a str, &'a str>>,
    ) -> Self {
        Self {
            arena,
            program,
            symbols,
            resolutions,

            inst_index: 0,
            data: HashMap::new(),
            rodata: HashMap::new(),
            string_count: 0,
            const_count: 0,
            assembly_output: String::new(),
        }
    }

    pub(crate) fn assemble(mut self) -> Result<String, Error> {
        self.assembly_output.push_str("section .text\n");

        for inst in self.program {
            self.assemble_inst(inst)?;
            self.inst_index += 1;
        }

        self.assembly_output.push_str("section .data\n");

        for (id, (val, line)) in &self.data {
            let directive = self.get_val_size(*val, *line)?.get_directive();
            let val_str = match val {
                Value::Symbol(id) => self.resolve_sym(id).id.to_string(),
                Value::Bool(b) => (*b as u8).to_string(),
                Value::String(s) => s.to_string(),
                Value::UInt8(v) => v.to_string(),
                Value::UInt16(v) => v.to_string(),
                Value::UInt32(v) => v.to_string(),
                Value::UInt64(v) => v.to_string(),
                Value::UInt128(v) => v.to_string(),
                Value::Int8(v) => v.to_string(),
                Value::Int16(v) => v.to_string(),
                Value::Int32(v) => v.to_string(),
                Value::Int64(v) => v.to_string(),
                Value::Int128(v) => v.to_string(),
                Value::Float32(v) => v.to_string(),
                Value::Float64(v) => v.to_string(),
            };

            self.assembly_output
                .push_str(format!("\t{}: {} {}\n", id, directive, val_str).as_str());
        }

        self.assembly_output.push_str("section .rodata\n");

        for (id, (val, line)) in &self.rodata {
            let directive = self.get_val_size(*val, *line)?.get_directive();
            let val_str = match val {
                Value::Symbol(id) => self.resolve_sym(id).id.to_string(),
                Value::Bool(b) => (*b as u8).to_string(),
                Value::String(s) => s.to_string(),
                Value::UInt8(v) => v.to_string(),
                Value::UInt16(v) => v.to_string(),
                Value::UInt32(v) => v.to_string(),
                Value::UInt64(v) => v.to_string(),
                Value::UInt128(v) => v.to_string(),
                Value::Int8(v) => v.to_string(),
                Value::Int16(v) => v.to_string(),
                Value::Int32(v) => v.to_string(),
                Value::Int64(v) => v.to_string(),
                Value::Int128(v) => v.to_string(),
                Value::Float32(v) => v.to_string(),
                Value::Float64(v) => v.to_string(),
            };

            self.assembly_output
                .push_str(format!("\t{}: {} {}\n", id, directive, val_str).as_str());
        }

        Ok(self.assembly_output)
    }

    pub(super) fn resolve_sym(&self, id: &str) -> &Symbol<'a> {
        self.symbols
            .get(self.resolutions[self.inst_index].get(id).unwrap())
            .unwrap()
    }

    pub(super) fn get_val_size(&self, val: Value<'a>, line: usize) -> Result<Size, Error> {
        match val {
            Value::Symbol(id) => self.resolve_sym(id).ty.size(line, self.arena),
            Value::Bool(_) | Value::UInt8(_) | Value::Int8(_) => Ok(Size::B8),
            Value::UInt16(_) | Value::Int16(_) => Ok(Size::B2),
            Value::UInt32(_) | Value::Int32(_) | Value::Float32(_) => Ok(Size::B4),
            Value::UInt64(_) | Value::Int64(_) | Value::Float64(_) => Ok(Size::B8),
            Value::UInt128(_) | Value::Int128(_) | Value::String(_) => Ok(Size::B16),
        }
    }
}
