use std::path::Path;

use crate::{
    arena::Arena,
    error::{Error, loc_error},
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
pub(crate) enum Value<'a> {
    Symbol(&'a str),
    Bool(bool),
    String(&'a str),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    UInt128(u128),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Int128(i128),
    Float32(f32),
    Float64(f64),
}

impl<'a> Value<'a> {
    pub(crate) fn add_symbol_prefix(&mut self, arena: &'a Arena<'a>, prefix: &str) {
        if let Value::Symbol(s) = self {
            *s = arena.dyn_alloc(format!("{}.{}", prefix, s));
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Type<'a> {
    Bool,
    String,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    Float32,
    Float64,
    Ptr(&'a Type<'a>),
    Label,
    Undeclared,
}

impl<'a> Type<'a> {
    pub(crate) fn as_str(self, arena: &Arena<'a>) -> &'a str {
        match self {
            Type::Bool => "bool",
            Type::String => "string",
            Type::UInt8 => "uint8",
            Type::UInt16 => "uint16",
            Type::UInt32 => "uint32",
            Type::UInt64 => "uint64",
            Type::UInt128 => "uint128",
            Type::Int8 => "int8",
            Type::Int16 => "int16",
            Type::Int32 => "int32",
            Type::Int64 => "int64",
            Type::Int128 => "int128",
            Type::Float32 => "float32",
            Type::Float64 => "float64",
            Type::Ptr(ty) => arena.dyn_alloc(format!("*{}", ty.as_str(arena))),
            Type::Label => "label",
            Type::Undeclared => "undeclared",
        }
    }

    pub(crate) fn is_assignable_to(self, target: Type<'a>) -> bool {
        match (self, target) {
            (source, target) if source == target => true,
            (
                Type::Bool,
                Type::UInt8 | Type::UInt16 | Type::UInt32 | Type::UInt64 | Type::UInt128,
            ) => true,
            (
                Type::UInt8,
                Type::UInt16
                | Type::UInt32
                | Type::UInt64
                | Type::UInt128
                | Type::Int16
                | Type::Int32
                | Type::Int64
                | Type::Int128,
            ) => true,
            (
                Type::UInt16,
                Type::UInt32
                | Type::UInt64
                | Type::UInt128
                | Type::Int32
                | Type::Int64
                | Type::Int128,
            ) => true,
            (Type::UInt32, Type::UInt64 | Type::UInt128 | Type::Int64 | Type::Int128) => true,
            (Type::UInt64, Type::UInt128 | Type::Int128 | Type::Ptr(_)) => true,
            (Type::Int8, Type::Int16 | Type::Int32 | Type::Int64 | Type::Int128) => true,
            (Type::Int16, Type::Int32 | Type::Int64 | Type::Int128) => true,
            (Type::Int32, Type::Int64 | Type::Int128) => true,
            (Type::Int64, Type::Int128) => true,
            (Type::Float32, Type::Float64) => true,
            (Type::Ptr(_), Type::UInt64 | Type::UInt128) => true,
            (Type::Ptr(s_inner), Type::Ptr(t_inner)) => s_inner == t_inner,
            _ => false,
        }
    }

    pub(crate) fn join(
        self,
        other: Type<'a>,
        arena: &Arena<'a>,
        line: usize,
    ) -> Result<Type<'a>, Error> {
        if self.is_assignable_to(other) {
            return Ok(other);
        }

        if other.is_assignable_to(self) {
            return Ok(self);
        }

        match (self, other) {
            (Type::UInt8, Type::Int8) | (Type::Int8, Type::UInt8) => Ok(Type::Int16),
            (Type::UInt16, Type::Int16) | (Type::Int16, Type::UInt16) => Ok(Type::Int32),
            (Type::UInt32, Type::Int32) | (Type::Int32, Type::UInt32) => Ok(Type::Int64),
            (Type::UInt64, Type::Int64) | (Type::Int64, Type::UInt64) => Ok(Type::Int128),
            _ => loc_error(
                line,
                format!(
                    "No common type for types '{}' and '{}'",
                    self.as_str(arena),
                    other.as_str(arena)
                )
                .as_str(),
            ),
        }
    }
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
