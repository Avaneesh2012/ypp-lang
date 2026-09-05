/// Runtime value representation for Y++.
use std::fmt;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Str(String),
    Bool(bool),
    /// Opaque object handle (networking objects, streams, readers, GUI).
    Object(ObjectHandle),
    List(Vec<Value>),
    Nil,
}

/// Handle types for Y++ runtime objects.
#[derive(Debug, Clone)]
pub enum ObjectHandle {
    /// Placeholder — actual network objects are stored in Interpreter's object table
    /// and referenced by ID.
    Ref(usize),
}

impl Value {
    /// Y++ truthiness rules (matches Java Interpreter):
    /// - Nil -> false
    /// - Bool -> its value
    /// - Number -> value != 0
    /// - String -> !empty && != "false" && != "null"
    /// - Object -> true
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::Str(s) => !s.is_empty() && !s.eq_ignore_ascii_case("false") && !s.eq_ignore_ascii_case("null"),
            Value::Object(_) => true,
            Value::List(items) => !items.is_empty(),
        }
    }

    /// Check if a number value is an exact integer (no fractional part).
    pub fn is_integer_valued(&self) -> bool {
        match self {
            Value::Number(n) => n.fract() == 0.0 && n.is_finite(),
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => {
                if n.fract() == 0.0 && n.is_finite() {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", format_double(*n))
                }
            }
            Value::Str(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Object(h) => write!(f, "<object {:?}>", h),
            Value::List(items) => {
                let inner: Vec<String> = items.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", inner.join(", "))
            }
            Value::Nil => write!(f, "nil"),
        }
    }
}

/// Format a double to 15 significant figures, strip trailing zeros,
/// ensure at least one decimal place (e.g. "2.0" not "2").
pub fn format_double(val: f64) -> String {
    let s = format!("{:.15}", val);
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    // Ensure there is a decimal point
    if !s.contains('.') {
        format!("{}.0", s)
    } else {
        s.to_string()
    }
}
