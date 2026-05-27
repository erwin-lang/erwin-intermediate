use std::path::Path;

use crate::{
    arena::Arena,
    structure::{types::Type, value::Value},
};

#[derive(Debug, Clone)]
pub(crate) struct Instruction<'a> {
    pub(crate) kind: InstructionKind<'a>,
    pub(crate) line: usize,
}

impl<'a> Instruction<'a> {
    pub(crate) fn add_module_prefix(&mut self, arena: &'a Arena<'a>, prefix: &str) {
        match &mut self.kind {
            InstructionKind::Decl { id, value, .. } => {
                *id = arena.dyn_alloc(format!("{}.{}", prefix, id));
                value.add_symbol_prefix(arena, prefix);
            }
            InstructionKind::Assign { id: sym, value } => {
                *sym = arena.dyn_alloc(format!("{}.{}", prefix, sym));
                value.add_symbol_prefix(arena, prefix);
            }
            InstructionKind::Fn { id, params, .. } => {
                *id = arena.dyn_alloc(format!("{}.{}", prefix, id));

                for (_, p_id) in params {
                    *p_id = arena.dyn_alloc(format!("{}.{}", prefix, p_id));
                }
            }
            InstructionKind::Ret { ret } => {
                ret.add_symbol_prefix(arena, prefix);
            }
            InstructionKind::Call { op, id, args } => {
                *op = arena.dyn_alloc(format!("{}.{}", prefix, op));
                *id = arena.dyn_alloc(format!("{}.{}", prefix, id));

                for arg in args {
                    arg.add_symbol_prefix(arena, prefix);
                }
            }
            InstructionKind::Label { id } => {
                *id = arena.dyn_alloc(format!("{}.{}", prefix, id));
            }
            InstructionKind::Jump { cond, label } => {
                cond.add_symbol_prefix(arena, prefix);
                *label = arena.dyn_alloc(format!("{}.{}", prefix, label));
            }
            InstructionKind::UnaryOp { id: sym, val, .. } => {
                *sym = arena.dyn_alloc(format!("{}.{}", prefix, sym));
                val.add_symbol_prefix(arena, prefix);
            }
            InstructionKind::BinaryOp { id: sym, a, b, .. } => {
                *sym = arena.dyn_alloc(format!("{}.{}", prefix, sym));
                a.add_symbol_prefix(arena, prefix);
                b.add_symbol_prefix(arena, prefix);
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum InstructionKind<'a> {
    Import {
        alias: &'a str,
        path: &'a Path,
    },
    Decl {
        ty: Type<'a>,
        id: &'a str,
        value: Value<'a>,
    }, // Declares a new symbol, type is the opcode
    Assign {
        id: &'a str,
        value: Value<'a>,
    }, // Changes value of a symbol
    Fn {
        ty: Type<'a>,
        id: &'a str,
        params: Vec<(Type<'a>, &'a str)>,
    }, // Function start, all instructions after are skipped until we hit a matching return
    Ret {
        ret: Value<'a>,
    }, // Function end, keeps returned value and frees all others
    Call {
        op: &'a str,
        id: &'a str,
        args: Vec<Value<'a>>,
    }, // Func call, acts like a user-defined instruction where the called func is the opcode
    Label {
        id: &'a str,
    },
    Jump {
        cond: Value<'a>,
        label: &'a str,
    },
    UnaryOp {
        op: UnaryOp,
        id: &'a str,
        val: Value<'a>,
    },
    BinaryOp {
        op: BinaryOp,
        id: &'a str,
        a: Value<'a>,
        b: Value<'a>,
    },
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum UnaryOp {
    Neg,
    Not,
    Ref,
    Deref,
    Lsh,
    Rsh,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum BinaryOp {
    Add,
    Sub,
    Mult,
    Div,
    And,
    Or,
    Xor,
}
