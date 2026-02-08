use crate::{Environment, Rule, Value, evaluator::evaluate_expression};

pub fn execute_program(pair: pest::iterators::Pair<Rule>, env: &mut Environment) {
    match pair.as_rule() {
        Rule::PROGRAM => {
            for inner_pair in pair.into_inner() {
                execute_program(inner_pair, env);
            }
        }
        Rule::STATEMENT => {
            for inner_pair in pair.into_inner() {
                execute_program(inner_pair, env);
            }
        }
        Rule::VAR_DECL => {
            let mut inner = pair.into_inner();
            inner.next(); // Skip LET_KW
            let var_name = inner.next().unwrap().as_str().to_string();
            let value_pair = inner.next().unwrap();

            // Extract EXPRESSION from VALUE
            let expr_pair = value_pair.into_inner().next().unwrap();
            let value = evaluate_expression(expr_pair, env);
            env.set(var_name, value);
        }
        Rule::PRINT_STMT => {
            let mut inner = pair.into_inner();
            inner.next(); // Skip PRINT_KW
            let value_pair = inner.next().unwrap();

            // Extract EXPRESSION from VALUE
            let expr_pair = value_pair.into_inner().next().unwrap();
            let value = evaluate_expression(expr_pair, env);
            match value {
                Value::Number(n) => println!("{}", n),
                Value::String(s) => println!("{}", s),
                Value::Boolean(b) => println!("{}", b),
                Value::Function(f) => println!("<function {}>", f.name),
                Value::Void => {}
            }
        }
        _ => {}
    }
}
