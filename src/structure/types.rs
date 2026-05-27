use crate::{
    arena::Arena,
    error::{Error, loc_error},
    structure::sizes::Size,
};

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

    pub(crate) fn size(self, line: usize, arena: &Arena<'a>) -> Result<Size, Error> {
        match self {
            Type::Bool | Type::UInt8 | Type::Int8 => Ok(Size::B1),
            Type::UInt16 | Type::Int16 => Ok(Size::B2),
            Type::UInt32 | Type::Int32 | Type::Float32 => Ok(Size::B4),
            Type::UInt64 | Type::Int64 | Type::Float64 | Type::Ptr(_) => Ok(Size::B8),
            Type::UInt128 | Type::Int128 | Type::String => Ok(Size::B16),
            _ => loc_error(
                line,
                format!("Cannot get size of type '{}'", self.as_str(arena)).as_str(),
            ),
        }
    }
}
