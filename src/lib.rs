use pest_derive::Parser;

pub mod codegen;
pub mod evaluator;
pub mod executor;
pub mod ling_number;
pub mod parser;

// Re-export commonly used types
pub use environment::Environment;
pub use error::{LingError, LingResult};
pub use value::Value;

/// Main parser for the alien/ling language
#[derive(Parser)]
#[grammar = "ling_lang.pest"]
pub struct LingParser;

// Deprecated alias for backward compatibility
pub type ChineseLangParser = LingParser;

/// Module for value types
pub mod value {
    use std::fmt;

    /// Runtime value types in the language
    #[derive(Clone, Debug, PartialEq)]
    pub enum Value {
        Number(i64),
        String(String),
        Boolean(bool),
        Function(FunctionValue),
        Void,
    }

    /// Function value representation
    #[derive(Clone, Debug, PartialEq)]
    pub struct FunctionValue {
        pub name: String,
        pub params: Vec<String>,
        pub body: String, // Store as AST later
    }

    impl Value {
        /// Check if value is truthy (for conditionals)
        pub fn is_truthy(&self) -> bool {
            match self {
                Value::Boolean(b) => *b,
                Value::Number(n) => *n != 0,
                Value::String(s) => !s.is_empty(),
                Value::Void => false,
                Value::Function(_) => true,
            }
        }

        /// Convert value to number if possible
        pub fn to_number(&self) -> Option<i64> {
            match self {
                Value::Number(n) => Some(*n),
                Value::String(s) => s.parse().ok(),
                Value::Boolean(b) => Some(if *b { 1 } else { 0 }),
                _ => None,
            }
        }

        /// Get the type name of the value
        pub fn type_name(&self) -> &'static str {
            match self {
                Value::Number(_) => "number",
                Value::String(_) => "string",
                Value::Boolean(_) => "boolean",
                Value::Function(_) => "function",
                Value::Void => "void",
            }
        }
    }

    impl fmt::Display for Value {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Value::Number(n) => write!(f, "{}", n),
                Value::String(s) => write!(f, "{}", s),
                Value::Boolean(b) => write!(f, "{}", b),
                Value::Function(func) => write!(f, "<function {}>", func.name),
                Value::Void => write!(f, ""),
            }
        }
    }

    impl From<i64> for Value {
        fn from(n: i64) -> Self {
            Value::Number(n)
        }
    }

    impl From<String> for Value {
        fn from(s: String) -> Self {
            Value::String(s)
        }
    }

    impl From<&str> for Value {
        fn from(s: &str) -> Self {
            Value::String(s.to_string())
        }
    }

    impl From<bool> for Value {
        fn from(b: bool) -> Self {
            Value::Boolean(b)
        }
    }
}

/// Module for environment/scope management
pub mod environment {
    use super::value::Value;
    use std::collections::HashMap;

    /// Runtime environment to store variables and scopes
    #[derive(Clone, Debug)]
    pub struct Environment {
        scopes: Vec<Scope>,
    }

    /// A single scope containing variables
    #[derive(Clone, Debug)]
    struct Scope {
        variables: HashMap<String, Variable>,
    }

    /// Variable metadata
    #[derive(Clone, Debug)]
    struct Variable {
        value: Value,
        is_mutable: bool,
        normalized_name: String, // ASCII-safe name for alien identifiers
    }

    impl Environment {
        /// Create a new environment with global scope
        pub fn new() -> Self {
            Environment {
                scopes: vec![Scope::new()],
            }
        }

        /// Push a new scope (for functions, blocks)
        pub fn push_scope(&mut self) {
            self.scopes.push(Scope::new());
        }

        /// Pop the current scope
        pub fn pop_scope(&mut self) {
            if self.scopes.len() > 1 {
                self.scopes.pop();
            }
        }

        /// Get the current scope depth
        pub fn scope_depth(&self) -> usize {
            self.scopes.len()
        }

        /// Set a variable in the current scope
        pub fn set(&mut self, name: String, value: Value) {
            self.set_with_mutability(name, value, true);
        }

        /// Set a variable with mutability flag
        pub fn set_with_mutability(&mut self, name: String, value: Value, is_mutable: bool) {
            let normalized = Self::normalize_identifier(&name);

            if let Some(scope) = self.scopes.last_mut() {
                scope.variables.insert(
                    name.clone(),
                    Variable {
                        value,
                        is_mutable,
                        normalized_name: normalized,
                    },
                );
            }
        }

        /// Set a constant (immutable variable)
        pub fn set_const(&mut self, name: String, value: Value) {
            self.set_with_mutability(name, value, false);
        }

        /// Update an existing variable
        pub fn update(&mut self, name: &str, value: Value) -> Result<(), String> {
            // Search from innermost to outermost scope
            for scope in self.scopes.iter_mut().rev() {
                if let Some(var) = scope.variables.get_mut(name) {
                    if !var.is_mutable {
                        return Err(format!("Cannot assign to immutable variable '{}'", name));
                    }
                    var.value = value;
                    return Ok(());
                }
            }
            Err(format!("Undefined variable '{}'", name))
        }

        /// Get a variable value
        pub fn get(&self, name: &str) -> Option<Value> {
            // Search from innermost to outermost scope
            for scope in self.scopes.iter().rev() {
                if let Some(var) = scope.variables.get(name) {
                    return Some(var.value.clone());
                }
            }
            None
        }

        /// Check if a variable exists
        pub fn exists(&self, name: &str) -> bool {
            self.scopes
                .iter()
                .rev()
                .any(|scope| scope.variables.contains_key(name))
        }

        /// Check if a variable is mutable
        pub fn is_mutable(&self, name: &str) -> Option<bool> {
            for scope in self.scopes.iter().rev() {
                if let Some(var) = scope.variables.get(name) {
                    return Some(var.is_mutable);
                }
            }
            None
        }

        /// Get the normalized (ASCII-safe) name of a variable
        pub fn get_normalized_name(&self, name: &str) -> Option<String> {
            for scope in self.scopes.iter().rev() {
                if let Some(var) = scope.variables.get(name) {
                    return Some(var.normalized_name.clone());
                }
            }
            None
        }

        /// Get all variables in the current scope
        pub fn current_scope_vars(&self) -> Vec<String> {
            self.scopes
                .last()
                .map(|scope| scope.variables.keys().cloned().collect())
                .unwrap_or_default()
        }

        /// Get all variables across all scopes
        pub fn all_vars(&self) -> Vec<String> {
            let mut vars = Vec::new();
            for scope in &self.scopes {
                vars.extend(scope.variables.keys().cloned());
            }
            vars
        }

        /// Clear all variables in the current scope
        pub fn clear_current_scope(&mut self) {
            if let Some(scope) = self.scopes.last_mut() {
                scope.variables.clear();
            }
        }

        /// Clear all variables in all scopes
        pub fn clear_all(&mut self) {
            self.scopes.clear();
            self.scopes.push(Scope::new());
        }

        /// Normalize alien Unicode identifiers to ASCII-safe names
        fn normalize_identifier(name: &str) -> String {
            let mut result = String::new();

            for ch in name.chars() {
                if ch.is_ascii_alphanumeric() || ch == '_' {
                    result.push(ch);
                } else {
                    // Convert Unicode to hex representation
                    result.push_str(&format!("_U{:04X}_", ch as u32));
                }
            }

            // Ensure valid identifier (doesn't start with digit)
            if result.is_empty() || result.chars().next().unwrap().is_ascii_digit() {
                result.insert(0, '_');
            }

            result
        }
    }

    impl Scope {
        fn new() -> Self {
            Scope {
                variables: HashMap::new(),
            }
        }
    }

    impl Default for Environment {
        fn default() -> Self {
            Self::new()
        }
    }

    // Backward compatibility methods
    impl Environment {
        #[deprecated(note = "Use set() instead")]
        pub fn set_var(&mut self, name: String, value: Value) {
            self.set(name, value);
        }

        #[deprecated(note = "Use get() instead")]
        pub fn get_var(&self, _name: &str) -> Option<&Value> {
            // This requires returning a reference, which doesn't work with our clone approach
            // Users should migrate to get() which returns Option<Value>
            unimplemented!("Use get() instead, which returns Option<Value>")
        }
    }
}

/// Module for error handling
pub mod error {
    use std::fmt;

    /// Result type for language operations
    pub type LingResult<T> = Result<T, LingError>;

    /// Error types in the language
    #[derive(Debug, Clone, PartialEq)]
    pub enum LingError {
        ParseError(String),
        RuntimeError(String),
        TypeError {
            expected: String,
            found: String,
        },
        UndefinedVariable(String),
        ImmutableAssignment(String),
        DivisionByZero,
        InvalidOperation {
            op: String,
            left: String,
            right: String,
        },
        FunctionNotFound(String),
        ArgumentMismatch {
            expected: usize,
            found: usize,
        },
        CompilationError(String),
        IOError(String),
    }

    impl fmt::Display for LingError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                LingError::ParseError(msg) => write!(f, "Parse error: {}", msg),
                LingError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
                LingError::TypeError { expected, found } => {
                    write!(f, "Type error: expected {}, found {}", expected, found)
                }
                LingError::UndefinedVariable(name) => {
                    write!(f, "Undefined variable: {}", name)
                }
                LingError::ImmutableAssignment(name) => {
                    write!(f, "Cannot assign to immutable variable: {}", name)
                }
                LingError::DivisionByZero => write!(f, "Division by zero"),
                LingError::InvalidOperation { op, left, right } => {
                    write!(f, "Invalid operation: {} {} {}", left, op, right)
                }
                LingError::FunctionNotFound(name) => {
                    write!(f, "Function not found: {}", name)
                }
                LingError::ArgumentMismatch { expected, found } => {
                    write!(
                        f,
                        "Argument mismatch: expected {} arguments, found {}",
                        expected, found
                    )
                }
                LingError::CompilationError(msg) => write!(f, "Compilation error: {}", msg),
                LingError::IOError(msg) => write!(f, "IO error: {}", msg),
            }
        }
    }

    impl std::error::Error for LingError {}
}

/// Configuration for the language runtime
#[derive(Debug, Clone)]
pub struct LingConfig {
    pub debug_mode: bool,
    pub strict_mode: bool,
    pub max_recursion_depth: usize,
    pub enable_alien_syntax: bool,
}

impl Default for LingConfig {
    fn default() -> Self {
        LingConfig {
            debug_mode: false,
            strict_mode: false,
            max_recursion_depth: 1000,
            enable_alien_syntax: true,
        }
    }
}

/// Utility functions for the language
pub mod utils {
    use super::Value;

    /// Check if two values are equal
    pub fn values_equal(left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Number(l), Value::Number(r)) => l == r,
            (Value::String(l), Value::String(r)) => l == r,
            (Value::Boolean(l), Value::Boolean(r)) => l == r,
            (Value::Void, Value::Void) => true,
            _ => false,
        }
    }

    /// Convert alien operator to ASCII equivalent
    pub fn normalize_operator(op: &str) -> &str {
        match op {
            "⇐" | "⟸" => "=",
            "⊕" | "⊞" | "⨁" => "+",
            "⊟" | "⨂" => "-",
            "⊗" | "⊠" => "*",
            "⊘" | "⨸" => "/",
            "⊙" | "≡" => "==",
            "≢" => "!=",
            "◁" | "⊲" => "<",
            "▷" | "⊳" => ">",
            _ => op,
        }
    }

    /// Check if a string is an alien operator
    pub fn is_alien_operator(s: &str) -> bool {
        matches!(
            s,
            "⇐" | "⟸"
                | "⊕"
                | "⊞"
                | "⨁"
                | "⊟"
                | "⨂"
                | "⊗"
                | "⊠"
                | "⊘"
                | "⨸"
                | "⊙"
                | "≡"
                | "≢"
                | "◁"
                | "⊲"
                | "▷"
                | "⊳"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_basic() {
        let mut env = Environment::new();
        env.set("x".to_string(), Value::Number(42));
        assert_eq!(env.get("x"), Some(Value::Number(42)));
    }

    #[test]
    fn test_environment_scopes() {
        let mut env = Environment::new();
        env.set("x".to_string(), Value::Number(1));

        env.push_scope();
        env.set("x".to_string(), Value::Number(2));
        assert_eq!(env.get("x"), Some(Value::Number(2)));

        env.pop_scope();
        assert_eq!(env.get("x"), Some(Value::Number(1)));
    }

    #[test]
    fn test_value_conversions() {
        let num: Value = 42.into();
        assert_eq!(num, Value::Number(42));

        let str_val: Value = "hello".into();
        assert_eq!(str_val, Value::String("hello".to_string()));

        let bool_val: Value = true.into();
        assert_eq!(bool_val, Value::Boolean(true));
    }

    #[test]
    fn test_alien_identifier_normalization() {
        let mut env = Environment::new();
        env.set("变量".to_string(), Value::Number(123));

        let normalized = env.get_normalized_name("变量").unwrap();
        assert!(normalized.starts_with('_'));
        assert!(normalized.contains("U53D8")); // Unicode code point for 变
    }

    #[test]
    fn test_immutable_variables() {
        let mut env = Environment::new();
        env.set_const("pi".to_string(), Value::Number(314));

        let result = env.update("pi", Value::Number(315));
        assert!(result.is_err());
    }
}
