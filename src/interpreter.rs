use std::collections::{HashMap, HashSet};
use std::io::{self, Write};
use crate::ast::*;
use crate::error::{YppError, YppResult};
use crate::gui::{color_from_value, color_named, GuiRuntime};
use crate::packages;
use crate::value::{format_double, ObjectHandle, Value};
use crate::networking::{
    create_console_reader, create_network, create_server, get_input_stream, get_output_stream,
    server_accept, stream_read, stream_write, NetObject,
};

const MAX_RUNTIME_OBJECTS: usize = 1024;

/// Returned by execute_statement to propagate control flow signals up the call stack.
#[derive(Debug, PartialEq)]
pub enum ControlFlow {
    Normal,
    Continue,
}

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
    pub objects: HashMap<usize, NetObject>,
    next_obj_id: usize,
    pub input_provider: Box<dyn InputProvider>,
    gui: Option<GuiRuntime>,
    frame_id: Option<usize>,
    keyboard_id: Option<usize>,
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
            gui: None,
            frame_id: None,
            keyboard_id: None,
        }
    }

    fn has_pkg(&self, pkg: &str) -> bool {
        if packages::is_networking(pkg) {
            self.imported_packages.contains(packages::PKG_NETWORKING)
        } else if packages::is_gui(pkg) {
            self.imported_packages.contains(packages::PKG_GUI)
        } else {
            self.imported_packages.contains(&packages::normalize_package(pkg))
        }
    }

    fn alloc_object(&mut self, obj: NetObject, line: usize) -> YppResult<usize> {
        if self.objects.len() >= MAX_RUNTIME_OBJECTS {
            return Err(YppError::object_limit_reached(line));
        }
        let id = self.next_obj_id;
        self.next_obj_id = self.next_obj_id.saturating_add(1);
        self.objects.insert(id, obj);
        Ok(id)
    }

    fn require_gui(&self, line: usize, feature: &str) -> YppResult<()> {
        if self.has_pkg(packages::PKG_GUI) {
            Ok(())
        } else {
            Err(YppError::missing_ygui(line, feature))
        }
    }

    fn require_net(&self, line: usize, feature: &str) -> YppResult<()> {
        if self.has_pkg(packages::PKG_NETWORKING) {
            Ok(())
        } else {
            Err(YppError::missing_ynetworking(line, feature))
        }
    }

    fn install_gui_builtins(&mut self, line: usize) -> YppResult<()> {
        for name in [
            "BLACK", "WHITE", "RED", "GREEN", "BLUE", "YELLOW", "CYAN", "AQUA", "MAGENTA",
            "PURPLE", "ORANGE", "GRAY", "GREY", "PINK", "BROWN", "NAVY", "TEAL", "LIME", "SILVER",
        ] {
            if let Some(c) = color_named(name) {
                self.globals.insert(name.to_string(), Value::Number(c as f64));
            }
        }
        if self.keyboard_id.is_none() {
            let id = self.alloc_object(NetObject::Keyboard, line)?;
            self.keyboard_id = Some(id);
            self.globals
                .insert("KEYBOARD".to_string(), Value::Object(ObjectHandle::Ref(id)));
        }
        Ok(())
    }

    pub fn execute_program(&mut self, program: &AstNode) -> YppResult<()> {
        if let AstNode::Program { statements } = program {
            let has_ycomponents = statements.iter().any(|s| {
                if let AstNode::Import { pkg, .. } = s {
                    packages::is_components(pkg)
                } else {
                    false
                }
            });

            if !has_ycomponents {
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

    pub fn execute_statement(&mut self, stmt: &AstNode) -> Result<ControlFlow, YppError> {
        match stmt {
            AstNode::Import { pkg, .. } => {
                let line = 1;
                if !packages::is_known_package(pkg) {
                    return Err(YppError::unknown_package(line, pkg));
                }
                let canon = packages::canonical_package(pkg);
                self.imported_packages.insert(canon.clone());
                if packages::is_gui(&canon) {
                    self.install_gui_builtins(line)?;
                }
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
                match val {
                    Value::Number(n) => {
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
                            "bool" => {}
                            _ => {}
                        }
                        if let Some(ref label) = self.current_num_block {
                            let map = self.num_blocks.get_mut(label).unwrap();
                            map.insert(var_name.clone(), n);
                        }
                        self.globals.insert(var_name.clone(), Value::Number(n));
                    }
                    Value::Bool(b) => {
                        self.globals.insert(var_name.clone(), Value::Bool(b));
                    }
                    other => {
                        self.globals.insert(var_name.clone(), other);
                    }
                }
            }
            AstNode::StringVarDecl { type_name, var_name, value, line } => {
                let val = self.evaluate(value)?;
                self.assign_string_var(type_name, var_name, val, *line)?;
            }
            AstNode::MultiStringVarDecl { type_name, names, values, line } => {
                let val = self.evaluate(values)?;
                match val {
                    Value::List(items) => {
                        for (i, name) in names.iter().enumerate() {
                            let item = items.get(i).cloned().unwrap_or(Value::Str(String::new()));
                            self.assign_string_var(type_name, name, item, *line)?;
                        }
                    }
                    other => {
                        if let Some(first) = names.first() {
                            self.assign_string_var(type_name, first, other, *line)?;
                        }
                    }
                }
            }
            AstNode::Assign { name, value } => {
                let val = self.evaluate(value)?;
                self.globals.insert(name.clone(), val);
            }
            AstNode::MemberAssign { target, field, value, line } => {
                let val = self.evaluate(value)?;
                self.assign_member(target, field, val, *line)?;
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

                for s in &func.body {
                    match s {
                        AstNode::GlobalBlock { inner } => {
                            self.execute_statement(inner)?;
                        }
                        AstNode::NewAlias { func_name, block_label, alias_name, .. } => {
                            self.aliases.insert(alias_name.clone(), AliasTarget {
                                func_name: func_name.clone(),
                                block_label: block_label.clone(),
                            });
                        }
                        _ => {}
                    }
                }
            }
            AstNode::NewAlias { func_name, block_label, alias_name, .. } => {
                self.aliases.insert(alias_name.clone(), AliasTarget {
                    func_name: func_name.clone(),
                    block_label: block_label.clone(),
                });
            }
            AstNode::GlobalBlock { inner } => {
                self.execute_statement(inner)?;
            }
            AstNode::FuncCall { alias_name, args, line } => {
                let _ = self.call_name(alias_name, args, *line)?;
            }
            AstNode::ParamBind { param_name, param_ref, .. } => {
                let _ = (param_name, param_ref);
            }
            AstNode::ParamInput { param_name, prompt, line } => {
                let prompt_val = self.evaluate(prompt)?;
                let p = format!("{}", prompt_val);
                let input_str = self.input_provider.read_line(&p);
                println!("{}", input_str);

                if let Some(t) = self.param_types.get(param_name).cloned() {
                    if t == "schar" {
                        if input_str.chars().count() > 1 {
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
            AstNode::Continue { .. } => {
                return Ok(ControlFlow::Continue);
            }
            AstNode::While { condition, body, .. } => {
                while self.evaluate(condition)?.is_truthy() {
                    if self.gui.as_ref().map(|g| g.closed).unwrap_or(false) {
                        break;
                    }
                    let mut did_continue = false;
                    for s in body {
                        if self.execute_statement(s)? == ControlFlow::Continue {
                            did_continue = true;
                            break;
                        }
                    }
                    let _ = did_continue; // loop just restarts naturally
                }
            }
            AstNode::For { init, condition, update, body, .. } => {
                if let Some(i) = init {
                    self.execute_statement(i)?;
                }
                loop {
                    if let Some(c) = condition {
                        if !self.evaluate(c)?.is_truthy() {
                            break;
                        }
                    }
                    if self.gui.as_ref().map(|g| g.closed).unwrap_or(false) {
                        break;
                    }
                    let mut did_continue = false;
                    for s in body {
                        if self.execute_statement(s)? == ControlFlow::Continue {
                            did_continue = true;
                            break;
                        }
                    }
                    // Always run update even if continue fired
                    if let Some(u) = update {
                        self.execute_statement(u)?;
                    }
                    let _ = did_continue;
                }
            }
            AstNode::If { condition, body, else_body, .. } => {
                if self.evaluate(condition)?.is_truthy() {
                    for s in body {
                        let cf = self.execute_statement(s)?;
                        if cf == ControlFlow::Continue {
                            return Ok(ControlFlow::Continue);
                        }
                    }
                } else if let Some(eb) = else_body {
                    for s in eb {
                        let cf = self.execute_statement(s)?;
                        if cf == ControlFlow::Continue {
                            return Ok(ControlFlow::Continue);
                        }
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
                if let Expr::NewObject { class_name, args, line, .. } = expr {
                    let _ = self.call_name(class_name, args, *line)?;
                } else {
                    self.evaluate(expr)?;
                }
            }
            _ => {}
        }
        Ok(ControlFlow::Normal)
    }

    fn assign_string_var(&mut self, type_name: &str, var_name: &str, val: Value, line: usize) -> YppResult<()> {
        let s = match val {
            Value::Str(s) => s,
            other => format!("{}", other),
        };
        if type_name == "schar" && s.chars().count() > 1 {
            return Err(YppError::schar_too_long(line, var_name, &s));
        }
        if let Some(ref label) = self.current_string_block {
            let map = self.string_blocks.get_mut(label).unwrap();
            map.insert(var_name.to_string(), s.clone());
        }
        self.globals.insert(var_name.to_string(), Value::Str(s));
        Ok(())
    }

    fn assign_member(&mut self, target: &Expr, field: &str, val: Value, line: usize) -> YppResult<()> {
        let field_l = field.to_ascii_lowercase();
        if let Expr::Ident { name } = target {
            if let Some(Value::Object(ObjectHandle::Ref(id))) = self.globals.get(name).cloned() {
                if self.frame_id == Some(id) || matches!(self.objects.get(&id), Some(NetObject::Frame)) {
                    self.require_gui(line, "frame properties")?;
                    if field_l == "color" {
                        let color = match val {
                            Value::Number(n) => color_from_value(n),
                            Value::Str(s) => color_named(&s).unwrap_or(0),
                            _ => 0,
                        };
                        if let Some(gui) = self.gui.as_mut() {
                            gui.set_color(color);
                        }
                        return Ok(());
                    }
                }
            }
            if let Some(map) = self.num_blocks.get_mut(name) {
                if let Value::Number(n) = val {
                    map.insert(field.to_string(), n);
                    return Ok(());
                }
            }
            if let Some(map) = self.string_blocks.get_mut(name) {
                map.insert(field.to_string(), format!("{}", val));
                return Ok(());
            }
        }
        Ok(())
    }

    fn call_name(&mut self, name: &str, args: &[Expr], line: usize) -> YppResult<Value> {
        if packages::is_gui(name) || name.eq_ignore_ascii_case("ygui") {
            return self.tick_gui(line);
        }
        if name.eq_ignore_ascii_case("dimensions") {
            return self.builtin_dimensions(args, line);
        }

        if let Some(target) = self.aliases.get(name).cloned() {
            if packages::is_gui(&target.func_name) {
                return self.setup_gui(&target.block_label, line);
            }
            return self.execute_function(&target.func_name, &target.block_label, args, line);
        }
        if self.functions.contains_key(name) {
            return self.execute_function(name, &None, args, line);
        }
        // Fall through to object constructors used as statements.
        self.evaluate(&Expr::NewObject {
            class_name: name.to_string(),
            args: args.to_vec(),
            block_type: None,
            block_label: None,
            line,
        })
    }

    fn setup_gui(&mut self, block_label: &Option<String>, line: usize) -> YppResult<Value> {
        self.require_gui(line, "yGUI")?;
        let (w, h) = self.read_frame_dims(block_label);
        if self.gui.is_none() {
            self.gui = Some(GuiRuntime::new(w, h, line)?);
            let cx = (w.saturating_sub(3) / 2) as f64;
            let cy = (h.saturating_sub(3) / 2) as f64;
            self.globals.entry("x".to_string()).or_insert(Value::Number(cx));
            self.globals.entry("y".to_string()).or_insert(Value::Number(cy));
        } else if let Some(gui) = self.gui.as_mut() {
            gui.set_dimensions(w, h, line)?;
        }
        let id = if let Some(id) = self.frame_id {
            id
        } else {
            let id = self.alloc_object(NetObject::Frame, line)?;
            self.frame_id = Some(id);
            id
        };
        Ok(Value::Object(ObjectHandle::Ref(id)))
    }

    fn read_frame_dims(&self, block_label: &Option<String>) -> (usize, usize) {
        let mut w = 60usize;
        let mut h = 70usize;
        let labels: Vec<String> = if let Some(l) = block_label {
            vec![l.clone()]
        } else {
            self.num_blocks.keys().cloned().collect()
        };
        for label in labels {
            if let Some(map) = self.num_blocks.get(&label) {
                if let Some(v) = map.get("framewidth").or_else(|| map.get("width")) {
                    w = (*v as i64).max(1) as usize;
                }
                if let Some(v) = map.get("framelength").or_else(|| map.get("height")).or_else(|| map.get("length")) {
                    h = (*v as i64).max(1) as usize;
                }
            }
        }
        (w, h)
    }

    fn tick_gui(&mut self, line: usize) -> YppResult<Value> {
        self.require_gui(line, "yGUI()")?;
        if self.gui.is_none() {
            let (w, h) = self.read_frame_dims(&None);
            self.gui = Some(GuiRuntime::new(w, h, line)?);
            if self.frame_id.is_none() {
                let id = self.alloc_object(NetObject::Frame, line)?;
                self.frame_id = Some(id);
            }
        }
        self.sync_player_from_globals();
        let open = if let Some(gui) = self.gui.as_mut() {
            gui.present(line)?
        } else {
            false
        };
        if !open {
            self.globals.insert("trueforever".to_string(), Value::Bool(false));
        }
        Ok(Value::Bool(open))
    }

    fn sync_player_from_globals(&mut self) {
        let x = self.number_of("x").or_else(|| self.number_of("playerx"));
        let y = self.number_of("y").or_else(|| self.number_of("playery"));
        let w = self
            .lookup_entity_dim("width")
            .or_else(|| self.number_of("playerwidth"));
        let h = self
            .lookup_entity_dim("length")
            .or_else(|| self.lookup_entity_dim("height"))
            .or_else(|| self.number_of("playerlength"));
        if let Some(gui) = self.gui.as_mut() {
            let ew = w.filter(|n| *n > 0.0).unwrap_or(3.0) as usize;
            let eh = h.filter(|n| *n > 0.0).unwrap_or(3.0) as usize;
            gui.upsert_player(ew, eh);
            if let Some(p) = gui.player_mut() {
                if let Some(xv) = x {
                    p.x = xv as i32;
                }
                if let Some(yv) = y {
                    p.y = yv as i32;
                }
            }
        }
    }

    fn lookup_entity_dim(&self, field: &str) -> Option<f64> {
        for map in self.num_blocks.values() {
            if let Some(v) = map.get(field) {
                return Some(*v);
            }
        }
        self.number_of(field)
    }

    fn number_of(&self, name: &str) -> Option<f64> {
        match self.globals.get(name) {
            Some(Value::Number(n)) => Some(*n),
            _ => None,
        }
    }

    fn builtin_dimensions(&mut self, args: &[Expr], line: usize) -> YppResult<Value> {
        let w = match args.get(0).map(|e| self.evaluate(e)) {
            Some(Ok(Value::Number(n))) => n,
            _ => 1.0,
        };
        let h = match args.get(1).map(|e| self.evaluate(e)) {
            Some(Ok(Value::Number(n))) => n,
            _ => 1.0,
        };
        if let Some(gui) = self.gui.as_mut() {
            gui.upsert_player(w.max(1.0) as usize, h.max(1.0) as usize);
        }
        let id = self.alloc_object(NetObject::Dimensions { width: w, height: h }, line)?;
        Ok(Value::Object(ObjectHandle::Ref(id)))
    }

    fn execute_function(
        &mut self,
        func_name: &str,
        target_block: &Option<String>,
        args: &[Expr],
        line: usize,
    ) -> YppResult<Value> {
        let func = self
            .functions
            .get(func_name)
            .cloned()
            .ok_or_else(|| YppError::undefined_func_or_alias(line, func_name))?;

        for (i, param) in func.params.iter().enumerate() {
            let val = if let Some(expr) = args.get(i) {
                self.evaluate(expr)?
            } else {
                self.globals
                    .get(&param.name)
                    .cloned()
                    .unwrap_or(Value::Number(0.0))
            };
            self.globals.insert(param.name.clone(), val);
        }

        for stmt in &func.body {
            if let Some(target) = target_block {
                match stmt {
                    AstNode::NumBlock { label, .. } if label == target => {
                        self.execute_statement(stmt)?;
                    }
                    AstNode::StringBlock { label, .. } if label == target => {
                        self.execute_statement(stmt)?;
                    }
                    _ => {}
                }
            } else {
                self.execute_statement(stmt)?;
            }
        }

        if func_name == "entity" {
            self.sync_player_from_globals();
        }

        Ok(self.globals.get(func_name).cloned().unwrap_or(Value::Nil))
    }

    pub fn evaluate(&mut self, expr: &Expr) -> YppResult<Value> {
        match expr {
            Expr::NumberLiteral { value, .. } => Ok(Value::Number(*value)),
            Expr::StringLiteral { value } => Ok(Value::Str(value.clone())),
            Expr::BoolLiteral { value } => Ok(Value::Bool(*value)),
            Expr::ArrayLiteral { elements } => {
                let mut out = Vec::new();
                for e in elements {
                    out.push(self.evaluate(e)?);
                }
                Ok(Value::List(out))
            }
            Expr::BlockRef { .. } => Ok(Value::Nil),
            Expr::Postfix { name, delta, .. } => {
                let cur = match self.globals.get(name) {
                    Some(Value::Number(n)) => *n,
                    _ => 0.0,
                };
                let next = cur + delta;
                self.globals.insert(name.clone(), Value::Number(next));
                if let Some(gui) = self.gui.as_mut() {
                    if let Some(p) = gui.player_mut() {
                        if name == "x" {
                            p.x = next as i32;
                        }
                        if name == "y" {
                            p.y = next as i32;
                        }
                    }
                }
                Ok(Value::Number(next))
            }
            Expr::Ident { name } => {
                if let Some(v) = self.globals.get(name) {
                    return Ok(v.clone());
                }
                if self.aliases.contains_key(name) || self.functions.contains_key(name) {
                    return self.call_name(name, &[], 0);
                }
                if let Some(c) = color_named(name) {
                    if self.has_pkg(packages::PKG_GUI) {
                        return Ok(Value::Number(c as f64));
                    }
                }
                Err(YppError::undefined_variable(name))
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
            Expr::Property { target, name, line } => self.eval_property(target, name, *line),
            Expr::CompareExpr { left, op, right } => {
                let l = self.evaluate(left)?;
                let r = self.evaluate(right)?;
                Ok(Value::Bool(compare_values(&l, &r, op)))
            }
            Expr::BinaryExpr { left, op, right } => {
                let l = self.evaluate(left)?;
                let r = self.evaluate(right)?;

                match (l, r) {
                    (Value::Number(n1), Value::Number(n2)) => match op {
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
                    },
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
                self.eval_call_or_construct(class_name, args, *line)
            }
            Expr::MethodCall { target, method_name, args, line } => {
                self.eval_method(target, method_name, args, *line)
            }
        }
    }

    fn eval_property(&mut self, target: &Expr, name: &str, line: usize) -> YppResult<Value> {
        if let Expr::Ident { name: tname } = target {
            if tname.eq_ignore_ascii_case("KEYBOARD") {
                self.require_gui(line, "KEYBOARD")?;
                return Ok(self.globals.get("KEYBOARD").cloned().unwrap_or(Value::Nil));
            }
            if let Some(map) = self.string_blocks.get(tname) {
                if let Some(v) = map.get(name) {
                    return Ok(Value::Str(v.clone()));
                }
            }
            if let Some(map) = self.num_blocks.get(tname) {
                if let Some(v) = map.get(name) {
                    return Ok(Value::Number(*v));
                }
            }
            if let Some(func) = self.functions.get(tname) {
                for stmt in &func.body {
                    if let AstNode::NumBlock { label, .. } = stmt {
                        if let Some(map) = self.num_blocks.get(label) {
                            if let Some(v) = map.get(name) {
                                return Ok(Value::Number(*v));
                            }
                        }
                    }
                    if let AstNode::StringBlock { label, .. } = stmt {
                        if let Some(map) = self.string_blocks.get(label) {
                            if let Some(v) = map.get(name) {
                                return Ok(Value::Str(v.clone()));
                            }
                        }
                    }
                }
            }
            if let Some(Value::Object(ObjectHandle::Ref(id))) = self.globals.get(tname).cloned() {
                if let Some(NetObject::Dimensions { width, height }) = self.objects.get(&id) {
                    match name.to_ascii_lowercase().as_str() {
                        "width" => return Ok(Value::Number(*width)),
                        "length" | "height" => return Ok(Value::Number(*height)),
                        _ => {}
                    }
                }
            }
        }
        let target_val = self.evaluate(target)?;
        if let Value::Object(ObjectHandle::Ref(id)) = target_val {
            if let Some(NetObject::Dimensions { width, height }) = self.objects.get(&id) {
                match name.to_ascii_lowercase().as_str() {
                    "width" => return Ok(Value::Number(*width)),
                    "length" | "height" => return Ok(Value::Number(*height)),
                    _ => {}
                }
            }
        }
        Ok(Value::Nil)
    }

    fn eval_call_or_construct(&mut self, class_name: &str, args: &[Expr], line: usize) -> YppResult<Value> {
        if self.aliases.contains_key(class_name) || self.functions.contains_key(class_name) || packages::is_gui(class_name) {
            return self.call_name(class_name, args, line);
        }
        let class_lower = class_name.to_lowercase();
        if class_lower == "dimensions" {
            return self.builtin_dimensions(args, line);
        }
        if class_lower == "network" {
            self.require_net(line, "Network")?;
            let host = match args.get(0).map(|e| self.evaluate(e)) {
                Some(Ok(Value::Str(s))) => s,
                _ => "127.0.0.1".to_string(),
            };
            let port = match args.get(1).map(|e| self.evaluate(e)) {
                Some(Ok(Value::Number(n))) => port_from_number(n),
                _ => 5000,
            };
            let obj = create_network(&host, port, line)?;
            let id = self.alloc_object(obj, line)?;
            Ok(Value::Object(ObjectHandle::Ref(id)))
        } else if class_lower == "server" {
            self.require_net(line, "Server")?;
            let port = match args.get(0).map(|e| self.evaluate(e)) {
                Some(Ok(Value::Number(n))) => port_from_number(n),
                _ => 5000,
            };
            let obj = create_server(port, line)?;
            let id = self.alloc_object(obj, line)?;
            Ok(Value::Object(ObjectHandle::Ref(id)))
        } else if class_lower == "primitivedatastream" {
            let arg = args.get(0).ok_or_else(|| YppError::unexpected_token(line, "primitivedataStream"))?;
            self.evaluate(arg)
        } else if class_lower == "reader" {
            let obj = create_console_reader();
            let id = self.alloc_object(obj, line)?;
            Ok(Value::Object(ObjectHandle::Ref(id)))
        } else if class_lower == "userinput" {
            Ok(Value::Nil)
        } else {
            Ok(Value::Nil)
        }
    }

    fn eval_method(&mut self, target: &Expr, method_name: &str, args: &[Expr], line: usize) -> YppResult<Value> {
        if let Expr::Ident { name } = target {
            if name == "comp" && method_name == "quit" {
                std::process::exit(0);
            }
            if name.eq_ignore_ascii_case("KEYBOARD") {
                self.require_gui(line, "KEYBOARD")?;
                let key = match args.get(0).map(|e| self.evaluate(e)) {
                    Some(Ok(Value::Str(s))) => s,
                    Some(Ok(other)) => format!("{}", other),
                    _ => String::new(),
                };
                let held = self.gui.as_ref().map(|g| g.key_held(&key)).unwrap_or(false);
                let m = method_name.to_ascii_uppercase();
                if m == "KEYHOLD" || m == "KEYDOWN" || m == "HELD" {
                    if held {
                        return Ok(Value::Str(key));
                    }
                    return Ok(Value::Str(String::new()));
                }
            }
        }

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
                    Ok(Value::Number(s.chars().count() as f64))
                } else if method_name == "readline" || method_name == "readutf-8" {
                    Ok(Value::Str(s))
                } else {
                    Ok(Value::Nil)
                }
            }
            Value::Object(ObjectHandle::Ref(id)) => {
                let method = method_name.to_ascii_lowercase();
                if method == "dimensions" {
                    self.require_gui(line, "frame.dimensions")?;
                    let w = match args.get(0).map(|e| self.evaluate(e)) {
                        Some(Ok(Value::Number(n))) => n.max(1.0) as usize,
                        _ => 60,
                    };
                    let h = match args.get(1).map(|e| self.evaluate(e)) {
                        Some(Ok(Value::Number(n))) => n.max(1.0) as usize,
                        _ => 70,
                    };
                    if let Some(gui) = self.gui.as_mut() {
                        gui.set_dimensions(w, h, line)?;
                    }
                    return Ok(Value::Nil);
                }
                if method == "accept" {
                    let listener = if let Some(NetObject::Server { listener }) = self.objects.get(&id) {
                        listener.try_clone().ok()
                    } else {
                        None
                    };
                    if let Some(l) = listener {
                        let new_obj = server_accept(&l, line)?;
                        let new_id = self.alloc_object(new_obj, line)?;
                        Ok(Value::Object(ObjectHandle::Ref(new_id)))
                    } else {
                        Ok(Value::Nil)
                    }
                } else if method == "outstream" {
                    let stream = match self.objects.get(&id) {
                        Some(NetObject::Client { stream }) => stream.try_clone().ok(),
                        _ => None,
                    };
                    if let Some(s) = stream {
                        let new_obj = get_output_stream(&s, line)?;
                        let new_id = self.alloc_object(new_obj, line)?;
                        Ok(Value::Object(ObjectHandle::Ref(new_id)))
                    } else {
                        Ok(Value::Nil)
                    }
                } else if method == "inputstream" {
                    let stream = match self.objects.get(&id) {
                        Some(NetObject::Client { stream }) => stream.try_clone().ok(),
                        _ => None,
                    };
                    if let Some(s) = stream {
                        let new_obj = get_input_stream(&s, line)?;
                        let new_id = self.alloc_object(new_obj, line)?;
                        Ok(Value::Object(ObjectHandle::Ref(new_id)))
                    } else {
                        Ok(Value::Nil)
                    }
                } else if method == "utf-8" || method == "writeutf-8" || method == "write" {
                    let arg_val = self.evaluate(&args[0])?;
                    let msg = format!("{}", arg_val);
                    if let Some(NetObject::OutputStream { writer }) = self.objects.get_mut(&id) {
                        stream_write(writer, &msg, line)?;
                    }
                    Ok(Value::Nil)
                } else if method == "readutf-8" || method == "readline" || method == "read" {
                    let mut is_console = false;
                    if let Some(NetObject::ConsoleReader) = self.objects.get(&id) {
                        is_console = true;
                    }
                    if is_console {
                        let input = self.input_provider.read_line("");
                        Ok(Value::Str(input))
                    } else if let Some(NetObject::InputStream { reader }) = self.objects.get_mut(&id) {
                        let line_str = stream_read(reader, line)?;
                        Ok(Value::Str(line_str))
                    } else {
                        Ok(Value::Nil)
                    }
                } else if method == "close" {
                    self.objects.remove(&id);
                    Ok(Value::Nil)
                } else if method == "keyhold" || method == "keydown" {
                    self.require_gui(line, "KEYBOARD")?;
                    let key = match args.get(0).map(|e| self.evaluate(e)) {
                        Some(Ok(Value::Str(s))) => s,
                        Some(Ok(other)) => format!("{}", other),
                        _ => String::new(),
                    };
                    let held = self.gui.as_ref().map(|g| g.key_held(&key)).unwrap_or(false);
                    if held {
                        Ok(Value::Str(key))
                    } else {
                        Ok(Value::Str(String::new()))
                    }
                } else {
                    Ok(Value::Nil)
                }
            }
            _ => Ok(Value::Nil),
        }
    }
}

fn port_from_number(n: f64) -> u16 {
    if !n.is_finite() {
        return 0;
    }
    n as i64 as u16
}

fn compare_values(l: &Value, r: &Value, op: &CmpOp) -> bool {
    match (l, r) {
        (Value::Number(a), Value::Number(b)) => match op {
            CmpOp::Eq => a == b,
            CmpOp::Ne => a != b,
            CmpOp::Lt => a < b,
            CmpOp::Gt => a > b,
            CmpOp::Le => a <= b,
            CmpOp::Ge => a >= b,
        },
        (Value::Str(a), Value::Str(b)) => match op {
            CmpOp::Eq => a == b,
            CmpOp::Ne => a != b,
            CmpOp::Lt => a < b,
            CmpOp::Gt => a > b,
            CmpOp::Le => a <= b,
            CmpOp::Ge => a >= b,
        },
        (Value::Bool(a), Value::Bool(b)) => match op {
            CmpOp::Eq => a == b,
            CmpOp::Ne => a != b,
            _ => false,
        },
        _ => match op {
            CmpOp::Eq => format!("{}", l) == format!("{}", r),
            CmpOp::Ne => format!("{}", l) != format!("{}", r),
            _ => false,
        },
    }
}
