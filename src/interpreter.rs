use std::collections::{HashMap, HashSet};
use std::io::{self, Write};
use crate::ast::*;
use crate::error::{YppError, YppResult};
use crate::value::{format_double, ObjectHandle, Value};
use crate::networking::{
    create_console_reader, create_network, create_server, get_input_stream, get_output_stream,
    server_accept, stream_read, stream_write, NetObject,
};

pub trait InputProvider {
    fn read_line(&mut self, prompt: &str) -> String;
}

pub struct StdinInputProvider;

impl InputProvider for StdinInputProvider {
    fn read_line(&mut self, prompt: &str) -> String {
        print!("{}", prompt);
        let _ = io::stdout().flush();
        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
        input.trim_end_matches(|c| c == '\r' || c == '\n').to_string()
    }
}

#[derive(Clone)]
pub struct AliasTarget {
    pub func_name: String,
    pub block_label: Option<String>,
}

pub struct Interpreter {
    pub num_blocks: HashMap<String, HashMap<String, f64>>,
    pub string_blocks: HashMap<String, HashMap<String, String>>,
    pub globals: HashMap<String, Value>,
    pub param_types: HashMap<String, String>,
    pub functions: HashMap<String, FuncDeclNode>,
    pub aliases: HashMap<String, AliasTarget>,
    pub imported_packages: HashSet<String>,
    pub in_exception_concat: bool,
    pub current_num_block: Option<String>,
    pub current_string_block: Option<String>,
    
    // NetObject storage
    pub objects: HashMap<usize, NetObject>,
    next_obj_id: usize,

    pub input_provider: Box<dyn InputProvider>,
}

#[derive(Clone)]
pub struct FuncDeclNode {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<AstNode>,
    pub line: usize,
}

impl Interpreter {
    pub fn new(input_provider: Box<dyn InputProvider>) -> Self {
        Self {
            num_blocks: HashMap::new(),
            string_blocks: HashMap::new(),
            globals: HashMap::new(),
            param_types: HashMap::new(),
            functions: HashMap::new(),
            aliases: HashMap::new(),
            imported_packages: HashSet::new(),
            in_exception_concat: false,
            current_num_block: None,
            current_string_block: None,
            objects: HashMap::new(),
            next_obj_id: 1,
            input_provider,
        }
    }

    pub fn execute_program(&mut self, program: &AstNode) -> YppResult<()> {
        if let AstNode::Program { statements } = program {
            // Must have `Import ycomponents`
            let has_ycomponents = statements.iter().any(|s| {
                if let AstNode::Import { pkg, .. } = s {
                    pkg == "ycomponents"
                } else {
                    false
                }
            });

            if !has_ycomponents {
                // If there are any non-import statements, error
                let has_code = statements.iter().any(|s| !matches!(s, AstNode::Import { .. }));
                if has_code {
                    return Err(YppError::missing_ycomponents());
                }
            }

            for stmt in statements {
                self.execute_statement(stmt)?;
            }
        }
        Ok(())
    }

    pub fn execute_statement(&mut self, stmt: &AstNode) -> YppResult<()> {
        match stmt {
            AstNode::Import { pkg, .. } => {
                self.imported_packages.insert(pkg.clone());
            }
            AstNode::Print { cast, expr } => {
                let val = self.evaluate(expr)?;
                let out = match cast {
                    Cast::Int => match val {
                        Value::Number(n) => format!("{}", n.round() as i64),
                        _ => format!("{}", val),
                    },
                    Cast::Double => match val {
                        Value::Number(n) => format_double(n),
                        _ => format!("{}", val),
                    },
                    Cast::StringCast | Cast::StringInt => format!("{}", val),
                    Cast::None => format!("{}", val),
                };
                println!("{}", out);
            }
            AstNode::NumBlock { label, statements, .. } => {
                self.current_num_block = Some(label.clone());
                self.num_blocks.entry(label.clone()).or_insert_with(HashMap::new);
                for s in statements {
                    self.execute_statement(s)?;
                }
                self.current_num_block = None;
            }
            AstNode::StringBlock { label, statements, .. } => {
                self.current_string_block = Some(label.clone());
                self.string_blocks.entry(label.clone()).or_insert_with(HashMap::new);
                for s in statements {
                    self.execute_statement(s)?;
                }
                self.current_string_block = None;
            }
            AstNode::VarDecl { type_name, var_name, value, line } => {
                let val = self.evaluate(value)?;
                if let Value::Number(n) = val {
                    // Check bounds
                    match type_name.as_str() {
                        "smallint" => {
                            if n < -1000.0 || n > 1000.0 {
                                return Err(YppError::smallint_out_of_range(*line, n));
                            }
                        }
                        "integer" => {
                            if n < -1_000_000_000_000_000.0 || n > 1_000_000_000_000_000.0 {
                                return Err(YppError::integer_out_of_range(*line, n));
                            }
                        }
                        "double" => {
                            if n < -5677719218.1092 || n > 5677719218.1092 {
                                return Err(YppError::double_out_of_range(*line, n));
                            }
                        }
                        _ => {}
                    }
                    if let Some(ref label) = self.current_num_block {
                        let map = self.num_blocks.get_mut(label).unwrap();
                        map.insert(var_name.clone(), n);
                    }
                }
            }
            AstNode::StringVarDecl { type_name: _, var_name, value, line: _ } => {
                let val = self.evaluate(value)?;
                if let Value::Str(s) = val {
                    if let Some(ref label) = self.current_string_block {
                        let map = self.string_blocks.get_mut(label).unwrap();
                        map.insert(var_name.clone(), s);
                    }
                }
            }
            AstNode::Assign { name, value } => {
                let val = self.evaluate(value)?;
                self.globals.insert(name.clone(), val);
            }
            AstNode::FuncDecl { name, params, body, line } => {
                if self.functions.contains_key(name) {
                    return Err(YppError::function_already_defined(name));
                }
                for param in params {
                    self.param_types.insert(param.name.clone(), param.type_name.clone());
                }
                let func = FuncDeclNode {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    line: *line,
                };
                self.functions.insert(name.clone(), func.clone());
                
                // Auto-execute globals and aliases in body
                for s in &func.body {
                    match s {
                        AstNode::GlobalBlock { inner } => {
                            self.execute_statement(inner)?;
                        }
                        AstNode::NewAlias { func_name, block_type, block_label, alias_name, line: _ } => {
                            self.aliases.insert(alias_name.clone(), AliasTarget {
                                func_name: func_name.clone(),
                                block_label: block_label.clone(),
                            });
                        }
                        _ => {}
                    }
                }
            }
            AstNode::NewAlias { func_name, block_type: _, block_label, alias_name, line: _ } => {
                self.aliases.insert(alias_name.clone(), AliasTarget {
                    func_name: func_name.clone(),
                    block_label: block_label.clone(),
                });
            }
            AstNode::GlobalBlock { inner } => {
                self.execute_statement(inner)?;
            }
            AstNode::FuncCall { alias_name, line } => {
                if let Some(target) = self.aliases.get(alias_name).cloned() {
                    self.execute_function(&target.func_name, &target.block_label, *line)?;
                } else if self.functions.contains_key(alias_name) {
                    self.execute_function(alias_name, &None, *line)?;
                } else {
                    return Err(YppError::undefined_func_or_alias(*line, alias_name));
                }
            }
            AstNode::ParamBind { param_name, param_ref, .. } => {
                // simple validation or store
                let _ = (param_name, param_ref);
            }
            AstNode::ParamInput { param_name, prompt, line } => {
                let prompt_val = self.evaluate(prompt)?;
                let p = format!("{}", prompt_val);
                let input_str = self.input_provider.read_line(&p);
                println!("{}", input_str);

                if let Some(t) = self.param_types.get(param_name).cloned() {
                    if t == "schar" {
                        if input_str.len() > 1 {
                            return Err(YppError::schar_too_long(*line, param_name, &input_str));
                        }
                        self.globals.insert(param_name.clone(), Value::Str(input_str));
                    } else if t == "slong" {
                        self.globals.insert(param_name.clone(), Value::Str(input_str));
                    } else if t == "smallint" || t == "integer" || t == "double" {
                        if let Ok(num) = input_str.parse::<f64>() {
                            self.globals.insert(param_name.clone(), Value::Number(num));
                        } else {
                            return Err(YppError::param_expected_number(*line, &t, param_name, &input_str));
                        }
                    } else {
                        self.globals.insert(param_name.clone(), Value::Str(input_str));
                    }
                } else {
                    self.globals.insert(param_name.clone(), Value::Str(input_str));
                }
            }
            AstNode::ParamNext { .. } => {
                println!();
            }
            AstNode::ParamBreak { .. } => {}
            AstNode::While { condition, body, .. } => {
                while self.evaluate(condition)?.is_truthy() {
                    for s in body {
                        self.execute_statement(s)?;
                    }
                }
            }
            AstNode::ExceptionConcat { statements } => {
                let prev = self.in_exception_concat;
                self.in_exception_concat = true;
                for s in statements {
                    self.execute_statement(s)?;
                }
                self.in_exception_concat = prev;
            }
            AstNode::ExprStatement { expr } => {
                self.evaluate(expr)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn execute_function(&mut self, func_name: &str, target_block: &Option<String>, line: usize) -> YppResult<()> {
        let func = self.functions.get(func_name).cloned().ok_or_else(|| YppError::undefined_func_or_alias(line, func_name))?;
        for stmt in &func.body {
            if let Some(target) = target_block {
                match stmt {
                    AstNode::NumBlock { label, .. } if label == target => {
                        self.execute_statement(stmt)?;
                    }
                    AstNode::StringBlock { label, .. } if label == target => {
                        self.execute_statement(stmt)?;
                    }
                    _ => {} // Skip other statements
                }
            } else {
                self.execute_statement(stmt)?;
            }
        }
        Ok(())
    }

    pub fn evaluate(&mut self, expr: &Expr) -> YppResult<Value> {
        match expr {
            Expr::NumberLiteral { value, .. } => Ok(Value::Number(*value)),
            Expr::StringLiteral { value } => Ok(Value::Str(value.clone())),
            Expr::Ident { name } => {
                if let Some(v) = self.globals.get(name) {
                    Ok(v.clone())
                } else {
                    Err(YppError::undefined_variable(name))
                }
            }
            Expr::NumAccess { block_label, var_name } => {
                if let Some(map) = self.num_blocks.get(block_label) {
                    if let Some(v) = map.get(var_name) {
                        Ok(Value::Number(*v))
                    } else {
                        Err(YppError::undefined_var_in_block(var_name, block_label))
                    }
                } else {
                    Err(YppError::undefined_num_block(block_label))
                }
            }
            Expr::StringAccess { block_label, var_name } => {
                if let Some(map) = self.string_blocks.get(block_label) {
                    if let Some(v) = map.get(var_name) {
                        Ok(Value::Str(v.clone()))
                    } else {
                        Err(YppError::undefined_var_in_block(var_name, block_label))
                    }
                } else {
                    Err(YppError::undefined_string_block(block_label))
                }
            }
            Expr::BinaryExpr { left, op, right } => {
                let l = self.evaluate(left)?;
                let r = self.evaluate(right)?;

                match (l, r) {
                    (Value::Number(n1), Value::Number(n2)) => {
                        match op {
                            '+' => Ok(Value::Number(n1 + n2)),
                            '-' => Ok(Value::Number(n1 - n2)),
                            '*' => Ok(Value::Number(n1 * n2)),
                            '/' => {
                                if n2 == 0.0 {
                                    Err(YppError::division_by_zero())
                                } else {
                                    Ok(Value::Number(n1 / n2))
                                }
                            }
                            _ => Ok(Value::Number(0.0)),
                        }
                    }
                    (Value::Str(s1), Value::Str(s2)) => {
                        if *op == '+' {
                            Ok(Value::Str(format!("{}{}", s1, s2)))
                        } else if *op == '*' {
                            if self.in_exception_concat {
                                Ok(Value::Str(format!("{} {}", s1, s2)))
                            } else {
                                Ok(Value::Str(format!("{}{}", s1, s2)))
                            }
                        } else {
                            Ok(Value::Str(s1))
                        }
                    }
                    (Value::Str(s), Value::Number(n)) | (Value::Number(n), Value::Str(s)) => {
                        if self.in_exception_concat {
                            if let Value::Str(_) = self.evaluate(left)? {
                                Ok(Value::Str(format!("{} {}", s, n)))
                            } else {
                                Ok(Value::Str(format!("{} {}", n, s)))
                            }
                        } else {
                            println!("ERROR");
                            Err(YppError::concat_type_error())
                        }
                    }
                    _ => Ok(Value::Nil),
                }
            }
            Expr::Not { expr, .. } => {
                let val = self.evaluate(expr)?;
                Ok(Value::Bool(!val.is_truthy()))
            }
            Expr::InlineAssign { var_name, expr, .. } => {
                let val = self.evaluate(expr)?;
                self.globals.insert(var_name.clone(), val.clone());
                Ok(val)
            }
            Expr::NewObject { class_name, args, line, .. } => {
                let class_lower = class_name.to_lowercase();
                if class_lower == "network" {
                    if !self.imported_packages.contains("ynetworking") {
                        return Err(YppError::missing_ynetworking(*line, "Network"));
                    }
                    let host = match args.get(0).map(|e| self.evaluate(e)) {
                        Some(Ok(Value::Str(s))) => s,
                        _ => "127.0.0.1".to_string(),
                    };
                    let port = match args.get(1).map(|e| self.evaluate(e)) {
                        Some(Ok(Value::Number(n))) => n as u16,
                        _ => 5000,
                    };
                    let obj = create_network(&host, port, *line)?;
                    let id = self.next_obj_id;
                    self.next_obj_id += 1;
                    self.objects.insert(id, obj);
                    Ok(Value::Object(ObjectHandle::Ref(id)))
                } else if class_lower == "server" {
                    if !self.imported_packages.contains("ynetworking") {
                        return Err(YppError::missing_ynetworking(*line, "Server"));
                    }
                    let port = match args.get(0).map(|e| self.evaluate(e)) {
                        Some(Ok(Value::Number(n))) => n as u16,
                        _ => 5000,
                    };
                    let obj = create_server(port, *line)?;
                    let id = self.next_obj_id;
                    self.next_obj_id += 1;
                    self.objects.insert(id, obj);
                    Ok(Value::Object(ObjectHandle::Ref(id)))
                } else if class_lower == "primitivedatastream" {
                    let arg = args.get(0).unwrap();
                    let val = self.evaluate(arg)?;
                    Ok(val) // The input/output stream method already wrapped it
                } else if class_lower == "reader" {
                    let obj = create_console_reader();
                    let id = self.next_obj_id;
                    self.next_obj_id += 1;
                    self.objects.insert(id, obj);
                    Ok(Value::Object(ObjectHandle::Ref(id)))
                } else if class_lower == "userinput" {
                    Ok(Value::Nil) // dummy for reader(userinput())
                } else {
                    Ok(Value::Nil)
                }
            }
            Expr::MethodCall { target, method_name, args, line } => {
                let target_val = self.evaluate(target)?;
                match target_val {
                    Value::Str(s) => {
                        if method_name == "equals" {
                            let arg_val = self.evaluate(&args[0])?;
                            if let Value::Str(s2) = arg_val {
                                Ok(Value::Bool(s == s2))
                            } else {
                                Ok(Value::Bool(false))
                            }
                        } else if method_name == "length" {
                            Ok(Value::Number(s.len() as f64))
                        } else if method_name == "readline" || method_name == "readutf-8" {
                            Ok(Value::Str(s))
                        } else {
                            Ok(Value::Nil)
                        }
                    }
                    Value::Object(ObjectHandle::Ref(id)) => {
                        // Needs mutable borrow of objects
                        if method_name == "accept" {
                            let listener = if let Some(NetObject::Server { listener }) = self.objects.get(&id) {
                                Some(listener.try_clone().unwrap())
                            } else { None };
                            if let Some(l) = listener {
                                let new_obj = server_accept(&l, *line)?;
                                let new_id = self.next_obj_id;
                                self.next_obj_id += 1;
                                self.objects.insert(new_id, new_obj);
                                Ok(Value::Object(ObjectHandle::Ref(new_id)))
                            } else {
                                Ok(Value::Nil)
                            }
                        } else if method_name == "outstream" {
                            let stream = if let Some(NetObject::Client { stream }) = self.objects.get(&id) {
                                Some(stream.try_clone().unwrap())
                            } else { None };
                            if let Some(s) = stream {
                                let new_obj = get_output_stream(&s, *line)?;
                                let new_id = self.next_obj_id;
                                self.next_obj_id += 1;
                                self.objects.insert(new_id, new_obj);
                                Ok(Value::Object(ObjectHandle::Ref(new_id)))
                            } else {
                                Ok(Value::Nil)
                            }
                        } else if method_name == "inputstream" {
                            let stream = if let Some(NetObject::Client { stream }) = self.objects.get(&id) {
                                Some(stream.try_clone().unwrap())
                            } else { None };
                            if let Some(s) = stream {
                                let new_obj = get_input_stream(&s, *line)?;
                                let new_id = self.next_obj_id;
                                self.next_obj_id += 1;
                                self.objects.insert(new_id, new_obj);
                                Ok(Value::Object(ObjectHandle::Ref(new_id)))
                            } else {
                                Ok(Value::Nil)
                            }
                        } else if method_name == "utf-8" || method_name == "writeutf-8" || method_name == "write" {
                            let arg_val = self.evaluate(&args[0])?;
                            let msg = format!("{}", arg_val);
                            if let Some(NetObject::OutputStream { writer }) = self.objects.get_mut(&id) {
                                stream_write(writer, &msg, *line)?;
                            }
                            Ok(Value::Nil)
                        } else if method_name == "readutf-8" || method_name == "readline" || method_name == "read" {
                            let mut is_console = false;
                            if let Some(NetObject::ConsoleReader) = self.objects.get(&id) {
                                is_console = true;
                            }
                            if is_console {
                                let input = self.input_provider.read_line("");
                                Ok(Value::Str(input))
                            } else {
                                if let Some(NetObject::InputStream { reader }) = self.objects.get_mut(&id) {
                                    let line_str = stream_read(reader, *line)?;
                                    Ok(Value::Str(line_str))
                                } else {
                                    Ok(Value::Nil)
                                }
                            }
                        } else if method_name == "close" {
                            self.objects.remove(&id);
                            Ok(Value::Nil)
                        } else {
                            Ok(Value::Nil)
                        }
                    }
                    _ => Ok(Value::Nil),
                }
            }
        }
    }
}
