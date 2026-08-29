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

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        self.peek_at(0)
    }

    fn peek_at(&self, offset: usize) -> &Token {
        if self.pos + offset >= self.tokens.len() {
            &self.tokens.last().unwrap()
        } else {
            &self.tokens[self.pos + offset]
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.pos += 1;
        }
        self.tokens[self.pos - 1].clone()
    }

    fn expect(&mut self, token_type: TokenType) -> Result<Token, YppError> {
        if self.check(&token_type) {
            Ok(self.advance())
        } else {
            Err(YppError::ExpectedToken(token_type, self.peek().clone())) // Assumes YppError has this variant or similar, as we have to compile against the user's codebase
        }
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return token_type == &TokenType::Eof;
        }
        &self.peek().token_type == token_type
    }

    fn match_token(&mut self, token_type: &TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn optional_semicolon(&mut self) {
        self.match_token(&TokenType::Semicolon);
    }

    fn parse_statement(&mut self) -> Result<AstNode, YppError> {
        match self.peek().token_type {
            TokenType::Import => self.parse_import(),
            TokenType::Print => self.parse_print(),
            TokenType::Num => self.parse_num_block(),
            TokenType::StringBlock => self.parse_string_block(),
            TokenType::Exception => self.parse_exception_concat(),
            TokenType::Func => self.parse_func_decl(),
            TokenType::New => self.parse_new_alias(),
            TokenType::KwGlobal => {
                self.advance();
                let stmt = self.parse_statement()?;
                Ok(AstNode::GlobalBlock { statement: Box::new(stmt) })
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
                let p1 = &self.peek_at(1).token_type;
                let p2 = &self.peek_at(2).token_type;
                if p1 == &TokenType::LBrace {
                    self.parse_named_block()
                } else if p1 == &TokenType::Dot {
                    let method_name = if let TokenType::Ident(ref name) = p2 { name.as_str() } else { "" };
                    if method_name == "input" || method_name == "next" || method_name == "break" {
                        self.parse_param_action()
                    } else {
                        self.parse_expr_statement()
                    }
                } else if p1 == &TokenType::LParen && p2 == &TokenType::RParen {
                    let p3 = &self.peek_at(3).token_type;
                    if p3 == &TokenType::Semicolon || p3 == &TokenType::RBrace || p3 == &TokenType::Eof {
                        let node = self.parse_func_call()?;
                        self.optional_semicolon();
                        Ok(node)
                    } else {
                        self.parse_expr_statement()
                    }
                } else if p1 == &TokenType::Equals && p2 == &TokenType::New {
                    self.parse_alias_assignment()
                } else if p1 == &TokenType::Equals {
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
        let pkg_token = self.advance();
        let pkg = match pkg_token.token_type {
            TokenType::Ident | TokenType::StringLiteral => pkg_token.lexeme,
            _ => return Err(YppError::Message("Expected package name".to_string())),
        };

        if self.match_token(&TokenType::Star) {
            self.optional_semicolon();
            return Ok(AstNode::Import { pkg, classes: vec![], import_all: true });
        } else if self.match_token(&TokenType::LBrace) {
            let mut classes = Vec::new();
            while !self.check(&TokenType::RBrace) && !self.is_at_end() {
                let class_token = self.advance();
                classes.push(class_token.lexeme);
                self.match_token(&TokenType::Comma);
            }
            self.expect(TokenType::RBrace)?;
            self.optional_semicolon();
            return Ok(AstNode::Import { pkg, classes, import_all: false });
        } else {
            self.optional_semicolon();
            return Ok(AstNode::Import { pkg, classes: vec![], import_all: false });
        }
    }

    fn parse_print(&mut self) -> Result<AstNode, YppError> {
        self.advance(); // consume PRINT
        self.expect(TokenType::Colon)?;

        let mut cast = None;
        if self.check(&TokenType::CastInt) || self.check(&TokenType::CastDouble) || 
           self.check(&TokenType::CastString) || self.check(&TokenType::CastStringint) {
            cast = Some(self.advance().token_type);
        }

        let expr = self.parse_expr()?;
        self.optional_semicolon();
        Ok(AstNode::Print { expr: Box::new(expr), cast })
    }

    fn parse_num_block(&mut self) -> Result<AstNode, YppError> {
        self.advance(); // consume Num
        let mut labels = Vec::new();
        while !self.check(&TokenType::LBrace) && !self.is_at_end() {
            labels.push(self.advance().lexeme);
        }
        self.expect(TokenType::LBrace)?;
        let mut statements = Vec::new();
        while !self.check(&TokenType::RBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        self.expect(TokenType::RBrace)?;
        Ok(AstNode::NumBlock { label: labels.join(" "), statements })
    }

    fn parse_string_block(&mut self) -> Result<AstNode, YppError> {
        self.advance(); // consume STRING
        let name_or_label = self.advance().lexeme;
        
        if self.match_token(&TokenType::LBrace) {
            let mut statements = Vec::new();
            while !self.check(&TokenType::RBrace) && !self.is_at_end() {
                statements.push(self.parse_statement()?);
            }
            self.expect(TokenType::RBrace)?;
            return Ok(AstNode::StringBlock { label: name_or_label, statements });
        }

        let mut expr = None;
        if self.match_token(&TokenType::Equals) {
            expr = Some(Box::new(self.parse_expr()?));
        }
        self.optional_semicolon();
        Ok(AstNode::StringBlock {
            label: name_or_label.clone(),
            statements: vec![AstNode::VarDecl {
                type_name: "STRING".to_string(),
                var_name: name_or_label,
                expr,
            }],
        })
    }

    fn parse_exception_concat(&mut self) -> Result<AstNode, YppError> {
        self.advance(); // Exception
        self.expect(TokenType::Concat)?;
        if self.match_token(&TokenType::LParen) {
            self.expect(TokenType::RParen)?;
        }
        self.expect(TokenType::LBrace)?;
        let mut statements = Vec::new();
        while !self.check(&TokenType::RBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        self.expect(TokenType::RBrace)?;
        Ok(AstNode::ExceptionConcat { statements })
    }

    fn parse_func_decl(&mut self) -> Result<AstNode, YppError> {
        self.advance(); // consume func
        let name = self.advance().lexeme;
        let mut params = Vec::new();
        if self.match_token(&TokenType::LParen) {
            while !self.check(&TokenType::RParen) && !self.is_at_end() {
                let p_type = self.advance().lexeme;
                let p_name = self.advance().lexeme;
                params.push((p_type, p_name));
                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
            self.expect(TokenType::RParen)?;
        }
        self.expect(TokenType::LBrace)?;
        let mut statements = Vec::new();
        while !self.check(&TokenType::RBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        self.expect(TokenType::RBrace)?;
        Ok(AstNode::FuncDecl { name, params, body: statements })
    }

    fn parse_new_alias(&mut self) -> Result<AstNode, YppError> {
        self.advance(); // NEW
        let func_name = self.advance().lexeme;
        let mut block_type = None;
        let mut block_label = None;
        
        if self.match_token(&TokenType::Dot) {
            self.expect(TokenType::LParen)?;
            block_type = Some(self.advance().lexeme);
            block_label = Some(self.advance().lexeme);
            self.expect(TokenType::RParen)?;
        }
        self.expect(TokenType::Equals)?;
        let alias_name = self.advance().lexeme;
        self.optional_semicolon();
        Ok(AstNode::NewAlias { func_name, block_type, block_label, alias_name })
    }

    fn parse_while(&mut self) -> Result<AstNode, YppError> {
        self.advance(); // while
        self.expect(TokenType::LParen)?;
        let condition = self.parse_expr()?;
        self.expect(TokenType::RParen)?;
        self.expect(TokenType::LBrace)?;
        let mut statements = Vec::new();
        while !self.check(&TokenType::RBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        self.expect(TokenType::RBrace)?;
        Ok(AstNode::While { condition: Box::new(condition), body: statements })
    }

    fn parse_bare_var_decl(&mut self) -> Result<AstNode, YppError> {
        let type_name = self.advance().lexeme;
        let var_name = if self.check(&TokenType::Equals) {
            type_name.clone()
        } else {
            self.advance().lexeme
        };
        
        let mut expr = None;
        if self.match_token(&TokenType::Equals) {
            expr = Some(Box::new(self.parse_expr()?));
        }
        if self.check(&TokenType::Comma) {
            self.advance();
        }
        self.optional_semicolon();
        Ok(AstNode::VarDecl { type_name, var_name, expr })
    }

    fn parse_bare_string_var_decl(&mut self) -> Result<AstNode, YppError> {
        self.parse_bare_var_decl()
    }

    fn parse_param_bind(&mut self) -> Result<AstNode, YppError> {
        let type_name = self.advance().lexeme;
        self.expect(TokenType::Dot)?;
        let name = self.advance().lexeme;
        self.expect(TokenType::Equals)?;
        let val_name = self.advance().lexeme;
        self.optional_semicolon();
        Ok(AstNode::ParamBind { type_name, name, val_name })
    }

    fn parse_named_block(&mut self) -> Result<AstNode, YppError> {
        let label = self.advance().lexeme;
        self.expect(TokenType::LBrace)?;
        let mut statements = Vec::new();
        while !self.check(&TokenType::RBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        self.expect(TokenType::RBrace)?;
        Ok(AstNode::NumBlock { label, statements })
    }

    fn parse_param_action(&mut self) -> Result<AstNode, YppError> {
        let target = self.advance().lexeme;
        self.expect(TokenType::Dot)?;
        let action = self.advance().lexeme;
        if action == "input" {
            self.expect(TokenType::LParen)?;
            let prompt = self.parse_expr()?;
            self.expect(TokenType::RParen)?;
            if !self.match_token(&TokenType::Semicolon) {
                return Err(YppError::MissingSemicolonInput);
            }
            Ok(AstNode::ParamInput { target, prompt: Box::new(prompt) })
        } else if action == "next" {
            self.expect(TokenType::LParen)?;
            self.expect(TokenType::RParen)?;
            if !self.match_token(&TokenType::Semicolon) {
                return Err(YppError::MissingSemicolonNext);
            }
            Ok(AstNode::ParamNext { target })
        } else if action == "break" {
            if !self.match_token(&TokenType::Semicolon) {
                return Err(YppError::MissingSemicolonBreak);
            }
            Ok(AstNode::ParamBreak { target })
        } else {
            Err(YppError::Message("Unknown param action".to_string()))
        }
    }

    fn parse_func_call(&mut self) -> Result<AstNode, YppError> {
        let name = self.advance().lexeme;
        self.expect(TokenType::LParen)?;
        self.expect(TokenType::RParen)?;
        Ok(AstNode::FuncCall { name })
    }

    fn parse_alias_assignment(&mut self) -> Result<AstNode, YppError> {
        let alias_name = self.advance().lexeme;
        self.expect(TokenType::Equals)?;
        self.expect(TokenType::New)?;
        let mut block_type = None;
        let mut block_label = None;
        if self.match_token(&TokenType::LParen) {
            block_type = Some(self.advance().lexeme);
            block_label = Some(self.advance().lexeme);
            self.expect(TokenType::RParen)?;
            self.expect(TokenType::Dot)?;
        }
        let func_name = self.advance().lexeme;
        self.expect(TokenType::LParen)?;
        let mut args = Vec::new();
        while !self.check(&TokenType::RParen) && !self.is_at_end() {
            args.push(self.parse_expr()?);
            self.match_token(&TokenType::Comma);
        }
        self.expect(TokenType::RParen)?;
        self.optional_semicolon();
        Ok(AstNode::NewAlias { func_name, block_type, block_label, alias_name })
    }

    fn parse_assignment(&mut self) -> Result<AstNode, YppError> {
        let name = self.advance().lexeme;
        self.expect(TokenType::Equals)?;
        let expr = self.parse_expr()?;
        self.optional_semicolon();
        Ok(AstNode::Assign { name, expr: Box::new(expr) })
    }

    fn parse_expr_statement(&mut self) -> Result<AstNode, YppError> {
        let expr = self.parse_expr()?;
        self.optional_semicolon();
        Ok(AstNode::ExprStatement { expr: Box::new(expr) })
    }

    fn parse_expr(&mut self) -> Result<AstNode, YppError> {
        if self.check(&TokenType::Bang) || (self.check(&TokenType::Not) && self.peek_at(1).token_type == TokenType::Colon) {
            self.advance(); // consume ! or NOT
            if self.peek_at(0).token_type == TokenType::Colon {
                self.advance(); // consume Colon if NOT:
            }
            let inner = self.parse_expr()?;
            return Ok(AstNode::NotExpr { expr: Box::new(inner) });
        }
        self.parse_additive()
    }

    fn parse_additive(&mut self) -> Result<AstNode, YppError> {
        let mut expr = self.parse_multiplicative()?;
        while self.check(&TokenType::Plus) || self.check(&TokenType::Minus) {
            let op = self.advance().token_type;
            let right = self.parse_multiplicative()?;
            expr = AstNode::BinaryOp {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> Result<AstNode, YppError> {
        let mut expr = self.parse_primary()?;
        while self.check(&TokenType::Star) || self.check(&TokenType::Slash) {
            let op = self.advance().token_type;
            let right = self.parse_primary()?;
            expr = AstNode::BinaryOp {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<AstNode, YppError> {
        let mut expr = self.parse_primary_base()?;
        while self.match_token(&TokenType::Dot) {
            let method = self.advance().lexeme;
            self.expect(TokenType::LParen)?;
            let mut args = Vec::new();
            while !self.check(&TokenType::RParen) && !self.is_at_end() {
                args.push(self.parse_expr()?);
                self.match_token(&TokenType::Comma);
            }
            self.expect(TokenType::RParen)?;
            expr = AstNode::MethodCall {
                object: Box::new(expr),
                method,
                args,
            };
        }
        Ok(expr)
    }

    fn parse_primary_base(&mut self) -> Result<AstNode, YppError> {
        let p = self.peek();
        match p.token_type {
            TokenType::New => {
                self.advance();
                let class_name = self.advance().lexeme;
                self.expect(TokenType::LParen)?;
                let mut args = Vec::new();
                while !self.check(&TokenType::RParen) && !self.is_at_end() {
                    args.push(self.parse_expr()?);
                    self.match_token(&TokenType::Comma);
                }
                self.expect(TokenType::RParen)?;
                Ok(AstNode::NewObject { class_name, args })
            }
            TokenType::LParen => {
                self.advance();
                if self.check(&TokenType::Num) || self.check(&TokenType::StringBlock) || self.check(&TokenType::Ident) {
                    let block_type = self.advance().lexeme;
                    let label = if self.check(&TokenType::RParen) {
                        "".to_string()
                    } else {
                        self.advance().lexeme
                    };
                    self.expect(TokenType::RParen)?;
                    let var = self.advance().lexeme;
                    if block_type == "NUM" {
                        Ok(AstNode::NumAccess { label, var })
                    } else {
                        Ok(AstNode::StringAccess { label, var })
                    }
                } else {
                    let expr = self.parse_expr()?;
                    self.expect(TokenType::RParen)?;
                    Ok(expr)
                }
            }
            TokenType::StringLiteral => {
                let val = self.advance().lexeme;
                Ok(AstNode::StringLiteral { value: val })
            }
            TokenType::LitSmallint | TokenType::LitInteger | TokenType::LitDouble | TokenType::NumberLabel => {
                let tok = self.advance();
                let type_name = match tok.token_type {
                    TokenType::LitSmallint => "smallint",
                    TokenType::LitInteger => "integer",
                    TokenType::LitDouble => "double",
                    TokenType::NumberLabel => "label",
                    _ => "",
                }.to_string();
                Ok(AstNode::NumberLiteral { value: tok.lexeme, type_name })
            }
            TokenType::Ident => {
                let ident = self.advance().lexeme;
                if self.check(&TokenType::LParen) {
                    if ident == "userinput" {
                        self.advance();
                        self.expect(TokenType::RParen)?;
                        Ok(AstNode::UserInput)
                    } else {
                        self.advance();
                        let mut args = Vec::new();
                        while !self.check(&TokenType::RParen) && !self.is_at_end() {
                            args.push(self.parse_expr()?);
                            self.match_token(&TokenType::Comma);
                        }
                        self.expect(TokenType::RParen)?;
                        Ok(AstNode::NewObject { class_name: ident, args })
                    }
                } else {
                    Ok(AstNode::Ident { name: ident })
                }
            }
            _ => Err(YppError::ExpectedExpr(self.peek().clone()))
        }
    }
}
