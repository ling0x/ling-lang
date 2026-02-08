#[cfg(test)]
mod tests {
    use ling_lang::*;
    use pest::Parser;

    // ─── Helper: parse source and return the first PROGRAM pair ───
    fn parse_program(source: &str) -> pest::iterators::Pairs<'_, Rule> {
        LingParser::parse(Rule::PROGRAM, source).expect("Failed to parse")
    }

    // ─── Helper: parse and interpret, returning the environment ───
    fn run_program(source: &str) -> Environment {
        let pairs = parse_program(source);
        let mut env = Environment::new();
        for pair in pairs {
            executor::execute_program(pair, &mut env);
        }
        env
    }

    // ═══════════════════════════════════════════════════════════════
    //  Parsing tests – verify the grammar accepts valid programs
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_parse_alien_numbers() {
        let source = "◈ x ⇐ ⊕⊕⊕⊕⊕ ⋄";
        let pairs = parse_program(source);
        assert!(pairs.into_iter().next().is_some());
    }

    #[test]
    fn test_parse_chinese_numbers() {
        let source = "◈ x ⇐ 五 ⋄";
        let pairs = parse_program(source);
        assert!(pairs.into_iter().next().is_some());
    }

    #[test]
    fn test_parse_function_definition() {
        let source = "⟡ test ⦃ n ⦄ ⇒ ⦃ ⟴ n ⋄ ⦄";
        let pairs = parse_program(source);
        assert!(pairs.into_iter().next().is_some());
    }

    #[test]
    fn test_parse_print_statement() {
        let source = "◈ x ⇐ ⊕⊕⊕ ⋄ ⟲ x ⋄";
        let pairs = parse_program(source);
        assert!(pairs.into_iter().next().is_some());
    }

    #[test]
    fn test_parse_alien_if_else() {
        let source = "◈ x ⇐ ⊕⊕⊕ ⋄ ◬ x ▷ ⊕⊕ ◭ ⦃ ⟲ x ⋄ ⦄ ◮ ⦃ ⟲ ⊕ ⋄ ⦄";
        let pairs = parse_program(source);
        assert!(pairs.into_iter().next().is_some());
    }

    #[test]
    fn test_parse_traditional_syntax() {
        let source = r#"变量 x = "你好" ; 输出 x ;"#;
        let pairs = parse_program(source);
        assert!(pairs.into_iter().next().is_some());
    }

    #[test]
    fn test_parse_while_loop() {
        let source = "◈ n ⇐ ⊕⊕⊕ ⋄ ⟳ n ▷ 〇 ⦃ ◈ n ⇐ n ⊟ ⊕ ⋄ ⦄";
        let pairs = parse_program(source);
        assert!(pairs.into_iter().next().is_some());
    }

    #[test]
    fn test_parse_string_concat() {
        let source = r#"◈ a ⇐ ⟦你好⟧ ⋄ ◈ b ⇐ a ⧺ ⟦世界⟧ ⋄"#;
        let pairs = parse_program(source);
        assert!(pairs.into_iter().next().is_some());
    }

    #[test]
    fn test_parse_arithmetic_operators() {
        let source = "◈ a ⇐ ⊕⊕⊕ ⊞ ⊗⊗ ⋄ ◈ b ⇐ ⊕⊕⊕⊕⊕ ⊟ ⊗⊗ ⋄ ◈ c ⇐ ⊕⊕⊕ ⊠ ⊗⊗ ⋄";
        let pairs = parse_program(source);
        assert!(pairs.into_iter().next().is_some());
    }

    #[test]
    fn test_parse_comments() {
        let source = "// 这是注释\n◈ x ⇐ ⊕⊕⊕ ⋄ /* 块注释 */ ⟲ x ⋄";
        let pairs = parse_program(source);
        assert!(pairs.into_iter().next().is_some());
    }

    #[test]
    fn test_parse_multiple_functions() {
        let source = "⟡ 加 ⦃ a, b ⦄ ⇒ ⦃ ⟴ a ⊞ b ⋄ ⦄ ⟡ 减 ⦃ a, b ⦄ ⇒ ⦃ ⟴ a ⊟ b ⋄ ⦄";
        let pairs = parse_program(source);
        assert!(pairs.into_iter().next().is_some());
    }

    #[test]
    fn test_parse_nested_if() {
        let source = "\
            ◈ x ⇐ ⊕⊕⊕⊕⊕ ⋄ \
            ◬ x ▷ ⊕⊕⊕ ◭ ⦃ \
                ◬ x ▷ ⊕⊕⊕⊕ ◭ ⦃ ⟲ ⟦大⟧ ⋄ ⦄ ◮ ⦃ ⟲ ⟦中⟧ ⋄ ⦄ \
            ⦄ ◮ ⦃ ⟲ ⟦小⟧ ⋄ ⦄";
        let pairs = parse_program(source);
        assert!(pairs.into_iter().next().is_some());
    }

    // ═══════════════════════════════════════════════════════════════
    //  Parse-failure tests – verify the grammar rejects bad input
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_parse_fail_missing_var_name() {
        let result = LingParser::parse(Rule::PROGRAM, "◈ ⇐ ⊕⊕ ⋄");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_fail_empty_function_body_missing_braces() {
        let result = LingParser::parse(Rule::PROGRAM, "⟡ f ⦃ ⦄ ⇒");
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════
    //  Chinese number conversion tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_chinese_number_simple_digits() {
        assert_eq!(ling_number::chinese_to_number("零"), Some(0));
        assert_eq!(ling_number::chinese_to_number("一"), Some(1));
        assert_eq!(ling_number::chinese_to_number("五"), Some(5));
        assert_eq!(ling_number::chinese_to_number("九"), Some(9));
    }

    #[test]
    fn test_chinese_number_with_units() {
        assert_eq!(ling_number::chinese_to_number("十"), Some(10));
        assert_eq!(ling_number::chinese_to_number("二十"), Some(20));
        assert_eq!(ling_number::chinese_to_number("二十三"), Some(23));
        assert_eq!(ling_number::chinese_to_number("四十二"), Some(42));
    }

    #[test]
    fn test_chinese_number_hundreds() {
        assert_eq!(ling_number::chinese_to_number("百"), Some(100));
        assert_eq!(ling_number::chinese_to_number("一百"), Some(100));
        assert_eq!(ling_number::chinese_to_number("一百二十三"), Some(123));
        assert_eq!(ling_number::chinese_to_number("三百"), Some(300));
    }

    // ═══════════════════════════════════════════════════════════════
    //  Environment tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_env_set_and_get() {
        let mut env = Environment::new();
        env.set("x".to_string(), Value::Number(42));
        assert_eq!(env.get("x"), Some(Value::Number(42)));
    }

    #[test]
    fn test_env_undefined_variable() {
        let env = Environment::new();
        assert_eq!(env.get("nonexistent"), None);
    }

    #[test]
    fn test_env_scope_push_pop() {
        let mut env = Environment::new();
        env.set("x".to_string(), Value::Number(1));

        env.push_scope();
        env.set("x".to_string(), Value::Number(2));
        assert_eq!(env.get("x"), Some(Value::Number(2)));

        env.pop_scope();
        assert_eq!(env.get("x"), Some(Value::Number(1)));
    }

    #[test]
    fn test_env_immutable_variable() {
        let mut env = Environment::new();
        env.set_const("pi".to_string(), Value::Number(314));

        let result = env.update("pi", Value::Number(0));
        assert!(result.is_err());
        assert_eq!(env.get("pi"), Some(Value::Number(314)));
    }

    #[test]
    fn test_env_mutable_update() {
        let mut env = Environment::new();
        env.set("x".to_string(), Value::Number(1));

        let result = env.update("x", Value::Number(99));
        assert!(result.is_ok());
        assert_eq!(env.get("x"), Some(Value::Number(99)));
    }

    #[test]
    fn test_env_exists() {
        let mut env = Environment::new();
        assert!(!env.exists("x"));
        env.set("x".to_string(), Value::Number(0));
        assert!(env.exists("x"));
    }

    #[test]
    fn test_env_unicode_identifiers() {
        let mut env = Environment::new();
        env.set("数值".to_string(), Value::Number(100));
        assert_eq!(env.get("数值"), Some(Value::Number(100)));

        let normalized = env.get_normalized_name("数值").unwrap();
        assert!(normalized.contains("U"));
    }

    // ═══════════════════════════════════════════════════════════════
    //  Value type tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_value_truthiness() {
        assert!(Value::Boolean(true).is_truthy());
        assert!(!Value::Boolean(false).is_truthy());
        assert!(Value::Number(1).is_truthy());
        assert!(!Value::Number(0).is_truthy());
        assert!(Value::String("hi".to_string()).is_truthy());
        assert!(!Value::String(String::new()).is_truthy());
        assert!(!Value::Void.is_truthy());
    }

    #[test]
    fn test_value_to_number() {
        assert_eq!(Value::Number(42).to_number(), Some(42));
        assert_eq!(Value::String("10".to_string()).to_number(), Some(10));
        assert_eq!(Value::Boolean(true).to_number(), Some(1));
        assert_eq!(Value::Boolean(false).to_number(), Some(0));
        assert_eq!(Value::Void.to_number(), None);
    }

    #[test]
    fn test_value_display() {
        assert_eq!(format!("{}", Value::Number(42)), "42");
        assert_eq!(format!("{}", Value::String("你好".to_string())), "你好");
        assert_eq!(format!("{}", Value::Boolean(true)), "true");
        assert_eq!(format!("{}", Value::Void), "");
    }

    #[test]
    fn test_value_from_conversions() {
        let n: Value = 42.into();
        assert_eq!(n, Value::Number(42));

        let s: Value = "你好".into();
        assert_eq!(s, Value::String("你好".to_string()));

        let b: Value = true.into();
        assert_eq!(b, Value::Boolean(true));
    }

    // ═══════════════════════════════════════════════════════════════
    //  Utility function tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_values_equal() {
        assert!(utils::values_equal(
            &Value::Number(1),
            &Value::Number(1)
        ));
        assert!(!utils::values_equal(
            &Value::Number(1),
            &Value::Number(2)
        ));
        assert!(utils::values_equal(
            &Value::String("a".into()),
            &Value::String("a".into())
        ));
        assert!(!utils::values_equal(
            &Value::Number(1),
            &Value::String("1".into())
        ));
    }

    #[test]
    fn test_normalize_operator() {
        assert_eq!(utils::normalize_operator("⊕"), "+");
        assert_eq!(utils::normalize_operator("⊟"), "-");
        assert_eq!(utils::normalize_operator("⊠"), "*");
        assert_eq!(utils::normalize_operator("⊘"), "/");
        assert_eq!(utils::normalize_operator("⊙"), "==");
        assert_eq!(utils::normalize_operator("≢"), "!=");
        assert_eq!(utils::normalize_operator("+"), "+"); // pass-through
    }

    #[test]
    fn test_is_alien_operator() {
        assert!(utils::is_alien_operator("⊕"));
        assert!(utils::is_alien_operator("⊙"));
        assert!(utils::is_alien_operator("≢"));
        assert!(!utils::is_alien_operator("+"));
        assert!(!utils::is_alien_operator("abc"));
    }

    // ═══════════════════════════════════════════════════════════════
    //  Error type tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_error_display() {
        let err = LingError::UndefinedVariable("x".to_string());
        assert!(format!("{}", err).contains("x"));

        let err = LingError::DivisionByZero;
        assert!(format!("{}", err).contains("zero"));

        let err = LingError::TypeError {
            expected: "number".to_string(),
            found: "string".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("number"));
        assert!(msg.contains("string"));
    }
}
