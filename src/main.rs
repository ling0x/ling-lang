use inkwell::context::Context;
use ling_lang::{Environment, LingParser, Rule, Value};
use pest::Parser;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

mod codegen;
use codegen::{Compiler, StringPart};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <source-file.ling>", args[0]);
        eprintln!("\nExamples:");
        eprintln!("  {} programs/poem.ling", args[0]);
        eprintln!("  {} examples/example1.alien", args[0]);
        std::process::exit(1);
    }

    let source_file = &args[1];

    match compile_and_run(source_file) {
        Ok(()) => println!("\n✓ Success!"),
        Err(e) => {
            eprintln!("\n✗ Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn compile_and_run(source_file: &str) -> Result<(), String> {
    println!("════════════════════════════════════════");
    println!("  Ling's Alien Language Compiler");
    println!("════════════════════════════════════════\n");

    // Read source file
    let source = fs::read_to_string(source_file)
        .map_err(|e| format!("Failed to read file '{}': {}", source_file, e))?;

    println!("📄 Source file: {}", source_file);
    println!("─────────────────────────────────────────");
    println!("{}", source);
    println!("─────────────────────────────────────────\n");

    // Parse
    println!("🔍 Parsing...");
    let _pairs =
        LingParser::parse(Rule::PROGRAM, &source).map_err(|e| format!("Parse error: {}", e))?;
    println!("✓ Parsed successfully!\n");

    // Interpret for immediate feedback
    println!("🎭 Interpreting...");
    interpret_program(&source)?;
    println!();

    // Compile to LLVM
    println!("⚙️  Compiling to LLVM IR...");
    let context = Context::create();
    let mut compiler = Compiler::new(&context, "alien_module");

    compiler.declare_stdlib();
    compiler.create_main_function();

    let pairs =
        LingParser::parse(Rule::PROGRAM, &source).map_err(|e| format!("Parse error: {}", e))?;

    for pair in pairs {
        if pair.as_rule() == Rule::PROGRAM {
            for statement_pair in pair.into_inner() {
                if statement_pair.as_rule() != Rule::EOI {
                    compile_statement(statement_pair, &mut compiler)?;
                }
            }
        }
    }

    compiler.finish_main();

    // Output files - place compiled artifacts in an output directory
    let output_dir = std::env::var("LING_OUTPUT_DIR")
        .unwrap_or_else(|_| "tests/test_compiled".to_string());

    fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Failed to create output directory '{}': {}", output_dir, e))?;

    let base_name = Path::new(source_file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let ir_file = format!("{}/{}.ll", output_dir, base_name);
    let obj_file = format!("{}/{}.o", output_dir, base_name);
    let exe_file = format!("{}/{}", output_dir, base_name);

    compiler.write_llvm_ir(&ir_file);
    println!("✓ Generated LLVM IR: {}", ir_file);

    compiler.write_object_file(&obj_file);
    println!("✓ Generated object file: {}", obj_file);

    // Link with clang
    println!("\n🔗 Linking...");
    let status = Command::new("clang")
        .args([&obj_file, "-o", &exe_file])
        .status()
        .map_err(|e| format!("Failed to run clang: {}", e))?;

    if !status.success() {
        return Err("Linking failed".to_string());
    }

    println!("✓ Generated executable: {}\n", exe_file);

    // Run the compiled program
    println!("🚀 Running compiled program:");
    println!("─────────────────────────────────────────");
    let output = Command::new(&exe_file)
        .output()
        .map_err(|e| format!("Failed to run program: {}", e))?;

    print!("{}", String::from_utf8_lossy(&output.stdout));
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }
    println!("─────────────────────────────────────────");

    Ok(())
}

// Interpreter for immediate feedback
fn interpret_program(source: &str) -> Result<(), String> {
    let pairs =
        LingParser::parse(Rule::PROGRAM, source).map_err(|e| format!("Parse error: {}", e))?;

    let mut env = Environment::new();
    let mut function_defs: HashMap<String, FunctionDef> = HashMap::new();

    for pair in pairs {
        if pair.as_rule() == Rule::PROGRAM {
            for statement_pair in pair.into_inner() {
                if statement_pair.as_rule() != Rule::EOI {
                    interpret_statement(statement_pair, &mut env, &mut function_defs)?;
                }
            }
        }
    }

    Ok(())
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct FunctionDef {
    params: Vec<String>,
    body: Vec<String>,
}

fn interpret_statement(
    pair: pest::iterators::Pair<Rule>,
    env: &mut Environment,
    functions: &mut HashMap<String, FunctionDef>,
) -> Result<(), String> {
    match pair.as_rule() {
        Rule::STATEMENT => {
            for inner in pair.into_inner() {
                interpret_statement(inner, env, functions)?;
            }
        }
        Rule::VAR_DECL => {
            let mut inner = pair.into_inner();
            inner.next(); // Skip LET_KW (变量, 变, ⟡, ◈, etc.)

            let var_name = inner
                .next()
                .ok_or("Missing variable name")?
                .as_str()
                .to_string();

            inner.next(); // Skip ASSIGN_OP (=, ⇐, ⟸)

            let value_pair = inner.next().ok_or("Missing value")?;

            let value = evaluate_expression(value_pair, env)?;
            env.set(var_name.clone(), value.clone());

            println!("  {} = {}", var_name, value);
        }
        Rule::PRINT_STMT => {
            let mut inner = pair.into_inner();
            inner.next(); // Skip PRINT_KW (输出, ⟲, ◉)

            let value_pair = inner.next().ok_or("Missing print value")?;

            let value = evaluate_expression(value_pair, env)?;
            println!("  Output: {}", value);
        }
        Rule::FUNC_DEF => {
            let mut inner = pair.into_inner();
            inner.next(); // Skip FUNC_KW (⟡, 函数)

            let func_name = inner
                .next()
                .ok_or("Missing function name")?
                .as_str()
                .to_string();

            inner.next(); // Skip BLOCK_START (⦃)

            let mut params = Vec::new();

            // Collect parameters until BLOCK_END
            loop {
                match inner.next() {
                    Some(p) if p.as_rule() == Rule::BLOCK_END => break,
                    Some(p) if p.as_rule() == Rule::VAR_NAME => {
                        params.push(p.as_str().to_string());
                    }
                    Some(_) => continue,
                    None => break,
                }
            }

            functions.insert(
                func_name.clone(),
                FunctionDef {
                    params: params.clone(),
                    body: Vec::new(),
                },
            );

            println!("  Defined function: {}({:?})", func_name, params);
        }
        Rule::IF_STMT | Rule::ALIEN_IF_STMT | Rule::TRAD_IF_STMT => {
            interpret_if_statement(pair, env, functions)?;
        }
        Rule::RETURN_STMT => {
            let mut inner = pair.into_inner();
            inner.next(); // Skip RETURN_KW (⟴, 返回)

            if let Some(expr) = inner.next() {
                let value = evaluate_expression(expr, env)?;
                println!("  Return: {}", value);
            }
        }
        _ => {}
    }

    Ok(())
}

fn interpret_if_statement(
    pair: pest::iterators::Pair<Rule>,
    env: &mut Environment,
    functions: &mut HashMap<String, FunctionDef>,
) -> Result<(), String> {
    // IF_STMT wraps ALIEN_IF_STMT or TRAD_IF_STMT — unwrap it
    let actual_pair = if pair.as_rule() == Rule::IF_STMT {
        pair.into_inner().next().ok_or("Empty IF_STMT")?
    } else {
        pair
    };

    let mut inner = actual_pair.into_inner();

    inner.next(); // Skip IF_KW (◬, 如果)

    // For TRAD_IF_STMT, skip PAREN_OPEN before the condition
    let cond_pair = inner.next().ok_or("Missing condition")?;
    let condition = if cond_pair.as_rule() == Rule::PAREN_OPEN {
        // Traditional if: skip ( , get expression, skip )
        let expr = inner.next().ok_or("Missing condition expression")?;
        let val = evaluate_expression(expr, env)?;
        inner.next(); // Skip PAREN_CLOSE
        val
    } else {
        evaluate_expression(cond_pair, env)?
    };

    // Skip THEN_KW (◭) or find BLOCK_START
    while let Some(next) = inner.next() {
        if next.as_rule() == Rule::BLOCK_START {
            break;
        }
    }

    let is_true = condition.is_truthy();

    // Collect then block
    let mut then_stmts = Vec::new();
    for stmt in inner.by_ref() {
        if stmt.as_rule() == Rule::BLOCK_END {
            break;
        }
        if stmt.as_rule() == Rule::STATEMENT {
            then_stmts.push(stmt);
        }
    }

    // Check for else block
    let mut else_stmts = Vec::new();
    if let Some(else_kw) = inner.next() {
        if matches!(else_kw.as_rule(), Rule::ELSE_KW) {
            inner.next(); // Skip BLOCK_START

            for stmt in inner {
                if stmt.as_rule() == Rule::STATEMENT {
                    else_stmts.push(stmt);
                }
            }
        }
    }

    // Execute appropriate block
    if is_true {
        for stmt in then_stmts {
            interpret_statement(stmt, env, functions)?;
        }
    } else {
        for stmt in else_stmts {
            interpret_statement(stmt, env, functions)?;
        }
    }

    Ok(())
}

fn evaluate_expression(
    pair: pest::iterators::Pair<Rule>,
    env: &Environment,
) -> Result<Value, String> {
    match pair.as_rule() {
        Rule::VALUE
        | Rule::EXPRESSION
        | Rule::CONCAT_EXPR
        | Rule::COMPARISON
        | Rule::ADD_EXPR
        | Rule::MULT_EXPR => {
            if let Some(inner) = pair.clone().into_inner().next() {
                evaluate_expression(inner, env)
            } else {
                Err("Empty expression".to_string())
            }
        }
        Rule::PRIMARY => {
            let inner = pair.into_inner().next().ok_or("Empty primary")?;
            evaluate_expression(inner, env)
        }
        Rule::NUMBER => Ok(Value::Number(parse_number(pair.as_str()))),
        Rule::OPERATOR_NUMBER => Ok(Value::Number(parse_number(pair.as_str()))),
        Rule::OPERATOR_SYMBOL => Ok(Value::Number(parse_operator_literal(pair.as_str()))),
        Rule::STRING => {
            let s = pair.as_str();
            let content = extract_string_content(s);
            Ok(Value::String(content))
        }
        Rule::VAR_NAME => {
            let var_name = pair.as_str();
            env.get(var_name)
                .ok_or_else(|| format!("Undefined variable: {}", var_name))
        }
        Rule::TERM => {
            let inner = pair.into_inner().next().ok_or("Empty term")?;
            evaluate_expression(inner, env)
        }
        _ => Err(format!("Unknown expression type: {:?}", pair.as_rule())),
    }
}

fn compile_statement(
    pair: pest::iterators::Pair<Rule>,
    compiler: &mut Compiler,
) -> Result<(), String> {
    match pair.as_rule() {
        Rule::STATEMENT => {
            for inner in pair.into_inner() {
                compile_statement(inner, compiler)?;
            }
        }
        Rule::VAR_DECL => {
            let mut inner = pair.into_inner();
            inner.next(); // Skip LET_KW

            let var_name = inner.next().ok_or("Missing variable name")?.as_str();

            inner.next(); // Skip ASSIGN_OP

            let value_pair = inner.next().ok_or("Missing value")?;

            let parts = extract_string_parts(value_pair);

            if parts.len() == 1 {
                match &parts[0] {
                    StringPart::Literal(s) => {
                        compiler.store_string(var_name, s);
                    }
                    StringPart::Variable(v) => {
                        // Copy variable value
                        if let Some(_val) = compiler.get_variable(v) {
                            // TODO: Handle based on type
                        }
                    }
                }
            } else {
                compiler.concat_strings(var_name, parts);
            }
        }
        Rule::PRINT_STMT => {
            let mut inner = pair.into_inner();
            inner.next(); // Skip PRINT_KW

            let value_pair = inner.next().ok_or("Missing print value")?;

            let var_name = extract_var_name(value_pair)?;
            compiler.print_variable(&var_name);
        }
        Rule::FUNC_DEF => {
            println!("Note: Function compilation not yet implemented");
        }
        Rule::IF_STMT | Rule::ALIEN_IF_STMT | Rule::TRAD_IF_STMT => {
            println!("Note: If-statement compilation not yet implemented");
        }
        Rule::WHILE_STMT => {
            println!("Note: While-loop compilation not yet implemented");
        }
        _ => {}
    }

    Ok(())
}

fn extract_string_parts(pair: pest::iterators::Pair<Rule>) -> Vec<StringPart> {
    let mut parts = Vec::new();

    match pair.as_rule() {
        Rule::VALUE
        | Rule::EXPRESSION
        | Rule::ARITHMETIC_EXPR
        | Rule::COMPARISON
        | Rule::ADD_EXPR
        | Rule::MULT_EXPR
        | Rule::PRIMARY => {
            // Unwrap single-child wrapper rules
            if let Some(inner) = pair.into_inner().next() {
                parts.extend(extract_string_parts(inner));
            }
        }
        Rule::CONCAT_EXPR => {
            for child in pair.into_inner() {
                match child.as_rule() {
                    Rule::CONCAT_OP => continue,
                    _ => parts.extend(extract_string_parts(child)),
                }
            }
        }
        Rule::TERM => {
            parts.extend(extract_term_parts(pair));
        }
        Rule::STRING => {
            let content = extract_string_content(pair.as_str());
            parts.push(StringPart::Literal(content));
        }
        Rule::NUMBER | Rule::OPERATOR_NUMBER => {
            let num_value = parse_number(pair.as_str()).to_string();
            parts.push(StringPart::Literal(num_value));
        }
        Rule::OPERATOR_SYMBOL => {
            let num_value = parse_operator_literal(pair.as_str()).to_string();
            parts.push(StringPart::Literal(num_value));
        }
        Rule::VAR_NAME => {
            parts.push(StringPart::Variable(pair.as_str().to_string()));
        }
        _ => {
            if let Some(inner) = pair.into_inner().next() {
                parts.extend(extract_string_parts(inner));
            }
        }
    }

    parts
}

fn extract_term_parts(pair: pest::iterators::Pair<Rule>) -> Vec<StringPart> {
    match pair.as_rule() {
        Rule::TERM => {
            if let Some(inner) = pair.into_inner().next() {
                extract_term_parts(inner)
            } else {
                Vec::new()
            }
        }
        Rule::STRING => {
            let content = extract_string_content(pair.as_str());
            vec![StringPart::Literal(content)]
        }
        Rule::NUMBER | Rule::OPERATOR_NUMBER => {
            let num_value = parse_number(pair.as_str()).to_string();
            vec![StringPart::Literal(num_value)]
        }
        Rule::OPERATOR_SYMBOL => {
            let num_value = parse_operator_literal(pair.as_str()).to_string();
            vec![StringPart::Literal(num_value)]
        }
        Rule::VAR_NAME => {
            vec![StringPart::Variable(pair.as_str().to_string())]
        }
        _ => Vec::new(),
    }
}

fn extract_var_name(pair: pest::iterators::Pair<Rule>) -> Result<String, String> {
    match pair.as_rule() {
        Rule::VALUE
        | Rule::EXPRESSION
        | Rule::CONCAT_EXPR
        | Rule::COMPARISON
        | Rule::ADD_EXPR
        | Rule::MULT_EXPR
        | Rule::PRIMARY
        | Rule::ARITHMETIC_EXPR
        | Rule::TERM => {
            let inner = pair.into_inner().next().ok_or("Empty value")?;
            extract_var_name(inner)
        }
        Rule::VAR_NAME => Ok(pair.as_str().to_string()),
        _ => Err(format!("Expected variable name, got {:?}", pair.as_rule())),
    }
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

    // Try Chinese numbers
    if let Some(n) = ling_lang::ling_number::chinese_to_number(s) {
        return n;
    }

    0
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
        // UTF-8: ⟦ is 3 bytes, ⟧ is 3 bytes
        let bytes = s.as_bytes();
        String::from_utf8_lossy(&bytes[3..bytes.len() - 3]).to_string()
    } else if s.starts_with('⟨') && s.ends_with('⟩') {
        let bytes = s.as_bytes();
        String::from_utf8_lossy(&bytes[3..bytes.len() - 3]).to_string()
    } else {
        s.to_string()
    }
}
