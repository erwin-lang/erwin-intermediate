use crate::structure::types::Type;

#[derive(Debug, Clone)]
pub(crate) struct Symbol<'a> {
    pub(crate) id: &'a str,
    pub(crate) ty: Type<'a>,
    pub(crate) is_fn: bool,
    pub(crate) is_visible: bool,
    pub(crate) is_reassignable: bool,
    pub(crate) is_global: bool,
    pub(crate) mangled_params: Vec<&'a str>,
    pub(crate) stack_offset: i32,
}
