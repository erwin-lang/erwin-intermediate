use crate::structure::ast::Type;

#[derive(Debug, Clone)]
pub(crate) struct Symbol<'a> {
    pub(crate) id: &'a str,
    pub(crate) ty: Type<'a>,
    pub(crate) is_fn: bool,
    pub(crate) is_visible: bool,
    pub(crate) mangled_params: Vec<&'a str>,
}
