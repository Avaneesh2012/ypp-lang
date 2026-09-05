/// Error types for Y++ — exact error message formats matching Java YppException.
use std::fmt;

#[derive(Debug)]
pub struct YppError {
    pub message: String,
}

impl YppError {
    pub fn new(msg: impl Into<String>) -> Self {
        YppError { message: msg.into() }
    }

    // --- Missing import errors ---

    pub fn missing_ycomponents() -> Self {
        Self::new("Import ycomponents is required to run Y++ code.")
    }

    pub fn missing_ynetworking(line: usize, class_name: &str) -> Self {
        Self::new(format!("Line {}: Import ynetworking is required to use {}.", line, class_name))
    }

    pub fn missing_ygui(line: usize, feature: &str) -> Self {
        Self::new(format!("Line {}: Import yGUI is required to use {}.", line, feature))
    }

    pub fn unknown_package(line: usize, pkg: &str) -> Self {
        Self::new(format!(
            "Line {}: unknown package '{}'. Built-in packages are ycomponents, ynetworking, and yGUI.",
            line, pkg
        ))
    }

    // --- Type bounds errors ---

    pub fn smallint_out_of_range(line: usize, val: f64) -> Self {
        Self::new(format!("Line {}: smallint value {} out of range [-1000, 1000]", line, val as i64))
    }

    pub fn integer_out_of_range(line: usize, val: f64) -> Self {
        Self::new(format!("Line {}: integer value {} out of range [-1000000000000000, 1000000000000000]", line, val as i64))
    }

    pub fn double_out_of_range(line: usize, val: f64) -> Self {
        Self::new(format!("Line {}: double value {} out of range [-5677719218.1092, 5677719218.1092]", line, val))
    }

    pub fn schar_too_long(line: usize, name: &str, val: &str) -> Self {
        Self::new(format!("Line {}: schar '{}' can only hold a single character, but got \"{}\"", line, name, val))
    }

    pub fn param_expected_number(line: usize, type_name: &str, name: &str, val: &str) -> Self {
        Self::new(format!("Line {}: {} parameter '{}' expected a number, but got \"{}\"", line, type_name, name, val))
    }

    // --- Type mismatch ---

    pub fn concat_type_error() -> Self {
        Self::new("Type error: string and number concatenation requires EXCEPTION CONCAT")
    }

    // --- Undefined entities ---

    pub fn undefined_variable(name: &str) -> Self {
        Self::new(format!("Undefined variable: {}", name))
    }

    pub fn undefined_num_block(label: &str) -> Self {
        Self::new(format!("Undefined NUM block: {}", label))
    }

    pub fn undefined_string_block(label: &str) -> Self {
        Self::new(format!("Undefined STRING block: {}", label))
    }

    pub fn undefined_var_in_block(name: &str, label: &str) -> Self {
        Self::new(format!("Undefined variable '{}' in block {}", name, label))
    }

    pub fn undefined_func_or_alias(line: usize, name: &str) -> Self {
        Self::new(format!("Line {}: Undefined function or alias '{}'", line, name))
    }

    pub fn function_already_defined(name: &str) -> Self {
        Self::new(format!("Function already defined: {}", name))
    }

    // --- Syntax / parse errors ---

    pub fn expected_token(line: usize, expected: &str, got_type: &str, got_val: &str) -> Self {
        Self::new(format!("Line {}: expected {} but got {} ('{}')", line, expected, got_type, got_val))
    }

    pub fn missing_semicolon_input(line: usize, param: &str) -> Self {
        Self::new(format!("Line {}: missing ';' after {}.input(...) — semicolons are required on input calls.", line, param))
    }

    pub fn missing_semicolon_next(line: usize, param: &str) -> Self {
        Self::new(format!("Line {}: missing ';' after {}.next() — semicolons are required.", line, param))
    }

    pub fn missing_semicolon_break(line: usize, param: &str) -> Self {
        Self::new(format!("Line {}: missing ';' after {}.break — semicolons are required.", line, param))
    }

    pub fn unexpected_token(line: usize, val: &str) -> Self {
        Self::new(format!("Line {}: unexpected token '{}' in expression", line, val))
    }

    pub fn expected_literal(line: usize, val: &str) -> Self {
        Self::new(format!("Line {}: expected literal value, got '{}'", line, val))
    }

    // --- Networking errors ---

    pub fn network_connect_failed(line: usize, host: &str, port: u16, msg: &str) -> Self {
        Self::new(format!("Line {}: Network connection to {}:{} failed: {}", line, host, port, msg))
    }

    pub fn server_failed(line: usize, port: u16, msg: &str) -> Self {
        Self::new(format!("Line {}: Server failed on port {}: {}", line, port, msg))
    }

    pub fn stream_error(line: usize, method: &str, msg: &str) -> Self {
        Self::new(format!("Line {}: Stream error in {}: {}", line, method, msg))
    }

    pub fn invalid_network_host(line: usize, host: &str) -> Self {
        Self::new(format!("Line {}: invalid network host '{}'", line, host))
    }

    pub fn invalid_network_port(line: usize, port: u16) -> Self {
        Self::new(format!("Line {}: invalid network port {} (must be 1–65535)", line, port))
    }

    pub fn object_limit_reached(line: usize) -> Self {
        Self::new(format!("Line {}: runtime object limit reached", line))
    }

    // --- GUI errors ---

    pub fn gui_window_error(line: usize, msg: &str) -> Self {
        Self::new(format!("Line {}: yGUI window error: {}", line, msg))
    }

    pub fn gui_size_error(line: usize, w: usize, h: usize) -> Self {
        Self::new(format!(
            "Line {}: yGUI frame size {}x{} is too large (max 256x256 cells)",
            line, w, h
        ))
    }

    // --- Runtime errors ---

    pub fn division_by_zero() -> Self {
        Self::new("Division by zero")
    }
}

impl fmt::Display for YppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for YppError {}

pub type YppResult<T> = Result<T, YppError>;
