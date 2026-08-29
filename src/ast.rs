/// AST node definitions for Y++.

/// Cast mode for PRINT statements.
#[derive(Debug, Clone, PartialEq)]
pub enum Cast {
    None,
    Int,
    Double,
    StringCast,
    StringInt,
}

/// Function parameter declaration.
#[derive(Debug, Clone)]
pub struct Param {
    pub type_name: String,
    pub name: String,
}

/// Expression nodes.
#[derive(Debug, Clone)]
pub enum Expr {
    NumberLiteral {
        value: f64,
        type_name: String,
    },
    StringLiteral {
        value: String,
    },
    Ident {
        name: String,
    },
    NumAccess {
        block_label: String,
        var_name: String,
    },
    StringAccess {
        block_label: String,
        var_name: String,
    },
    BinaryExpr {
        left: Box<Expr>,
        op: char,
        right: Box<Expr>,
    },
    MethodCall {
        target: Box<Expr>,
        method_name: String,
        args: Vec<Expr>,
        line: usize,
    },
    NewObject {
        block_type: Option<String>,
        block_label: Option<String>,
        class_name: String,
        args: Vec<Expr>,
        line: usize,
    },
    Not {
        expr: Box<Expr>,
        line: usize,
    },
    InlineAssign {
        var_name: String,
        expr: Box<Expr>,
        line: usize,
    },
}

/// Statement / AST nodes.
#[derive(Debug, Clone)]
pub enum AstNode {
    Program {
        statements: Vec<AstNode>,
    },
    Import {
        pkg: String,
        import_all: bool,
        class_name: Option<String>,
    },
    NumBlock {
        label: String,
        statements: Vec<AstNode>,
        line: usize,
    },
    VarDecl {
        type_name: String,
        var_name: String,
        value: Expr,
        line: usize,
    },
    StringBlock {
        label: String,
        statements: Vec<AstNode>,
        line: usize,
    },
    StringVarDecl {
        type_name: String,
        var_name: String,
        value: Expr,
        line: usize,
    },
    Print {
        cast: Cast,
        expr: Expr,
    },
    ExceptionConcat {
        statements: Vec<AstNode>,
    },
    Assign {
        name: String,
        value: Expr,
    },
    FuncDecl {
        name: String,
        params: Vec<Param>,
        body: Vec<AstNode>,
        line: usize,
    },
    NewAlias {
        func_name: String,
        block_type: Option<String>,
        block_label: Option<String>,
        alias_name: String,
        line: usize,
    },
    GlobalBlock {
        inner: Box<AstNode>,
    },
    FuncCall {
        alias_name: String,
        line: usize,
    },
    ParamBind {
        type_name: String,
        param_name: String,
        param_ref: String,
        line: usize,
    },
    ParamInput {
        param_name: String,
        prompt: Expr,
        line: usize,
    },
    ParamNext {
        param_name: String,
    },
    ParamBreak {
        param_name: String,
    },
    While {
        condition: Expr,
        body: Vec<AstNode>,
        line: usize,
    },
    /// Expression used as a statement (e.g. bare method call).
    ExprStatement {
        expr: Expr,
    },
}
