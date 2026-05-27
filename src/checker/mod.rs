pub(super) mod inst;
pub(super) mod inst_initial;

use std::collections::{HashMap, HashSet};

use crate::{
    arena::Arena,
    error::{Error, loc_error},
    structure::{ast::Instruction, symbol::Symbol, types::Type, value::Value},
};

pub(crate) struct Checker<'a> {
    arena: &'a Arena<'a>,
    program: &'a [Instruction<'a>],

    symbols: HashMap<&'a str, Symbol<'a>>,       // Symbol table
    working_funcs: Vec<&'a str>,                 // Function scope stack
    passed_labels: HashSet<&'a str>,             // Keeps track of passed labels during second pass
    forward_jump: Option<&'a str>, // Resolved ID of the target of the current forward jump (if some)
    jumped_symbols: HashSet<&'a str>, // Symbols that have been jumped in a forward jump
    resolutions: Vec<HashMap<&'a str, &'a str>>, // Vec elements are instructions, HashMap maps (ID -> mangled ID)
    inst_index: usize,                           // Current instruction index
    current_stack_offset: i32,                   // Current offset of the memory stack
}

impl<'a> Checker<'a> {
    pub(crate) fn new(arena: &'a Arena<'a>, program: &'a [Instruction<'a>]) -> Self {
        Self {
            arena,
            program,

            symbols: HashMap::new(),
            working_funcs: Vec::new(),
            passed_labels: HashSet::new(),
            forward_jump: None,
            jumped_symbols: HashSet::new(),
            resolutions: (0..program.len()).map(|_| HashMap::new()).collect(),
            inst_index: 0,
            current_stack_offset: 0,
        }
    }

    pub(crate) fn check(
        mut self,
    ) -> Result<(HashMap<&'a str, Symbol<'a>>, Vec<HashMap<&'a str, &'a str>>), Error> {
        for inst in self.program {
            self.check_inst_initial(inst)?;
        }

        for inst in self.program {
            self.check_inst(inst)?;
            self.inst_index += 1;
        }

        Ok((self.symbols, self.resolutions))
    }

    pub(super) fn get_value_ty(
        &mut self,
        val: Value<'a>,
        idx: usize,
        line: usize,
    ) -> Result<Type<'a>, Error> {
        match val {
            Value::Symbol(id) => {
                let resolved_id = self.resolve_id_logged(idx, id);
                let Some(sym) = self.symbols.get(resolved_id) else {
                    return loc_error(line, format!("Symbol '{}' never declared", id).as_str());
                };

                if matches!(sym.ty, Type::Undeclared) {
                    return loc_error(
                        line,
                        format!("Symbol '{}' used before it's declared", id).as_str(),
                    );
                }

                if !sym.is_visible {
                    return loc_error(line, format!("Symbol '{}' is not visible", id).as_str());
                }

                if sym.is_fn {
                    return loc_error(
                        line,
                        format!(
                            "Symbol '{}' is a function and cannot be used as a value",
                            id
                        )
                        .as_str(),
                    );
                }

                Ok(sym.ty)
            }
            Value::Bool(_) => Ok(Type::Bool),
            Value::String(_) => Ok(Type::String),
            Value::UInt8(_) => Ok(Type::UInt8),
            Value::UInt16(_) => Ok(Type::UInt16),
            Value::UInt32(_) => Ok(Type::UInt32),
            Value::UInt64(_) => Ok(Type::UInt64),
            Value::UInt128(_) => Ok(Type::UInt128),
            Value::Int8(_) => Ok(Type::Int8),
            Value::Int16(_) => Ok(Type::Int16),
            Value::Int32(_) => Ok(Type::Int32),
            Value::Int64(_) => Ok(Type::Int64),
            Value::Int128(_) => Ok(Type::Int128),
            Value::Float32(_) => Ok(Type::Float32),
            Value::Float64(_) => Ok(Type::Float64),
        }
    }

    pub(super) fn mangle_id(&self, id: &'a str) -> &'a str {
        if self.working_funcs.is_empty() {
            return id;
        }

        self.arena
            .dyn_alloc(format!("{}#{}", self.working_funcs.last().unwrap(), id))
    }

    pub(super) fn resolve_id(&self, id: &'a str) -> &'a str {
        for prefix in self.working_funcs.iter().rev() {
            let mangled = format!("{}#{}", prefix, id);

            if let Some((key, _)) = self.symbols.get_key_value(mangled.as_str()) {
                return key;
            }
        }

        id
    }

    pub(super) fn resolve_id_logged(&mut self, idx: usize, id: &'a str) -> &'a str {
        let resolved = self.resolve_id(id);
        self.resolutions[idx].insert(id, resolved);
        resolved
    }
}
