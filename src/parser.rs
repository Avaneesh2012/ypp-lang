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
            &self.tokens[self.tokens.len() - 1]
        }
    }

    fn peek_at(&self, offset: usize) -> &Token {
        if self.pos + offset < self.tokens.len() {
            &self.tokens[self.pos + offset]
        } else {
            &self.tokens[self.tokens.len() - 1]
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
            TokenType::If => self.parse_if(),
            TokenType::KwSmallint | TokenType::KwInteger | TokenType::KwDouble | TokenType::KwBool => {
                self.parse_bare_var_decl()
            }
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
                } else if p1 == TokenType::PlusPlus || p1 == TokenType::MinusMinus {
                    self.parse_postfix_statement()
                } else if p1 == TokenType::Dot {
                    let m = &self.peek_at(2).value;
                    if m == "input" || m == "next" || m == "break" {
                        self.parse_param_action()
                    } else if p2 == TokenType::Ident && self.peek_at(3).token_type == TokenType::Equals {
                        self.parse_member_assign()
                    } else {
                        self.parse_expr_statement()
                    }
                } else if p1 == TokenType::LParen {
                    // name() as a statement if it is not clearly an assignment target
                    self.parse_expr_statement()
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

    fn parse_postfix_statement(&mut self) -> Result<AstNode, YppError> {
        let tok = self.advance();
        let line = tok.line;
        let delta = if self.match_token(TokenType::PlusPlus) {
            1.0
        } else {
            self.expect(TokenType::MinusMinus, "'--'")?;
            -1.0
        };
        self.optional_semicolon();
        Ok(AstNode::ExprStatement {
            expr: Expr::Postfix {
                name: tok.value,
                delta,
                line,
            },
        })
    }

    fn parse_member_assign(&mut self) -> Result<AstNode, YppError> {
        let target_tok = self.advance();
        let line = target_tok.line;
        self.expect(TokenType::Dot, "'.'")?;
        let field = self.expect(TokenType::Ident, "field name")?.value;
        self.expect(TokenType::Equals, "'='")?;
        let value = self.parse_expr()?;
        self.optional_semicolon();
        Ok(AstNode::MemberAssign {
            target: Expr::Ident { name: target_tok.value },
            field,
            value,
            line,
        })
    }

    fn parse_import(&mut self) -> Result<AstNode, YppError> {
        let line = self.peek().line;
        self.advance(); // consume Import
        let pkg_tok = self.expect(TokenType::Ident, "package name")?;
        let pkg = pkg_tok.value;
        let mut import_all = false;
        let mut class_name = None;

        if self.match_token(TokenType::ImportAll) || self.match_token(TokenType::Star) {
            import_all = true;
        } else if self.match_token(TokenType::LBrace) {
            class_name = Some(self.expect(TokenType::Ident, "class name")?.value);
            self.expect(TokenType::RBrace, "'}'")?;
        }
        self.optional_semicolon();
        let _ = line;
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

    fn parse_block_label(&mut self) -> Result<String, YppError> {
        let tok = self.peek().clone();
        match tok.token_type {
            TokenType::NumberLabel | TokenType::Ident | TokenType::LitInteger | TokenType::LitSmallint => {
                Ok(self.advance().value)
            }
            _ => Err(YppError::expected_token(
                tok.line,
                "block label",
                &format!("{:?}", tok.token_type),
                &tok.value,
            )),
        }
    }

    fn parse_num_block(&mut self) -> Result<AstNode, YppError> {
        self.advance(); // consume NUM
        let line = self.peek().line;
        let label = self.parse_block_label()?;
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
                    params.push(self.parse_param()?);
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

    fn parse_param(&mut self) -> Result<Param, YppError> {
        let first = self.advance();
        let type_like = matches!(
            first.token_type,
            TokenType::KwSmallint
                | TokenType::KwInteger
                | TokenType::KwDouble
                | TokenType::KwSlong
                | TokenType::KwSchar
                | TokenType::KwBool
        );
        if type_like && self.check(&TokenType::Ident) {
            let p_name = self.advance().value;
            Ok(Param {
                type_name: first.value,
                name: p_name,
            })
        } else {
            Ok(Param {
                type_name: "integer".to_string(),
                name: first.value,
            })
        }
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

    fn parse_block_body(&mut self) -> Result<Vec<AstNode>, YppError> {
        self.expect(TokenType::LBrace, "'{'")?;
        let mut body = Vec::new();
        while !self.check(&TokenType::RBrace) && !self.is_at_end() {
            body.push(self.parse_statement()?);
        }
        self.expect(TokenType::RBrace, "'}'")?;
        Ok(body)
    }

    fn parse_while(&mut self) -> Result<AstNode, YppError> {
        let line = self.peek().line;
        self.advance(); // consume while
        let condition = if self.match_token(TokenType::LParen) {
            let c = self.parse_expr()?;
            self.expect(TokenType::RParen, "')'")?;
            c
        } else {
            self.parse_expr()?
        };
        let body = self.parse_block_body()?;
        Ok(AstNode::While { condition, body, line })
    }

    fn parse_if(&mut self) -> Result<AstNode, YppError> {
        let line = self.peek().line;
        self.advance(); // consume if
        let condition = if self.match_token(TokenType::LParen) {
            let c = self.parse_expr()?;
            self.expect(TokenType::RParen, "')'")?;
            c
        } else {
            self.parse_expr()?
        };
        let body = self.parse_block_body()?;
        let else_body = if self.match_token(TokenType::Else) {
            Some(self.parse_block_body()?)
        } else {
            None
        };
        Ok(AstNode::If {
            condition,
            body,
            else_body,
            line,
        })
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
        let first = self.expect(TokenType::Ident, "variable name")?.value;
        let mut names = vec![first];

        while self.check(&TokenType::Comma)
            && self.peek_at(1).token_type == TokenType::Ident
            && self.peek_at(2).token_type != TokenType::Equals
            && !matches!(
                self.peek_at(1).token_type,
                TokenType::KwSmallint
                    | TokenType::KwInteger
                    | TokenType::KwDouble
                    | TokenType::KwSlong
                    | TokenType::KwSchar
                    | TokenType::KwBool
            )
        {
            // schar w,a,s,d = [...]
            if self.peek_at(2).token_type == TokenType::Comma
                || self.peek_at(2).token_type == TokenType::Equals
            {
                self.advance(); // comma
                names.push(self.advance().value);
            } else {
                break;
            }
        }

        // After last name: `schar w,a,s,d =` — last ident was pushed; equals follows
        if names.len() == 1 && self.check(&TokenType::Comma) {
            // Could still be `w, a, s, d =` if peek_at(2) was Equals after a later name.
            // Handle `w,a,s,d =` where after first name comma+ident+comma
            while self.check(&TokenType::Comma) && self.peek_at(1).token_type == TokenType::Ident {
                self.advance();
                names.push(self.advance().value);
                if self.check(&TokenType::Equals) {
                    break;
                }
            }
        }

        let value = if self.match_token(TokenType::Equals) {
            self.parse_expr()?
        } else {
            Expr::StringLiteral { value: "".to_string() }
        };
        self.match_token(TokenType::Comma);
        self.optional_semicolon();

        if names.len() == 1 {
            Ok(AstNode::StringVarDecl {
                type_name,
                var_name: names.remove(0),
                value,
                line,
            })
        } else {
            Ok(AstNode::MultiStringVarDecl {
                type_name,
                names,
                values: value,
                line,
            })
        }
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

        // Inline assign (but not ==)
        if self.peek().token_type == TokenType::Ident && self.peek_at(1).token_type == TokenType::Equals {
            let var_name = self.advance().value;
            self.advance(); // '='
            let expr = self.parse_expr()?;
            return Ok(Expr::InlineAssign { var_name, expr: Box::new(expr), line });
        }

        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<Expr, YppError> {
        let mut expr = self.parse_additive()?;
        loop {
            let op = if self.match_token(TokenType::EqEq) {
                Some(CmpOp::Eq)
            } else if self.match_token(TokenType::NotEq) {
                Some(CmpOp::Ne)
            } else if self.match_token(TokenType::LtEq) {
                Some(CmpOp::Le)
            } else if self.match_token(TokenType::GtEq) {
                Some(CmpOp::Ge)
            } else if self.match_token(TokenType::Lt) {
                Some(CmpOp::Lt)
            } else if self.match_token(TokenType::Gt) {
                Some(CmpOp::Gt)
            } else {
                None
            };
            if let Some(op) = op {
                let right = self.parse_additive()?;
                expr = Expr::CompareExpr {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(expr)
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
        let mut expr = self.parse_postfix_expr()?;
        while self.check(&TokenType::Star) || self.check(&TokenType::Slash) {
            let op = self.advance().value.chars().next().unwrap();
            let right = self.parse_postfix_expr()?;
            expr = Expr::BinaryExpr { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn parse_postfix_expr(&mut self) -> Result<Expr, YppError> {
        if self.peek().token_type == TokenType::Ident
            && (self.peek_at(1).token_type == TokenType::PlusPlus
                || self.peek_at(1).token_type == TokenType::MinusMinus)
        {
            let tok = self.advance();
            let delta = if self.match_token(TokenType::PlusPlus) {
                1.0
            } else {
                self.advance();
                -1.0
            };
            return Ok(Expr::Postfix {
                name: tok.value,
                delta,
                line: tok.line,
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, YppError> {
        let mut expr = self.parse_primary_base()?;
        loop {
            if self.match_token(TokenType::Dot) {
                let line = self.peek().line;
                if self.match_token(TokenType::LParen) {
                    let bt = self.advance().value;
                    let label = self.advance().value;
                    self.expect(TokenType::RParen, "')'")?;
                    expr = Expr::BlockRef {
                        block_type: bt,
                        label,
                    };
                } else {
                    let method_name = self.expect(TokenType::Ident, "method or field name")?.value;
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
                        expr = Expr::MethodCall {
                            target: Box::new(expr),
                            method_name,
                            args,
                            line,
                        };
                    } else {
                        expr = Expr::Property {
                            target: Box::new(expr),
                            name: method_name,
                            line,
                        };
                    }
                }
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_arg_list(&mut self) -> Result<Vec<Expr>, YppError> {
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
        Ok(args)
    }

    fn parse_primary_base(&mut self) -> Result<Expr, YppError> {
        let tok = self.peek().clone();
        match tok.token_type {
            TokenType::New => self.parse_new_object_expr(),
            TokenType::True => {
                self.advance();
                Ok(Expr::BoolLiteral { value: true })
            }
            TokenType::False => {
                self.advance();
                Ok(Expr::BoolLiteral { value: false })
            }
            TokenType::LBracket => {
                self.advance();
                let mut elements = Vec::new();
                if !self.check(&TokenType::RBracket) {
                    loop {
                        elements.push(self.parse_expr()?);
                        if !self.match_token(TokenType::Comma) {
                            break;
                        }
                    }
                }
                self.expect(TokenType::RBracket, "']'")?;
                Ok(Expr::ArrayLiteral { elements })
            }
            TokenType::Num | TokenType::StringBlock => {
                let bt = self.advance().value;
                let label = self.advance().value;
                Ok(Expr::BlockRef {
                    block_type: bt,
                    label,
                })
            }
            TokenType::LParen => {
                self.advance();
                let peek = self.peek().value.clone();
                if peek == "NUM" || peek == "STRING" {
                    self.advance();
                    let label = self.advance().value;
                    self.expect(TokenType::RParen, "')'")?;
                    if self.check(&TokenType::Ident) {
                        let var = self.advance().value;
                        if peek == "NUM" {
                            Ok(Expr::NumAccess { block_label: label, var_name: var })
                        } else {
                            Ok(Expr::StringAccess { block_label: label, var_name: var })
                        }
                    } else {
                        Ok(Expr::BlockRef {
                            block_type: peek,
                            label,
                        })
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
            TokenType::LitString | TokenType::LitChar => {
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
                }
                .to_string();
                let value = tok.value.parse::<f64>().unwrap_or(0.0);
                Ok(Expr::NumberLiteral { value, type_name })
            }
            TokenType::Ident => {
                self.advance();
                let name = tok.value;
                if self.match_token(TokenType::LParen) {
                    let args = self.parse_arg_list()?;
                    Ok(Expr::NewObject {
                        class_name: name,
                        args,
                        block_type: None,
                        block_label: None,
                        line: tok.line,
                    })
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
        let args = self.parse_arg_list()?;
        Ok(Expr::NewObject {
            block_type: None,
            block_label: None,
            class_name,
            args,
            line,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn parses_example6_gui_program() {
        let src = include_str!("../examples/example6.ypp");
        let mut lexer = Lexer::new(src);
        let mut parser = Parser::new(lexer.tokenize());
        parser.parse().expect("example6.ypp should parse");
    }
}
