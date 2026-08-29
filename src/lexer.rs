use crate::token::{Token, TokenType};

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    prev_token_type: Option<TokenType>,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        let stripped = Self::strip_comments(source);
        Self {
            source: stripped.chars().collect(),
            pos: 0,
            line: 1,
            prev_token_type: None,
        }
    }

    fn strip_comments(src: &str) -> String {
        let mut out = String::with_capacity(src.len());
        let mut chars = src.chars().peekable();
        let mut in_comment = false;

        while let Some(c) = chars.next() {
            if !in_comment {
                if c == '\\' && chars.peek() == Some(&'\\') {
                    chars.next(); // consume second \
                    in_comment = true;
                    out.push(' ');
                    out.push(' ');
                } else {
                    out.push(c);
                }
            } else {
                if c == '\\' && chars.peek() == Some(&'\\') {
                    chars.next(); // consume second \
                    in_comment = false;
                    out.push(' ');
                    out.push(' ');
                } else if c == '\n' {
                    out.push('\n');
                } else {
                    out.push(' ');
                }
            }
        }
        out
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace();
            if self.pos >= self.source.len() {
                tokens.push(Token {
                    token_type: TokenType::Eof,
                    line: self.line,
                });
                break;
            }
            let start_line = self.line;
            let tok = self.next_token(start_line);
            self.prev_token_type = Some(tok.token_type.clone());
            tokens.push(tok);
        }
        tokens
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c == ' ' || c == '\t' || c == '\r' {
                self.advance();
            } else if c == '\n' {
                self.line += 1;
                self.advance();
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    fn peek_char_at(&self, offset: usize) -> Option<char> {
        self.source.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.pos];
        self.pos += 1;
        c
    }

    fn next_token(&mut self, start_line: usize) -> Token {
        let c = self.peek_char().unwrap();

        match c {
            '=' => {
                self.advance();
                Token { token_type: TokenType::Equals, line: start_line }
            }
            '*' => {
                self.advance();
                Token { token_type: TokenType::Star, line: start_line }
            }
            '+' => {
                self.advance();
                Token { token_type: TokenType::Plus, line: start_line }
            }
            '-' => {
                let next_is_digit = self.peek_char_at(1).map(|ch| ch.is_ascii_digit()).unwrap_or(false);
                
                let can_be_minus = match &self.prev_token_type {
                    Some(TokenType::Ident(_)) |
                    Some(TokenType::NumberLabel(_)) |
                    Some(TokenType::LitSmallint(_)) |
                    Some(TokenType::LitInteger(_)) |
                    Some(TokenType::LitDouble(_)) |
                    Some(TokenType::LitString(_)) |
                    Some(TokenType::RParen) |
                    Some(TokenType::RBrace) => true,
                    _ => false,
                };

                if next_is_digit && !can_be_minus {
                    self.read_number(start_line)
                } else {
                    self.advance();
                    Token { token_type: TokenType::Minus, line: start_line }
                }
            }
            '/' => {
                self.advance();
                Token { token_type: TokenType::Slash, line: start_line }
            }
            ':' => {
                self.advance();
                if self.peek_char() == Some(':') {
                    self.advance();
                    Token { token_type: TokenType::DoubleColon, line: start_line }
                } else {
                    Token { token_type: TokenType::Colon, line: start_line }
                }
            }
            ';' => {
                self.advance();
                Token { token_type: TokenType::Semicolon, line: start_line }
            }
            ',' => {
                self.advance();
                Token { token_type: TokenType::Comma, line: start_line }
            }
            '.' => {
                self.advance();
                Token { token_type: TokenType::Dot, line: start_line }
            }
            '{' => {
                self.advance();
                Token { token_type: TokenType::LBrace, line: start_line }
            }
            '}' => {
                self.advance();
                Token { token_type: TokenType::RBrace, line: start_line }
            }
            '(' => {
                self.advance();
                Token { token_type: TokenType::LParen, line: start_line }
            }
            ')' => {
                self.advance();
                Token { token_type: TokenType::RParen, line: start_line }
            }
            '!' => {
                self.advance();
                Token { token_type: TokenType::Bang, line: start_line }
            }
            '"' => self.read_string(start_line),
            ch if ch.is_ascii_digit() => self.read_number(start_line),
            ch if ch.is_ascii_alphabetic() || ch == '_' => self.read_ident(start_line),
            _ => {
                self.advance();
                self.skip_whitespace();
                if self.pos >= self.source.len() {
                    Token { token_type: TokenType::Eof, line: self.line }
                } else {
                    self.next_token(self.line)
                }
            }
        }
    }

    fn read_string(&mut self, start_line: usize) -> Token {
        self.advance(); // consume opening "
        let mut val = String::new();
        while let Some(c) = self.peek_char() {
            if c == '"' {
                break;
            }
            if c == '\n' {
                self.line += 1;
            }
            val.push(self.advance());
        }
        if self.peek_char() == Some('"') {
            self.advance(); // consume closing "
        }
        Token { token_type: TokenType::LitString(val), line: start_line }
    }

    fn read_number(&mut self, start_line: usize) -> Token {
        let mut val = String::new();
        if self.peek_char() == Some('-') {
            val.push(self.advance());
        }
        
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                val.push(self.advance());
            } else {
                break;
            }
        }
        
        if self.peek_char() == Some('.') {
            val.push(self.advance());
            while let Some(c) = self.peek_char() {
                if c.is_ascii_digit() {
                    val.push(self.advance());
                } else {
                    break;
                }
            }
        }
        
        let mut suffix = String::new();
        while let Some(c) = self.peek_char() {
            if c.is_ascii_alphabetic() {
                suffix.push(self.advance());
            } else {
                break;
            }
        }

        let token_type = match suffix.as_str() {
            "si" => TokenType::LitSmallint(val),
            "i" => TokenType::LitInteger(val),
            "d" => TokenType::LitDouble(val),
            _ => TokenType::NumberLabel(val),
        };

        Token { token_type, line: start_line }
    }

    fn read_ident(&mut self, start_line: usize) -> Token {
        let mut val = String::new();
        while let Some(c) = self.peek_char() {
            if c.is_ascii_alphanumeric() || c == '_' {
                val.push(self.advance());
            } else {
                break;
            }
        }

        let is_cast = val == "int" || val == "double" || val == "string" || val == "stringint";
        if is_cast && self.peek_char() == Some('(') && self.peek_char_at(1) == Some(')') {
            self.advance(); // consume (
            self.advance(); // consume )
            let token_type = match val.as_str() {
                "int" => TokenType::CastInt,
                "double" => TokenType::CastDouble,
                "string" => TokenType::CastString,
                "stringint" => TokenType::CastStringint,
                _ => unreachable!(),
            };
            return Token { token_type, line: start_line };
        }

        let token_type = match val.as_str() {
            "Import" => TokenType::Import,
            "PRINT" => TokenType::Print,
            "NUM" => TokenType::Num,
            "STRING" => TokenType::StringBlock,
            "EXCEPTION" => TokenType::Exception,
            "CONCAT" => TokenType::Concat,
            "Public" => TokenType::Public,
            "class" => TokenType::Class,
            "func" => TokenType::Func,
            "NEW" | "new" => TokenType::New,
            "global" => TokenType::KwGlobal,
            "while" => TokenType::While,
            "NOT" => TokenType::Not,
            "smallint" => TokenType::KwSmallint,
            "integer" => TokenType::KwInteger,
            "double" => TokenType::KwDouble,
            "slong" => TokenType::KwSlong,
            "schar" => TokenType::KwSchar,
            _ => TokenType::Ident(val),
        };

        Token { token_type, line: start_line }
    }
}
