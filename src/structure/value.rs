use crate::arena::Arena;

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
        if let Value::Symbol(id) = self
            && !id.contains("::")
        {
            *id = arena.dyn_alloc(format!("{}.{}", prefix, id));
        }
    }
}
