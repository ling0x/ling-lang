use crate::{Environment, LingParser, Rule, Value};
use pest::Parser;
use pest::iterators::Pair;

pub fn parse_program(input: &str) -> Result<Vec<Statement>, String> {
    let pairs =
        LingParser::parse(Rule::PROGRAM, input).map_err(|e| format!("Parse error: {}", e))?;

    let mut statements = Vec::new();

    for pair in pairs {
        if pair.as_rule() == Rule::PROGRAM {
            for inner in pair.into_inner() {
                if inner.as_rule() == Rule::STATEMENT {
                    statements.push(parse_statement(inner)?);
                }
            }
        }
    }

    Ok(statements)
}

#[derive(Debug, Clone)]
pub enum Statement {
    VarDecl {
        name: String,
        value: Expression,
    },
    Print {
        expr: Expression,
    },
    FuncDef {
        name: String,
        params: Vec<String>,
        body: Vec<Statement>,
    },
    Return {
        expr: Option<Expression>,
    },
    If {
        condition: Expression,
        then_block: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
    },
}

#[derive(Debug, Clone)]
pub enum Expression {
    Number(i64),
    String(String),
    Variable(String),
    BinaryOp {
        op: String,
        left: Box<Expression>,
        right: Box<Expression>,
    },
}

pub fn parse_statement(pair: Pair<Rule>) -> Result<Statement, String> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::VAR_DECL => parse_var_decl(inner),
        Rule::PRINT_STMT => parse_print_stmt(inner),
        Rule::FUNC_DEF => parse_func_def(inner),
        Rule::RETURN_STMT => parse_return_stmt(inner),
        Rule::IF_STMT | Rule::ALIEN_IF_STMT => parse_if_stmt(inner),
        _ => Err(format!("Unknown statement: {:?}", inner.as_rule())),
    }
}

fn parse_var_decl(pair: Pair<Rule>) -> Result<Statement, String> {
    let mut inner = pair.into_inner();
    inner.next(); // Skip LET_KW

    let name = inner.next().unwrap().as_str().to_string();
    inner.next(); // Skip ASSIGN_OP
    let value = parse_expression(inner.next().unwrap())?;

    Ok(Statement::VarDecl { name, value })
}

fn parse_print_stmt(pair: Pair<Rule>) -> Result<Statement, String> {
    let mut inner = pair.into_inner();
    inner.next(); // Skip PRINT_KW
    let expr = parse_expression(inner.next().unwrap())?;

    Ok(Statement::Print { expr })
}

fn parse_func_def(pair: Pair<Rule>) -> Result<Statement, String> {
    let mut inner = pair.into_inner();
    inner.next(); // Skip FUNC_KW

    let name = inner.next().unwrap().as_str().to_string();
    inner.next(); // Skip BLOCK_START

    let mut params = Vec::new();
    let mut body = Vec::new();

    // Parse parameters until we hit BLOCK_END
    while let Some(next) = inner.next() {
        match next.as_rule() {
            Rule::BLOCK_END => break,
            Rule::VAR_NAME => params.push(next.as_str().to_string()),
            _ => continue,
        }
    }

    inner.next(); // Skip ARROW_OP
    inner.next(); // Skip BLOCK_START

    // Parse body
    for stmt_pair in inner {
        if stmt_pair.as_rule() == Rule::STATEMENT {
            body.push(parse_statement(stmt_pair)?);
        }
    }

    Ok(Statement::FuncDef { name, params, body })
}

fn parse_return_stmt(pair: Pair<Rule>) -> Result<Statement, String> {
    let mut inner = pair.into_inner();
    inner.next(); // Skip RETURN_KW

    let expr = inner.next().map(|p| parse_expression(p)).transpose()?;
    Ok(Statement::Return { expr })
}

fn parse_if_stmt(pair: Pair<Rule>) -> Result<Statement, String> {
    let mut inner = pair.into_inner();
    inner.next(); // Skip IF_KW

    let condition = parse_expression(inner.next().unwrap())?;

    // Skip THEN_KW or find BLOCK_START
    while let Some(next) = inner.next() {
        if next.as_rule() == Rule::BLOCK_START {
            break;
        }
    }

    let mut then_block = Vec::new();
    let mut else_block = None;

    // Parse then block
    for stmt_pair in inner.by_ref() {
        if stmt_pair.as_rule() == Rule::BLOCK_END {
            break;
        }
        if stmt_pair.as_rule() == Rule::STATEMENT {
            then_block.push(parse_statement(stmt_pair)?);
        }
    }

    // Check for else block
    if let Some(else_kw) = inner.next() {
        if matches!(else_kw.as_rule(), Rule::ELSE_KW) {
            inner.next(); // Skip BLOCK_START
            let mut else_stmts = Vec::new();

            for stmt_pair in inner {
                if stmt_pair.as_rule() == Rule::STATEMENT {
                    else_stmts.push(parse_statement(stmt_pair)?);
                }
            }

            else_block = Some(else_stmts);
        }
    }

    Ok(Statement::If {
        condition,
        then_block,
        else_block,
    })
}

fn parse_expression(pair: Pair<Rule>) -> Result<Expression, String> {
    match pair.as_rule() {
        Rule::NUMBER => {
            let num_str = pair.as_str();
            let value = parse_number(num_str);
            Ok(Expression::Number(value))
        }
        Rule::STRING => {
            let s = pair.as_str();
            let content = extract_string_content(s);
            Ok(Expression::String(content))
        }
        Rule::VAR_NAME => Ok(Expression::Variable(pair.as_str().to_string())),
        Rule::OPERATOR_SYMBOL => {
            let value = parse_operator_literal(pair.as_str());
            Ok(Expression::Number(value))
        }
        Rule::COMPARISON | Rule::ADD_EXPR | Rule::MULT_EXPR => parse_binary_expr(pair),
        _ => {
            // Try to parse as primary or nested expression
            let rule = pair.as_rule();
            if let Some(inner) = pair.into_inner().next() {
                parse_expression(inner)
            } else {
                Err(format!("Unknown expression type: {:?}", rule))
            }
        }
    }
}

fn parse_binary_expr(pair: Pair<Rule>) -> Result<Expression, String> {
    let mut inner = pair.into_inner();
    let mut left = parse_expression(inner.next().unwrap())?;

    while let Some(op_pair) = inner.next() {
        let op = op_pair.as_str().to_string();
        let right = parse_expression(inner.next().unwrap())?;
        left = Expression::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        };
    }

    Ok(left)
}

fn parse_number(s: &str) -> i64 {
    // Try ASCII
    if let Ok(n) = s.parse::<i64>() {
        return n;
    }

    // Repeated operators (⊕⊕⊕⊕⊕ = 5)
    if let Some(first_char) = s.chars().next() {
        if "⊕⊗⊘⊚⊙⊞⊟⊠⨁⨂⨸∀∃∄∅".contains(first_char) {
            let count = s.chars().filter(|&c| c == first_char).count();
            if count == s.chars().count() {
                return count as i64;
            }
        }
    }

    // Chinese numbers
    parse_chinese_number(s)
}

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

fn extract_string_content(s: &str) -> String {
    if s.starts_with('"') && s.ends_with('"') {
        s[1..s.len() - 1].to_string()
    } else if s.starts_with('⟦') && s.ends_with('⟧') {
        s[3..s.len() - 3].to_string()
    } else if s.starts_with('⟨') && s.ends_with('⟩') {
        s[3..s.len() - 3].to_string()
    } else {
        s.to_string()
    }
}

pub fn parse_value(pair: Pair<Rule>, env: &Environment) -> Value {
    match pair.as_rule() {
        Rule::NUMBER => Value::Number(parse_number(pair.as_str())),
        Rule::STRING => Value::String(extract_string_content(pair.as_str())),
        Rule::VAR_NAME => env.get(pair.as_str()).unwrap_or(Value::Number(0)),
        _ => Value::Number(0),
    }
}
