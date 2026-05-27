#[derive(Debug, Clone, Copy)]
pub(crate) enum Size {
    B1,
    B2,
    B4,
    B8,
    B16,
}

impl<'a> Size {
    pub(crate) fn size_str(self) -> &'a str {
        match self {
            Size::B1 => "byte",
            Size::B2 => "word",
            Size::B4 => "dword",
            Size::B8 => "qword",
            Size::B16 => "oword",
        }
    }

    pub(crate) fn get_directive(self) -> &'a str {
        match self {
            Size::B1 => "db",
            Size::B2 => "dw",
            Size::B4 => "dd",
            Size::B8 => "dq",
            Size::B16 => "do",
        }
    }
}
