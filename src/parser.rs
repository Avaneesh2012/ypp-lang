use crate::token::{Token, TokenType};
use crate::ast::*;
use crate::error::YppError;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<AstNode, YppError> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        Ok(AstNode::Program { statements })
    }

    fn peek(&self) -> &Token {
        if self.pos < self.tokens.len() {
            &self.tokens[self.pos]
        } else {
            &self.tokens.last().unwrap() // Should be EOF
        }
    }

    fn peek_at(&self, offset: usize) -> &Token {
        if self.pos + offset < self.tokens.len() {
            &self.tokens[self.pos + offset]
        } else {
            &self.tokens.last().unwrap() // Should be EOF
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.pos += 1;
        }
        self.tokens[self.pos - 1].clone()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().token_type == *token_type
    }

    fn match_token(&mut self, token_type: TokenType) -> bool {
        if self.check(&token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, token_type: TokenType, expected: &str) -> Result<Token, YppError> {
        if self.check(&token_type) {
            Ok(self.advance())
        } else {
            let p = self.peek();
            Err(YppError::expected_token(p.line, expected, &format!("{:?}", p.token_type), &p.value))
        }
    }

    fn optional_semicolon(&mut self) {
        self.match_token(TokenType::Semicolon);
    }

    fn parse_statement(&mut self) -> Result<AstNode, YppError> {
        let p = self.peek().token_type.clone();
        match p {
            TokenType::Import => self.parse_import(),
            TokenType::Print => self.parse_print(),
            TokenType::Num => self.parse_num_block(),
            TokenType::StringBlock => self.parse_string_block(),
            TokenType::Exception => self.parse_exception_concat(),
            TokenType::Func => self.parse_func_decl(),
            TokenType::New => self.parse_new_alias(),
            TokenType::KwGlobal => {
                self.advance();
                let inner = self.parse_statement()?;
                Ok(AstNode::GlobalBlock { inner: Box::new(inner) })
            }
            TokenType::While => self.parse_while(),
            TokenType::KwSmallint | TokenType::KwInteger | TokenType::KwDouble => self.parse_bare_var_decl(),
            TokenType::KwSlong | TokenType::KwSchar => {
                if self.peek_at(1).token_type == TokenType::Dot {
                    self.parse_param_bind()
                } else {
                    self.parse_bare_string_var_decl()
                }
            }
            TokenType::Ident => {
                let p1 = self.peek_at(1).token_type.clone();
                let p2 = self.peek_at(2).token_type.clone();

                if p1 == TokenType::LBrace {
                    self.parse_named_block()
                } else if p1 == TokenType::Dot {
                    let m = &self.peek_at(2).value;
                    if m == "input" || m == "next" || m == "break" {
                        self.parse_param_action()
                    } else {
                        self.parse_expr_statement()
                    }
                } else if p1 == TokenType::LParen && p2 == TokenType::RParen {
                    let p3 = self.peek_at(3).token_type.clone();
                    if p3 == TokenType::Semicolon || p3 == TokenType::RBrace || p3 == TokenType::Eof {
                        self.parse_func_call()
                    } else {
                        self.parse_expr_statement()
                    }
                } else if p1 == TokenType::Equals && p2 == TokenType::New {
                    self.parse_alias_assignment()
                } else if p1 == TokenType::Equals {
                    self.parse_assignment()
                } else {
                    self.parse_expr_statement()
                }
            }
            _ => self.parse_expr_statement(),
        }
    }

    fn parse_import(&mut self) -> Result<AstNode, YppError> {
        self.advance(); // consume Import
        let pkg = self.expect(TokenType::Ident, "package name")?.value;
        let mut import_all = false;
        let mut class_name = None;

        if self.match_token(TokenType::ImportAll) || self.match_token(TokenType::Star) {
            import_all = true;
        } else if self.match_token(TokenType::LBrace) {
            class_name = Some(self.expect(TokenType::Ident, "class name")?.value);
            self.expect(TokenType::RBrace, "'}'")?;
        }
        self.optional_semicolon();
        Ok(AstNode::Import { pkg, import_all, class_name })
    }

    fn parse_print(&mut self) -> Result<AstNode, YppError> {
        self.advance(); // consume PRINT
        self.expect(TokenType::Colon, "':' after PRINT")?;

        let cast = if self.match_token(TokenType::CastInt) {
            Cast::Int
        } else if self.match_token(TokenType::CastDouble) {
            Cast::Double
        } else if self.match_token(TokenType::CastString) {
            Cast::StringCast
        } else if self.match_token(TokenType::CastStringint) {
            Cast::StringInt
        } else {
            Cast::None
        };

        let expr = self.parse_expr()?;
        self.optional_semicolon();
        Ok(AstNode::Print { cast, expr })
    }

    fn parse_num_block(&mut self) -> Result<AstNode, YppError> {
        self.advance(); // consume NUM
        let line = self.peek().line;
        let label = self.expect(TokenType::NumberLabel, "block label")?.value;
        self.expect(TokenType::LBrace, "'{'")?;
        let mut statements = Vec::new();
        while !self.check(&TokenType::RBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        self.expect(TokenType::RBrace, "'}'")?;
        Ok(AstNode::NumBlock { label, statements, line })
    }

    fn parse_string_block(&mut self) -> Result<AstNode, YppError> {
        self.advance(); // consume STRING
        let line = self.peek().line;
        
        let label_tok = self.advance();
        if self.match_token(TokenType::LBrace) {
            let label = label_tok.value;
            let mut statements = Vec::new();
            while !self.check(&TokenType::RBrace) && !self.is_at_end() {
                statements.push(self.parse_statement()?);
            }
            self.expect(TokenType::RBrace, "'}'")?;
            Ok(AstNode::StringBlock { label, statements, line })
        } else {
            // Bare variable: STRING line; or STRING line = "val";
            let var_name = label_tok.value;
            let value = if self.match_token(TokenType::Equals) {
                self.parse_expr()?
            } else {
                Expr::StringLiteral { value: "".to_string() }
            };
            self.optional_semicolon();
            let decl = AstNode::StringVarDecl { type_name: "slong".to_string(), var_name, value, line };
            Ok(AstNode::StringBlock { label: "1".to_string(), statements: vec![decl], line })
        }
    }

    fn parse_exception_concat(&mut self) -> Result<AstNode, YppError> {
        self.advance(); // consume EXCEPTION
        self.expect(TokenType::Concat, "CONCAT")?;
        if self.match_token(TokenType::LParen) {
            self.expect(TokenType::RParen, "')'")?;
        }
        self.expect(TokenType::LBrace, "'{'")?;
        let mut statements = Vec::new();
        while !self.check(&TokenType::RBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        self.expect(TokenType::RBrace, "'}'")?;
        Ok(AstNode::ExceptionConcat { statements })
    }

    fn parse_func_decl(&mut self) -> Result<AstNode, YppError> {
        let line = self.peek().line;
        self.advance(); // consume func
        let name = self.expect(TokenType::Ident, "function name")?.value;
        let mut params = Vec::new();
        
        if self.match_token(TokenType::LParen) {
            if !self.check(&TokenType::RParen) {
                loop {
                    let type_tok = self.advance();
                    let type_name = type_tok.value;
                    let p_name = self.expect(TokenType::Ident, "parameter name")?.value;
                    params.push(Param { type_name, name: p_name });
                    if !self.match_token(TokenType::Comma) {
                        break;
                    }
                }
            }
            self.expect(TokenType::RParen, "')'")?;
        }

        self.expect(TokenType::LBrace, "'{'")?;
        let mut body = Vec::new();
        while !self.check(&TokenType::RBrace) && !self.is_at_end() {
            body.push(self.parse_statement()?);
        }
        self.expect(TokenType::RBrace, "'}'")?;
        Ok(AstNode::FuncDecl { name, params, body, line })
    }

    fn parse_new_alias(&mut self) -> Result<AstNode, YppError> {
        let line = self.peek().line;
        self.advance(); // consume NEW
        let func_name = self.expect(TokenType::Ident, "function name")?.value;
        
        let mut block_type = None;
        let mut block_label = None;

        if self.match_token(TokenType::Dot) {
            self.expect(TokenType::LParen, "'('")?;
            let bt = self.advance();
            block_type = Some(bt.value);
            let bl = self.advance();
            block_label = Some(bl.value);
            self.expect(TokenType::RParen, "')'")?;
        }

        self.expect(TokenType::Equals, "'='")?;
        let alias_name = self.expect(TokenType::Ident, "alias name")?.value;
        self.optional_semicolon();
        Ok(AstNode::NewAlias { func_name, block_type, block_label, alias_name, line })
    }

    fn parse_while(&mut self) -> Result<AstNode, YppError> {
        let line = self.peek().line;
        self.advance(); // consume while
        self.expect(TokenType::LParen, "'('")?;
        let condition = self.parse_expr()?;
        self.expect(TokenType::RParen, "')'")?;
        
        self.expect(TokenType::LBrace, "'{'")?;
        let mut body = Vec::new();
        while !self.check(&TokenType::RBrace) && !self.is_at_end() {
            body.push(self.parse_statement()?);
        }
        self.expect(TokenType::RBrace, "'}'")?;
        Ok(AstNode::While { condition, body, line })
    }

    fn parse_bare_var_decl(&mut self) -> Result<AstNode, YppError> {
        let tok = self.advance(); // consume type
        let line = tok.line;
        let type_name = tok.value.clone();
        
        let var_name = if self.check(&TokenType::Ident) {
            self.advance().value
        } else {
            type_name.clone()
        };

        self.expect(TokenType::Equals, "'='")?;
        let value = self.parse_expr()?;
        self.match_token(TokenType::Comma);
        self.optional_semicolon();
        
        Ok(AstNode::VarDecl { type_name, var_name, value, line })
    }

    fn parse_bare_string_var_decl(&mut self) -> Result<AstNode, YppError> {
        let tok = self.advance();
        let line = tok.line;
        let type_name = tok.value.clone();
        let var_name = self.expect(TokenType::Ident, "variable name")?.value;
        let value = if self.match_token(TokenType::Equals) {
            self.parse_expr()?
        } else {
            Expr::StringLiteral { value: "".to_string() }
        };
        self.match_token(TokenType::Comma);
        self.optional_semicolon();
        Ok(AstNode::StringVarDecl { type_name, var_name, value, line })
    }

    fn parse_param_bind(&mut self) -> Result<AstNode, YppError> {
        let tok = self.advance();
        let line = tok.line;
        let type_name = tok.value;
        self.expect(TokenType::Dot, "'.'")?;
        let param_name = self.expect(TokenType::Ident, "param name")?.value;
        self.expect(TokenType::Equals, "'='")?;
        let param_ref = self.expect(TokenType::Ident, "param ref")?.value;
        self.optional_semicolon();
        Ok(AstNode::ParamBind { type_name, param_name, param_ref, line })
    }

    fn parse_named_block(&mut self) -> Result<AstNode, YppError> {
        let label_tok = self.advance();
        let line = label_tok.line;
        let label = label_tok.value;
        self.expect(TokenType::LBrace, "'{'")?;
        let mut statements = Vec::new();
        while !self.check(&TokenType::RBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        self.expect(TokenType::RBrace, "'}'")?;
        Ok(AstNode::NumBlock { label, statements, line })
    }

    fn parse_param_action(&mut self) -> Result<AstNode, YppError> {
        let param_name = self.advance().value;
        let line = self.peek().line;
        self.expect(TokenType::Dot, "'.'")?;
        let action = self.advance().value;

        if action == "input" {
            self.expect(TokenType::LParen, "'('")?;
            let prompt = self.parse_expr()?;
            self.expect(TokenType::RParen, "')'")?;
            if !self.match_token(TokenType::Semicolon) {
                return Err(YppError::missing_semicolon_input(line, &param_name));
            }
            Ok(AstNode::ParamInput { param_name, prompt, line })
        } else if action == "next" {
            self.expect(TokenType::LParen, "'('")?;
            self.expect(TokenType::RParen, "')'")?;
            if !self.match_token(TokenType::Semicolon) {
                return Err(YppError::missing_semicolon_next(line, &param_name));
            }
            Ok(AstNode::ParamNext { param_name })
        } else if action == "break" {
            if !self.match_token(TokenType::Semicolon) {
                return Err(YppError::missing_semicolon_break(line, &param_name));
            }
            Ok(AstNode::ParamBreak { param_name })
        } else {
            Err(YppError::unexpected_token(line, &action))
        }
    }

    fn parse_func_call(&mut self) -> Result<AstNode, YppError> {
        let tok = self.advance();
        let line = tok.line;
        let alias_name = tok.value;
        self.expect(TokenType::LParen, "'('")?;
        self.expect(TokenType::RParen, "')'")?;
        self.optional_semicolon();
        Ok(AstNode::FuncCall { alias_name, line })
    }

    fn parse_alias_assignment(&mut self) -> Result<AstNode, YppError> {
        let alias_name = self.advance().value;
        let line = self.peek().line;
        self.expect(TokenType::Equals, "'='")?;
        self.expect(TokenType::New, "new")?;
        
        let mut block_type = None;
        let mut block_label = None;

        if self.match_token(TokenType::LParen) {
            let bt = self.advance();
            block_type = Some(bt.value);
            let bl = self.advance();
            block_label = Some(bl.value);
            self.expect(TokenType::RParen, "')'")?;
            self.expect(TokenType::Dot, "'.'")?;
        }

        let func_name = self.expect(TokenType::Ident, "function name")?.value;
        
        if self.match_token(TokenType::LParen) {
            while !self.check(&TokenType::RParen) && !self.is_at_end() {
                self.advance();
            }
            self.expect(TokenType::RParen, "')'")?;
        }
        self.optional_semicolon();
        Ok(AstNode::NewAlias { func_name, block_type, block_label, alias_name, line })
    }

    fn parse_assignment(&mut self) -> Result<AstNode, YppError> {
        let name = self.advance().value;
        self.expect(TokenType::Equals, "'='")?;
        let value = self.parse_expr()?;
        self.optional_semicolon();
        Ok(AstNode::Assign { name, value })
    }

    fn parse_expr_statement(&mut self) -> Result<AstNode, YppError> {
        let expr = self.parse_expr()?;
        self.optional_semicolon();
        Ok(AstNode::ExprStatement { expr })
    }

    fn parse_expr(&mut self) -> Result<Expr, YppError> {
        let line = self.peek().line;
        
        if self.match_token(TokenType::Not) {
            self.expect(TokenType::Colon, "':' after NOT")?;
            let expr = self.parse_expr()?;
            return Ok(Expr::Not { expr: Box::new(expr), line });
        }
        if self.match_token(TokenType::Bang) {
            let expr = self.parse_expr()?;
            return Ok(Expr::Not { expr: Box::new(expr), line });
        }

        // Inline assign
        if self.peek().token_type == TokenType::Ident && self.peek_at(1).token_type == TokenType::Equals {
            let var_name = self.advance().value;
            self.advance(); // '='
            let expr = self.parse_expr()?;
            return Ok(Expr::InlineAssign { var_name, expr: Box::new(expr), line });
        }

        self.parse_additive()
    }

    fn parse_additive(&mut self) -> Result<Expr, YppError> {
        let mut expr = self.parse_multiplicative()?;
        while self.check(&TokenType::Plus) || self.check(&TokenType::Minus) {
            let op = self.advance().value.chars().next().unwrap();
            let right = self.parse_multiplicative()?;
            expr = Expr::BinaryExpr { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, YppError> {
        let mut expr = self.parse_primary()?;
        while self.check(&TokenType::Star) || self.check(&TokenType::Slash) {
            let op = self.advance().value.chars().next().unwrap();
            let right = self.parse_primary()?;
            expr = Expr::BinaryExpr { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, YppError> {
        let mut expr = self.parse_primary_base()?;
        while self.match_token(TokenType::Dot) {
            let line = self.peek().line;
            let method_name = self.expect(TokenType::Ident, "method name")?.value;
            self.expect(TokenType::LParen, "'('")?;
            let mut args = Vec::new();
            if !self.check(&TokenType::RParen) {
                loop {
                    args.push(self.parse_expr()?);
                    if !self.match_token(TokenType::Comma) {
                        break;
                    }
                }
            }
            self.expect(TokenType::RParen, "')'")?;
            expr = Expr::MethodCall { target: Box::new(expr), method_name, args, line };
        }
        Ok(expr)
    }

    fn parse_primary_base(&mut self) -> Result<Expr, YppError> {
        let tok = self.peek().clone();
        match tok.token_type {
            TokenType::New => self.parse_new_object_expr(),
            TokenType::LParen => {
                self.advance();
                let peek = self.peek().value.clone();
                if peek == "NUM" || peek == "STRING" {
                    self.advance();
                    let label = self.advance().value;
                    self.expect(TokenType::RParen, "')'")?;
                    let var = self.expect(TokenType::Ident, "var name")?.value;
                    if peek == "NUM" {
                        Ok(Expr::NumAccess { block_label: label, var_name: var })
                    } else {
                        Ok(Expr::StringAccess { block_label: label, var_name: var })
                    }
                } else if self.peek().token_type == TokenType::NumberLabel && self.peek_at(1).token_type == TokenType::RParen {
                    let label = self.advance().value;
                    self.advance(); // )
                    let var = self.expect(TokenType::Ident, "var name")?.value;
                    Ok(Expr::NumAccess { block_label: label, var_name: var })
                } else {
                    let expr = self.parse_expr()?;
                    self.expect(TokenType::RParen, "')'")?;
                    Ok(expr)
                }
            }
            TokenType::LitString => {
                self.advance();
                Ok(Expr::StringLiteral { value: tok.value })
            }
            TokenType::LitSmallint | TokenType::LitInteger | TokenType::LitDouble | TokenType::NumberLabel => {
                self.advance();
                let type_name = match tok.token_type {
                    TokenType::LitSmallint => "smallint",
                    TokenType::LitInteger => "integer",
                    TokenType::LitDouble => "double",
                    _ => "number",
                }.to_string();
                let value = tok.value.parse::<f64>().unwrap_or(0.0);
                Ok(Expr::NumberLiteral { value, type_name })
            }
            TokenType::Ident => {
                self.advance();
                let name = tok.value;
                if self.match_token(TokenType::LParen) {
                    let mut args = Vec::new();
                    if !self.check(&TokenType::RParen) {
                        loop {
                            args.push(self.parse_expr()?);
                            if !self.match_token(TokenType::Comma) {
                                break;
                            }
                        }
                    }
                    self.expect(TokenType::RParen, "')'")?;
                    Ok(Expr::NewObject { class_name: name, args, block_type: None, block_label: None, line: tok.line })
                } else {
                    Ok(Expr::Ident { name })
                }
            }
            _ => Err(YppError::unexpected_token(tok.line, &tok.value)),
        }
    }

    fn parse_new_object_expr(&mut self) -> Result<Expr, YppError> {
        let line = self.peek().line;
        self.advance(); // consume new
        let class_name = self.expect(TokenType::Ident, "class name")?.value;
        self.expect(TokenType::LParen, "'('")?;
        let mut args = Vec::new();
        if !self.check(&TokenType::RParen) {
            loop {
                args.push(self.parse_expr()?);
                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }
        self.expect(TokenType::RParen, "')'")?;
        Ok(Expr::NewObject { block_type: None, block_label: None, class_name, args, line })
    }
}
