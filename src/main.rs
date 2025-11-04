use chinese_compiler::{ChineseLangParser, Rule, chinese_number::chinese_to_number};
use inkwell::context::Context;
use pest::Parser;
use std::fs;
use std::process::Command;

mod codegen;
use codegen::{Compiler, StringPart}; // Import StringPart

fn main() {
    let source = fs::read_to_string("programs/poem.zh").expect("无法读取文件");
    println!("Source file content:\n{}", source);

    let pairs = ChineseLangParser::parse(Rule::PROGRAM, &source).expect("解析错误");
    println!("Parsed successfully!");

    let context = Context::create();
    let mut compiler = Compiler::new(&context, "chinese_program");

    compiler.declare_printf();
    compiler.declare_sprintf(); // Add this for concatenation
    compiler.create_main_function();

    for pair in pairs {
        if pair.as_rule() == Rule::PROGRAM {
            for statement_pair in pair.into_inner() {
                compile_statement(statement_pair, &mut compiler);
            }
        }
    }

    compiler.finish_main();

    compiler.write_llvm_ir("output.ll");
    println!("✓ Generated LLVM IR: output.ll");

    compiler.write_object_file("output.o");
    println!("✓ Generated object file: output.o");

    let status = Command::new("clang")
        .args(["output.o", "-o", "program"])
        .status()
        .expect("Failed to link");

    if status.success() {
        println!("✓ Generated executable: ./program");
        println!("\nRunning compiled program:");
        Command::new("./program").status().unwrap();
    }
}

fn compile_statement(pair: pest::iterators::Pair<Rule>, compiler: &mut Compiler) {
    println!("Compiling statement: {:?}", pair.as_rule());
    match pair.as_rule() {
        Rule::STATEMENT => {
            for inner in pair.into_inner() {
                compile_statement(inner, compiler);
            }
        }
        Rule::VAR_DECL => {
            let mut inner = pair.into_inner();
            inner.next(); // Skip LET_KW
            let var_name = inner.next().unwrap().as_str();
            let value_pair = inner.next().unwrap();

            // Check if this is a concatenation or simple value
            let parts = extract_string_parts(value_pair);

            if parts.len() == 1 && matches!(parts[0], StringPart::Literal(_)) {
                // Simple literal assignment
                if let StringPart::Literal(s) = &parts[0] {
                    println!("Variable: {} = {} (literal)", var_name, s);
                    compiler.compile_string_var(var_name, s);
                }
            } else {
                // Concatenation needed
                println!("Variable: {} = <concatenation>", var_name);
                compiler.compile_string_concat(var_name, parts);
            }
        }
        Rule::PRINT_STMT => {
            let mut inner = pair.into_inner();
            inner.next(); // Skip PRINT_KW
            let value_pair = inner.next().unwrap();
            let var_name = extract_var_name(value_pair);
            println!("Printing variable: {}", var_name);
            compiler.compile_print_string(&var_name);
        }
        _ => {}
    }
}

// Extract StringParts from an expression
fn extract_string_parts(pair: pest::iterators::Pair<Rule>) -> Vec<StringPart> {
    let mut parts = Vec::new();

    match pair.as_rule() {
        Rule::VALUE => {
            let expr = pair.into_inner().next().unwrap();
            parts.extend(extract_string_parts(expr));
        }
        Rule::EXPRESSION => {
            // EXPRESSION contains TERMs separated by CONCAT_OP
            for term_pair in pair.into_inner() {
                if term_pair.as_rule() == Rule::TERM {
                    parts.extend(extract_term_parts(term_pair));
                }
                // Skip CONCAT_OP tokens
            }
        }
        _ => {}
    }

    parts
}

fn extract_term_parts(pair: pest::iterators::Pair<Rule>) -> Vec<StringPart> {
    match pair.as_rule() {
        Rule::TERM => {
            let inner = pair.into_inner().next().unwrap();
            extract_term_parts(inner)
        }
        Rule::STRING => {
            let s = pair.as_str();
            let content = s[1..s.len() - 1].to_string();
            vec![StringPart::Literal(content)]
        }
        Rule::NUMBER => {
            let num_str = pair.as_str();
            let num_value = if num_str.chars().next().unwrap() as u32 > 127 {
                chinese_to_number(num_str).unwrap().to_string()
            } else {
                num_str.to_string()
            };
            vec![StringPart::Literal(num_value)]
        }
        Rule::VAR_NAME => {
            vec![StringPart::Variable(pair.as_str().to_string())]
        }
        Rule::EXPRESSION => extract_string_parts(pair),
        _ => Vec::new(),
    }
}

fn extract_var_name(pair: pest::iterators::Pair<Rule>) -> String {
    match pair.as_rule() {
        Rule::VALUE | Rule::EXPRESSION | Rule::TERM => {
            let inner = pair.into_inner().next().unwrap();
            extract_var_name(inner)
        }
        Rule::VAR_NAME => pair.as_str().to_string(),
        _ => panic!("Expected variable name"),
    }
}
