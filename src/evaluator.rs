use crate::{Environment, Rule, Value, parser::parse_value};
use pest::iterators::Pair;

/// Evaluate expressions with support for concatenation, arithmetic, and comparisons
pub fn evaluate_expression(pair: Pair<Rule>, env: &Environment) -> Value {
    match pair.as_rule() {
        Rule::EXPRESSION => evaluate_concat_expr(pair, env),
        Rule::CONCAT_EXPR => evaluate_concat_expr(pair, env),
        Rule::COMPARISON => evaluate_comparison(pair, env),
        Rule::ADD_EXPR => evaluate_additive(pair, env),
        Rule::MULT_EXPR => evaluate_multiplicative(pair, env),
        Rule::PRIMARY => evaluate_primary(pair, env),
        _ => parse_value(pair, env),
    }
}

/// Evaluate concatenation expressions (string concatenation)
fn evaluate_concat_expr(pair: Pair<Rule>, env: &Environment) -> Value {
    let mut parts = Vec::new();
    let mut has_string = false;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::CONCAT_OP => continue,
            Rule::COMPARISON | Rule::ADD_EXPR | Rule::MULT_EXPR | Rule::TERM => {
                let value = evaluate_expression(inner, env);

                match &value {
                    Value::String(_) => has_string = true,
                    _ => {}
                }

                parts.push(value);
            }
            _ => {
                let value = evaluate_expression(inner, env);
                parts.push(value);
            }
        }
    }

    // If any part is a string, concatenate all as strings
    if has_string || parts.is_empty() {
        let result = parts
            .iter()
            .map(|v| match v {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Boolean(b) => b.to_string(),
                Value::Function(f) => format!("<function {}>", f.name),
                Value::Void => String::new(),
            })
            .collect::<String>();
        Value::String(result)
    } else if parts.len() == 1 {
        parts.into_iter().next().unwrap()
    } else {
        // Multiple numeric values without explicit operator - treat as string concat
        let result = parts
            .iter()
            .map(|v| match v {
                Value::Number(n) => n.to_string(),
                Value::String(s) => s.clone(),
                Value::Boolean(b) => b.to_string(),
                Value::Function(f) => format!("<function {}>", f.name),
                Value::Void => String::new(),
            })
            .collect::<String>();
        Value::String(result)
    }
}


/// Evaluate additive expressions (+ and -)
fn evaluate_additive(pair: Pair<Rule>, env: &Environment) -> Value {
    let mut inner = pair.into_inner();
    let mut result = evaluate_expression(inner.next().unwrap(), env);

    while let Some(next) = inner.next() {
        match next.as_rule() {
            Rule::ADD_OP | Rule::SUB_OP => {
                let operator = next.as_str();
                let right = evaluate_expression(inner.next().unwrap(), env);
                result = apply_arithmetic_op(operator, result, right);
            }
            _ => {
                result = evaluate_expression(next, env);
            }
        }
    }

    result
}

/// Evaluate multiplicative expressions (* and /)
fn evaluate_multiplicative(pair: Pair<Rule>, env: &Environment) -> Value {
    let mut inner = pair.into_inner();
    let mut result = evaluate_expression(inner.next().unwrap(), env);

    while let Some(next) = inner.next() {
        match next.as_rule() {
            Rule::MUL_OP | Rule::DIV_OP => {
                let operator = next.as_str();
                let right = evaluate_expression(inner.next().unwrap(), env);
                result = apply_arithmetic_op(operator, result, right);
            }
            _ => {
                result = evaluate_expression(next, env);
            }
        }
    }

    result
}

/// Evaluate comparison expressions
fn evaluate_comparison(pair: Pair<Rule>, env: &Environment) -> Value {
    let mut inner = pair.into_inner();
    let left = evaluate_expression(inner.next().unwrap(), env);

    if let Some(op_pair) = inner.next() {
        if matches!(
            op_pair.as_rule(),
            Rule::EQ_OP | Rule::NEQ_OP | Rule::LT_OP | Rule::GT_OP
        ) {
            let operator = op_pair.as_str();
            let right = evaluate_expression(inner.next().unwrap(), env);
            return apply_comparison_op(operator, left, right);
        }
    }

    left
}

/// Evaluate primary expressions (literals, variables, parenthesized expressions)
fn evaluate_primary(pair: Pair<Rule>, env: &Environment) -> Value {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::NUMBER => parse_number(inner.as_str()),
        Rule::STRING => {
            let s = inner.as_str();
            // Remove delimiters (", ⟦⟧, ⟨⟩)
            let content = if s.starts_with('"') && s.ends_with('"') {
                &s[1..s.len() - 1]
            } else if s.starts_with('⟦') && s.ends_with('⟧') {
                &s[3..s.len() - 3] // UTF-8 multibyte
            } else if s.starts_with('⟨') && s.ends_with('⟩') {
                &s[3..s.len() - 3]
            } else {
                s
            };
            Value::String(content.to_string())
        }
        Rule::VAR_NAME => {
            let var_name = inner.as_str();
            env.get(var_name)
                .unwrap_or_else(|| panic!("Undefined variable: {}", var_name))
        }
        Rule::OPERATOR_SYMBOL => {
            // Single operator as literal value
            Value::Number(parse_operator_literal(inner.as_str()))
        }
        Rule::EXPRESSION
        | Rule::CONCAT_EXPR
        | Rule::COMPARISON
        | Rule::ADD_EXPR
        | Rule::MULT_EXPR => evaluate_expression(inner, env),
        _ => parse_value(inner, env),
    }
}

/// Parse numbers (ASCII, Chinese, Alien, Operator-based)
fn parse_number(s: &str) -> Value {
    // Try ASCII number
    if let Ok(n) = s.parse::<i64>() {
        return Value::Number(n);
    }

    // Check for repeated operator numbers (⊕⊕⊕⊕⊕ = 5)
    if let Some(first_char) = s.chars().next() {
        if "⊕⊗⊘⊚⊙⊞⊟⊠⨁⨂⨸∀∃∄∅".contains(first_char) {
            let count = s.chars().take_while(|&c| c == first_char).count();

            if count == s.chars().count() {
                return Value::Number(count as i64);
            }
        }
    }

    // Alien digit strings (∅∄∃∀℧℥℞℟℣℈)
    if s.chars().all(|c| "∅∄∃∀℧℥℞℟℣℈".contains(c)) {
        let mut result = 0i64;
        for ch in s.chars() {
            let digit = match ch {
                '∅' => 0,
                '∄' => 1,
                '∃' => 2,
                '∀' => 3,
                '℧' => 4,
                '℥' => 5,
                '℞' => 6,
                '℟' => 7,
                '℣' => 8,
                '℈' => 9,
                _ => 0,
            };
            result = result * 10 + digit;
        }
        return Value::Number(result);
    }

    // Chinese numbers
    Value::Number(parse_chinese_number(s))
}

/// Parse Chinese numerals to i64
fn parse_chinese_number(s: &str) -> i64 {
    let mut result = 0i64;
    let mut current = 0i64;
    let mut has_digit = false;

    for ch in s.chars() {
        match ch {
            '零' | '〇' => {
                has_digit = true;
                current = 0;
            }
            '一' => {
                has_digit = true;
                current = 1;
            }
            '二' => {
                has_digit = true;
                current = 2;
            }
            '三' => {
                has_digit = true;
                current = 3;
            }
            '四' => {
                has_digit = true;
                current = 4;
            }
            '五' => {
                has_digit = true;
                current = 5;
            }
            '六' => {
                has_digit = true;
                current = 6;
            }
            '七' => {
                has_digit = true;
                current = 7;
            }
            '八' => {
                has_digit = true;
                current = 8;
            }
            '九' => {
                has_digit = true;
                current = 9;
            }
            '十' => {
                if !has_digit {
                    current = 1;
                }
                result += current * 10;
                current = 0;
                has_digit = false;
            }
            '百' => {
                if !has_digit {
                    current = 1;
                }
                result += current * 100;
                current = 0;
                has_digit = false;
            }
            '千' => {
                if !has_digit {
                    current = 1;
                }
                result += current * 1000;
                current = 0;
                has_digit = false;
            }
            '万' => {
                if !has_digit {
                    current = 1;
                }
                result = (result + current) * 10000;
                current = 0;
                has_digit = false;
            }
            _ => {}
        }
    }
    result + current
}

/// Parse single operator symbols as numeric values
fn parse_operator_literal(op: &str) -> i64 {
    match op {
        "⊕" => 1,
        "⊗" => 2,
        "⊘" => 0,
        "⊚" => 10,
        "⊙" => 5,
        "⊞" => 1,
        "⊟" => 0,
        "⊠" => 2,
        "⨁" => 1,
        "⨂" => 0,
        "⨸" => 0,
        "∀" => 3,
        "∃" => 2,
        "∄" => 1,
        "∅" => 0,
        _ => 0,
    }
}

/// Apply arithmetic operations with alien operator support
fn apply_arithmetic_op(operator: &str, left: Value, right: Value) -> Value {
    let left_num = match left {
        Value::Number(n) => n,
        Value::String(s) => s.parse().unwrap_or(0),
        Value::Boolean(b) => {
            if b {
                1
            } else {
                0
            }
        }
        Value::Function(_) => panic!("Cannot use function in arithmetic"),
        Value::Void => 0,
    };

    let right_num = match right {
        Value::Number(n) => n,
        Value::String(s) => s.parse().unwrap_or(0),
        Value::Boolean(b) => {
            if b {
                1
            } else {
                0
            }
        }
        Value::Function(_) => panic!("Cannot use function in arithmetic"),
        Value::Void => 0,
    };

    let result = match operator {
        "+" | "⊕" | "⊞" | "⨁" => left_num + right_num,
        "-" | "⊟" | "⨂" => left_num - right_num,
        "*" | "⊗" | "⊠" => left_num * right_num,
        "/" | "⊘" | "⨸" => {
            if right_num == 0 {
                panic!("Division by zero");
            }
            left_num / right_num
        }
        "%" => {
            if right_num == 0 {
                panic!("Modulo by zero");
            }
            left_num % right_num
        }
        _ => panic!("Unknown arithmetic operator: {}", operator),
    };

    Value::Number(result)
}

/// Apply comparison operations with alien operator support
fn apply_comparison_op(operator: &str, left: Value, right: Value) -> Value {
    let result = match (&left, &right) {
        (Value::Number(l), Value::Number(r)) => match operator {
            "==" | "⊙" | "≡" => l == r,
            "!=" | "⊗" | "≢" => l != r,
            "<" | "◁" | "⊲" => l < r,
            ">" | "▷" | "⊳" => l > r,
            "<=" => l <= r,
            ">=" => l >= r,
            _ => panic!("Unknown comparison operator: {}", operator),
        },
        (Value::String(l), Value::String(r)) => match operator {
            "==" | "⊙" | "≡" => l == r,
            "!=" | "⊗" | "≢" => l != r,
            "<" | "◁" | "⊲" => l < r,
            ">" | "▷" | "⊳" => l > r,
            "<=" => l <= r,
            ">=" => l >= r,
            _ => panic!("Unknown comparison operator: {}", operator),
        },
        _ => {
            // Try to convert both to numbers
            let left_num = match left {
                Value::Number(n) => n,
                Value::String(s) => s.parse().unwrap_or(0),
                Value::Boolean(b) => {
                    if b {
                        1
                    } else {
                        0
                    }
                }
                Value::Function(_) => panic!("Cannot use function in comparison"),
                Value::Void => 0,
            };

            let right_num = match right {
                Value::Number(n) => n,
                Value::String(s) => s.parse().unwrap_or(0),
                Value::Boolean(b) => {
                    if b {
                        1
                    } else {
                        0
                    }
                }
                Value::Function(_) => panic!("Cannot use function in comparison"),
                Value::Void => 0,
            };

            match operator {
                "==" | "⊙" | "≡" => left_num == right_num,
                "!=" | "⊗" | "≢" => left_num != right_num,
                "<" | "◁" | "⊲" => left_num < right_num,
                ">" | "▷" | "⊳" => left_num > right_num,
                "<=" => left_num <= right_num,
                ">=" => left_num >= right_num,
                _ => panic!("Unknown comparison operator: {}", operator),
            }
        }
    };

    Value::Boolean(result)
}

/// Evaluate a term (for backward compatibility)
pub fn evaluate_term(pair: Pair<Rule>, env: &Environment) -> Value {
    match pair.as_rule() {
        Rule::NUMBER => parse_number(pair.as_str()),
        Rule::STRING => {
            let s = pair.as_str();
            let content = &s[1..s.len() - 1]; // Remove quotes
            Value::String(content.to_string())
        }
        Rule::VAR_NAME => {
            let var_name = pair.as_str();
            env.get(var_name)
                .unwrap_or_else(|| panic!("Undefined variable: {}", var_name))
        }
        _ => evaluate_expression(pair, env),
    }
}
