pub(super) mod inst;
pub(super) mod inst_initial;

use std::collections::{HashMap, HashSet};

use crate::{
    arena::Arena,
    error::{Error, loc_error},
    structure::{
        ast::{Instruction, Type, Value},
        symbols::Symbol,
    },
};

pub(crate) struct Checker<'a> {
    arena: &'a Arena<'a>,
    program: &'a Vec<Instruction<'a>>,

    symbols: HashMap<&'a str, Symbol<'a>>,
    working_funcs: Vec<&'a str>,
    passed_labels: HashSet<&'a str>,
    forward_jump: Option<&'a str>,
    jumped_symbols: HashSet<&'a str>,
}

impl<'a> Checker<'a> {
    pub(crate) fn new(arena: &'a Arena<'a>, program: &'a Vec<Instruction<'a>>) -> Self {
        Self {
            arena,
            program,

            symbols: HashMap::new(),
            working_funcs: Vec::new(),
            passed_labels: HashSet::new(),
            forward_jump: None,
            jumped_symbols: HashSet::new(),
        }
    }

    pub(crate) fn check(mut self) -> Result<(), Error> {
        for inst in self.program {
            self.check_inst_initial(inst)?;
        }

        for inst in self.program {
            self.check_inst(inst)?;
        }

        Ok(())
    }

    pub(super) fn get_value_ty(&self, val: Value<'a>, line: usize) -> Result<Type<'a>, Error> {
        match val {
            Value::Symbol(id) => {
                let resolved_id = self.resolve_id(id);
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
            let mangled = self.arena.dyn_alloc(format!("{}#{}", prefix, id)).as_str();

            if self.symbols.contains_key(mangled) {
                return mangled;
            }
        }

        id
    }
}
