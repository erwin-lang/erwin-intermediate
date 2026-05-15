use logos::Logos;

#[derive(Debug, Clone, Copy, Logos, PartialEq)]
#[logos(skip r"[ \t\f]+")]
pub(crate) enum Token {
    // Keywords
    #[token("asm")]
    Asm,

    #[token("import")]
    Import,

    #[token("decl")]
    Decl,

    #[token("assign")]
    Assign,

    #[token("fn")]
    Fn,

    #[token("ret")]
    Ret,

    #[token("label")]
    Label,

    #[token("jump")]
    Jump,

    #[token("neg")]
    Neg,

    #[token("not")]
    Not,

    #[token("ref")]
    Ref,

    #[token("deref")]
    Deref,

    #[token("lsh")]
    Lsh,

    #[token("rsh")]
    Rsh,

    #[token("add")]
    Add,

    #[token("sub")]
    Sub,

    #[token("mult")]
    Mult,

    #[token("div")]
    Div,

    #[token("and")]
    And,

    #[token("or")]
    Or,

    #[token("xor")]
    Xor,

    // Types
    #[token("bool")]
    Bool,

    #[token("string")]
    String,

    #[token("uint8")]
    UInt8,

    #[token("uint16")]
    UInt16,

    #[token("uint32")]
    UInt32,

    #[token("uint64")]
    UInt64,

    #[token("uint128")]
    UInt128,

    #[token("int8")]
    Int8,

    #[token("int16")]
    Int16,

    #[token("int32")]
    Int32,

    #[token("int64")]
    Int64,

    #[token("int128")]
    Int128,

    #[token("float32")]
    Float32,

    #[token("float64")]
    Float64,

    #[token("*")]
    Ptr,

    // Values
    #[regex(r"[_a-zA-Z][_a-zA-Z0-9]*(\.[_a-zA-Z0-9]+)*")]
    Identifier,

    #[token("true")]
    True,

    #[token("false")]
    False,

    #[regex(r#""[^"]*""#)]
    StringLiteral,

    #[regex(r"-?[0-9]+(\.[0-9]+)?")]
    Number,

    // Delimiters
    #[token("\n")]
    Newline,

    EOF,
}
