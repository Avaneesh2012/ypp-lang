use crate::token::{Token, TokenType};

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
}

impl Lexer {
    pub fn new(src: &str) -> Self {
        let stripped = Self::strip_comments(src);
        Self {
            source: stripped.chars().collect(),
            pos: 0,
            line: 1,
        }
    }

    fn strip_comments(src: &str) -> String {
        let mut result = String::new();
        let mut chars = src.chars().peekable();
        let mut in_comment = false;

        while let Some(c) = chars.next() {
            if in_comment {
                if c == '\\' {
                    if let Some(&'\\') = chars.peek() {
                        chars.next();
                        in_comment = false;
                        result.push(' ');
                        result.push(' ');
                    } else if c == '\n' {
                        result.push('\n');
                    } else {
                        result.push(' ');
                    }
                } else if c == '\n' {
                    result.push('\n');
                } else {
                    result.push(' ');
                }
            } else if c == '\\' {
                if let Some(&'\\') = chars.peek() {
                    chars.next();
                    in_comment = true;
                    result.push(' ');
                    result.push(' ');
                } else {
                    result.push(c);
                }
            } else {
                result.push(c);
            }
        }
        result
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while !self.is_at_end() {
            self.skip_whitespace();
            if self.is_at_end() {
                break;
            }
            tokens.push(self.next_token());
        }
        tokens.push(Token::new(TokenType::Eof, "", self.line));
        tokens
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.pos];
        self.pos += 1;
        c
    }

    fn peek(&self) -> Option<char> {
        if self.is_at_end() {
            None
        } else {
            Some(self.source[self.pos])
        }
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        if self.pos + offset >= self.source.len() {
            None
        } else {
            Some(self.source[self.pos + offset])
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                if c == '\n' {
                    self.line += 1;
                }
                self.advance();
            } else {
                break;
            }
        }
    }

    fn next_token(&mut self) -> Token {
        let c = match self.peek() {
            Some(c) => c,
            None => return Token::new(TokenType::Eof, "", self.line),
        };
        let start_line = self.line;

        match c {
            '=' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenType::EqEq, "==", start_line)
                } else {
                    Token::new(TokenType::Equals, "=", start_line)
                }
            }
            '!' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenType::NotEq, "!=", start_line)
                } else {
                    Token::new(TokenType::Bang, "!", start_line)
                }
            }
            '<' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenType::LtEq, "<=", start_line)
                } else {
                    Token::new(TokenType::Lt, "<", start_line)
                }
            }
            '>' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenType::GtEq, ">=", start_line)
                } else {
                    Token::new(TokenType::Gt, ">", start_line)
                }
            }
            '*' => {
                self.advance();
                Token::new(TokenType::Star, "*", start_line)
            }
            '+' => {
                self.advance();
                if self.peek() == Some('+') {
                    self.advance();
                    Token::new(TokenType::PlusPlus, "++", start_line)
                } else {
                    Token::new(TokenType::Plus, "+", start_line)
                }
            }
            '/' => {
                self.advance();
                Token::new(TokenType::Slash, "/", start_line)
            }
            ';' => {
                self.advance();
                Token::new(TokenType::Semicolon, ";", start_line)
            }
            ',' => {
                self.advance();
                Token::new(TokenType::Comma, ",", start_line)
            }
            '.' => {
                self.advance();
                Token::new(TokenType::Dot, ".", start_line)
            }
            '{' => {
                self.advance();
                Token::new(TokenType::LBrace, "{", start_line)
            }
            '}' => {
                self.advance();
                Token::new(TokenType::RBrace, "}", start_line)
            }
            '(' => {
                self.advance();
                Token::new(TokenType::LParen, "(", start_line)
            }
            ')' => {
                self.advance();
                Token::new(TokenType::RParen, ")", start_line)
            }
            '[' => {
                self.advance();
                Token::new(TokenType::LBracket, "[", start_line)
            }
            ']' => {
                self.advance();
                Token::new(TokenType::RBracket, "]", start_line)
            }
            ':' => {
                self.advance();
                if let Some(':') = self.peek() {
                    self.advance();
                    Token::new(TokenType::DoubleColon, "::", start_line)
                } else {
                    Token::new(TokenType::Colon, ":", start_line)
                }
            }
            '-' => {
                self.advance();
                if self.peek() == Some('-') {
                    self.advance();
                    Token::new(TokenType::MinusMinus, "--", start_line)
                } else if let Some(next_c) = self.peek() {
                    if next_c.is_ascii_digit() {
                        self.pos -= 1;
                        self.read_number(start_line)
                    } else {
                        Token::new(TokenType::Minus, "-", start_line)
                    }
                } else {
                    Token::new(TokenType::Minus, "-", start_line)
                }
            }
            '"' => self.read_string(start_line),
            '\'' => self.read_char(start_line),
            c if c.is_ascii_digit() => self.read_number(start_line),
            c if c.is_alphabetic() || c == '_' => self.read_ident(start_line),
            _ => {
                self.advance();
                if self.is_at_end() {
                    Token::new(TokenType::Eof, "", start_line)
                } else {
                    self.next_token()
                }
            }
        }
    }

    fn read_char(&mut self, start_line: usize) -> Token {
        self.advance(); // consume '
        let mut val = String::new();
        while let Some(c) = self.peek() {
            if c == '\'' {
                self.advance();
                break;
            }
            if c == '\n' {
                self.line += 1;
            }
            if c == '\\' {
                self.advance();
                if let Some(n) = self.peek() {
                    let escaped = match n {
                        'n' => '\n',
                        't' => '\t',
                        '\\' => '\\',
                        '\'' => '\'',
                        '"' => '"',
                        other => other,
                    };
                    self.advance();
                    val.push(escaped);
                }
            } else {
                val.push(self.advance());
            }
        }
        Token::new(TokenType::LitChar, val, start_line)
    }

    fn read_string(&mut self, start_line: usize) -> Token {
        self.advance(); // consume "
        let mut val = String::new();
        while let Some(c) = self.peek() {
            if c == '"' {
                self.advance();
                break;
            }
            if c == '\n' {
                self.line += 1;
            }
            val.push(self.advance());
        }
        Token::new(TokenType::LitString, val, start_line)
    }

    fn read_number(&mut self, start_line: usize) -> Token {
        let mut val = String::new();
        if let Some('-') = self.peek() {
            val.push(self.advance());
        }
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '.' {
                val.push(self.advance());
            } else {
                break;
            }
        }

        let mut suffix = String::new();
        if let Some(c) = self.peek() {
            if c.is_alphabetic() {
                suffix.push(self.advance());
                if suffix == "s" {
                    if let Some('i') = self.peek() {
                        suffix.push(self.advance());
                    }
                }
            }
        }

        let tt = match suffix.as_str() {
            "si" => TokenType::LitSmallint,
            "i" => TokenType::LitInteger,
            "d" => TokenType::LitDouble,
            _ => TokenType::NumberLabel,
        };

        Token::new(tt, val, start_line)
    }

    fn read_ident(&mut self, start_line: usize) -> Token {
        let mut val = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                val.push(self.advance());
            } else {
                break;
            }
        }

        if self.peek() == Some('(') && self.peek_at(1) == Some(')') {
            let mut matched = true;
            let tt = match val.as_str() {
                "int" => TokenType::CastInt,
                "double" => TokenType::CastDouble,
                "string" => TokenType::CastString,
                "stringint" => TokenType::CastStringint,
                _ => {
                    matched = false;
                    TokenType::Ident
                }
            };
            if matched {
                self.advance();
                self.advance();
                return Token::new(tt, val, start_line);
            }
        }

        let tt = match val.as_str() {
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
            "for" => TokenType::For,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "NOT" => TokenType::Not,
            "TRUE" | "true" => TokenType::True,
            "FALSE" | "false" => TokenType::False,
            "smallint" => TokenType::KwSmallint,
            "integer" => TokenType::KwInteger,
            "double" => TokenType::KwDouble,
            "slong" => TokenType::KwSlong,
            "schar" => TokenType::KwSchar,
            "bool" => TokenType::KwBool,
            "continue" => TokenType::Continue,
            _ => TokenType::Ident,
        };

        Token::new(tt, val, start_line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexes_gui_tokens() {
        let mut lexer = Lexer::new("if KEYBOARD.KEYHOLD('w') { y++; }");
        let types: Vec<TokenType> = lexer.tokenize().into_iter().map(|t| t.token_type).collect();
        assert!(types.contains(&TokenType::If));
        assert!(types.contains(&TokenType::PlusPlus));
        assert!(types.contains(&TokenType::LitChar));
    }
}
