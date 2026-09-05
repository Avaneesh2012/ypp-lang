/// Token types for the Y++ lexer.

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Keywords
    Import,
    ImportAll,
    Print,
    Num,
    StringBlock,
    Exception,
    Concat,
    Public,
    Class,
    Func,
    New,
    KwGlobal,
    While,
    If,
    Else,
    Not,
    Bang,
    True,
    False,

    // Type keywords
    KwSmallint,
    KwInteger,
    KwDouble,
    KwSlong,
    KwSchar,
    KwBool,

    // Cast calls
    CastInt,
    CastDouble,
    CastString,
    CastStringint,

    // Literals
    LitSmallint,
    LitInteger,
    LitDouble,
    LitString,
    LitChar,

    // Identifiers and numbers
    Ident,
    NumberLabel,

    // Operators
    Equals,
    EqEq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    Star,
    Plus,
    PlusPlus,
    Minus,
    MinusMinus,
    Slash,

    // Punctuation
    Colon,
    DoubleColon,
    Semicolon,
    Comma,
    Dot,
    LBrace,
    RBrace,
    LParen,
    RParen,
    LBracket,
    RBracket,

    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, value: impl Into<String>, line: usize) -> Self {
        Token {
            token_type,
            value: value.into(),
            line,
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Token({:?}, {}, line={})", self.token_type, self.value, self.line)
    }
}
