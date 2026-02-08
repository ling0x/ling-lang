use inkwell::AddressSpace;
use inkwell::IntPredicate;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::BasicType;
use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum};
use inkwell::values::{BasicValueEnum, FunctionValue, IntValue, PointerValue};
use std::collections::HashMap;

/// Represents parts of a string expression (for concatenation)
#[derive(Debug, Clone)]
pub enum StringPart {
    Literal(String),
    Variable(String),
}

/// Runtime value wrapper for LLVM values
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum RuntimeValue<'ctx> {
    String(PointerValue<'ctx>),
    Integer(IntValue<'ctx>),
}

/// Symbol table entry for variables
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Symbol<'ctx> {
    value: RuntimeValue<'ctx>,
    is_mutable: bool,
}

pub struct Compiler<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,

    // Symbol table for variables
    symbols: HashMap<String, Symbol<'ctx>>,

    // Symbol mapping for alien characters
    alien_symbol_map: HashMap<String, String>,

    // Counter for generating unique names
    temp_counter: usize,

    // Current function being compiled
    current_function: Option<FunctionValue<'ctx>>,
}

#[allow(dead_code)]
impl<'ctx> Compiler<'ctx> {
    /// Create a new compiler instance
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        let mut compiler = Compiler {
            context,
            module,
            builder,
            symbols: HashMap::new(),
            alien_symbol_map: HashMap::new(),
            temp_counter: 0,
            current_function: None,
        };

        compiler.init_alien_symbols();
        compiler.declare_runtime_functions();
        compiler
    }

    /// Declare all standard library functions at once
    pub fn declare_stdlib(&self) {
        self.declare_printf();
        self.declare_sprintf();
    }

    /// Initialize alien symbol mappings
    fn init_alien_symbols(&mut self) {
        // Alien digits (mathematical symbols)
        let alien_digits = [
            ("∅", "0"),
            ("∄", "1"),
            ("∃", "2"),
            ("∀", "3"),
            ("℧", "4"),
            ("℥", "5"),
            ("℞", "6"),
            ("℟", "7"),
            ("℣", "8"),
            ("℈", "9"),
        ];

        for (alien, ascii) in alien_digits {
            self.alien_symbol_map
                .insert(alien.to_string(), ascii.to_string());
        }

        // Chinese digits
        let chinese_digits = [
            ("零", "0"),
            ("〇", "0"),
            ("一", "1"),
            ("二", "2"),
            ("三", "3"),
            ("四", "4"),
            ("五", "5"),
            ("六", "6"),
            ("七", "7"),
            ("八", "8"),
            ("九", "9"),
        ];

        for (chinese, ascii) in chinese_digits {
            self.alien_symbol_map
                .insert(chinese.to_string(), ascii.to_string());
        }
    }

    /// Declare all runtime functions (printf, sprintf, etc.)
    fn declare_runtime_functions(&self) {
        self.declare_printf();
        self.declare_sprintf();
        self.declare_strlen();
        self.declare_strcat();
        self.declare_malloc();
    }

    /// Declare printf for output
    pub fn declare_printf(&self) {
        let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
        let printf_type = self.context.i32_type().fn_type(
            &[i8_ptr_type.into()],
            true, // variadic
        );
        self.module.add_function("printf", printf_type, None);
    }

    /// Declare sprintf for string building
    pub fn declare_sprintf(&self) {
        let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
        let sprintf_type = self.context.i32_type().fn_type(
            &[i8_ptr_type.into(), i8_ptr_type.into()],
            true, // variadic
        );
        self.module.add_function("sprintf", sprintf_type, None);
    }

    /// Declare strlen for string length
    pub fn declare_strlen(&self) {
        let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
        let strlen_type = self
            .context
            .i64_type()
            .fn_type(&[i8_ptr_type.into()], false);
        self.module.add_function("strlen", strlen_type, None);
    }

    /// Declare strcat for string concatenation
    pub fn declare_strcat(&self) {
        let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
        let strcat_type = i8_ptr_type.fn_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false);
        self.module.add_function("strcat", strcat_type, None);
    }

    /// Declare malloc for dynamic memory allocation
    pub fn declare_malloc(&self) {
        let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
        let malloc_type = i8_ptr_type.fn_type(&[self.context.i64_type().into()], false);
        self.module.add_function("malloc", malloc_type, None);
    }

    /// Generate a unique temporary name
    fn gen_temp_name(&mut self, prefix: &str) -> String {
        let name = format!("{}{}", prefix, self.temp_counter);
        self.temp_counter += 1;
        name
    }

    /// Create main function wrapper
    pub fn create_main_function(&mut self) -> FunctionValue<'ctx> {
        let i32_type = self.context.i32_type();
        let main_type = i32_type.fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_type, None);
        let entry = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(entry);
        self.current_function = Some(main_fn);
        main_fn
    }

    /// Create a custom function
    pub fn create_function(
        &mut self,
        name: &str,
        param_types: &[BasicMetadataTypeEnum<'ctx>],
        return_type: Option<BasicTypeEnum<'ctx>>,
    ) -> FunctionValue<'ctx> {
        let fn_type = if let Some(ret_type) = return_type {
            ret_type.fn_type(param_types, false)
        } else {
            self.context.void_type().fn_type(param_types, false)
        };

        let function = self.module.add_function(name, fn_type, None);
        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);
        self.current_function = Some(function);
        function
    }

    /// Finish the current function with return
    pub fn finish_function(&self, return_value: Option<BasicValueEnum<'ctx>>) {
        if let Some(val) = return_value {
            self.builder.build_return(Some(&val)).unwrap();
        } else {
            self.builder.build_return(None).unwrap();
        }
    }

    /// Finish main function with return 0
    pub fn finish_main(&self) {
        let i32_type = self.context.i32_type();
        let zero = i32_type.const_int(0, false);
        self.builder.build_return(Some(&zero)).unwrap();
    }

    /// Parse alien/Chinese numbers to i64
    pub fn parse_number(&self, num_str: &str) -> i64 {
        // Try ASCII number first
        if let Ok(n) = num_str.parse::<i64>() {
            return n;
        }

        // Check if it's a repeated operator number (⊕⊕⊕⊕⊕ = 5)
        if let Some(first_char) = num_str.chars().next() {
            if "⊕⊗⊘⊚⊙⊞⊟⊠⨁⨂⨸∀∃∄∅".contains(first_char) {
                let count = num_str.chars().take_while(|&c| c == first_char).count();

                if count == num_str.chars().count() {
                    return count as i64;
                }
            }
        }

        // Handle alien digit strings
        if num_str.chars().all(|c| "∅∄∃∀℧℥℞℟℣℈".contains(c)) {
            let mut result = String::new();
            for ch in num_str.chars() {
                if let Some(digit) = self.alien_symbol_map.get(&ch.to_string()) {
                    result.push_str(digit);
                }
            }
            return result.parse().unwrap_or(0);
        }

        // Handle Chinese numbers
        self.parse_chinese_number(num_str)
    }

    /// Parse Chinese numerals
    fn parse_chinese_number(&self, s: &str) -> i64 {
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
    pub fn parse_operator_literal(&self, op: &str) -> i64 {
        match op {
            "⊕" => 1,
            "⊗" => 2,
            "⊘" => 0,
            "⊚" => 10,
            "⊙" => 5,
            "⊞" => 1,
            "⊟" => 0,
            "⊠" => 2,
            _ => 0,
        }
    }

    /// Normalize alien identifiers to ASCII-safe names
    pub fn normalize_identifier(&self, name: &str) -> String {
        let mut result = String::new();

        for ch in name.chars() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                result.push(ch);
            } else {
                // Convert Unicode to hex representation
                result.push_str(&format!("_U{:04X}_", ch as u32));
            }
        }

        if result.is_empty() || result.chars().next().unwrap().is_ascii_digit() {
            result.insert(0, '_');
        }

        result
    }

    /// Store an integer variable
    pub fn store_integer(&mut self, var_name: &str, value: i64) {
        let i64_type = self.context.i64_type();
        let alloca = self.builder.build_alloca(i64_type, var_name).unwrap();
        let int_val = i64_type.const_int(value as u64, false);
        self.builder.build_store(alloca, int_val).unwrap();

        let loaded = self.builder.build_load(i64_type, alloca, "load").unwrap();
        self.symbols.insert(
            var_name.to_string(),
            Symbol {
                value: RuntimeValue::Integer(loaded.into_int_value()),
                is_mutable: true,
            },
        );
    }

    /// Store a string variable
    pub fn store_string(&mut self, var_name: &str, value: &str) {
        let name = self.gen_temp_name("str");
        let global = self.builder.build_global_string_ptr(value, &name).unwrap();

        self.symbols.insert(
            var_name.to_string(),
            Symbol {
                value: RuntimeValue::String(global.as_pointer_value()),
                is_mutable: true,
            },
        );
    }

    /// Get a variable's runtime value
    pub fn get_variable(&self, var_name: &str) -> Option<RuntimeValue<'ctx>> {
        self.symbols.get(var_name).map(|sym| sym.value)
    }

    /// Print a value (string or integer)
    pub fn print_value(&self, value: RuntimeValue<'ctx>) {
        let printf = self.module.get_function("printf").unwrap();

        match value {
            RuntimeValue::String(ptr) => {
                let format_str = self
                    .builder
                    .build_global_string_ptr("%s\n", "str_fmt")
                    .unwrap();
                self.builder
                    .build_call(
                        printf,
                        &[format_str.as_pointer_value().into(), ptr.into()],
                        "printf_call",
                    )
                    .unwrap();
            }
            RuntimeValue::Integer(int_val) => {
                let format_str = self
                    .builder
                    .build_global_string_ptr("%lld\n", "int_fmt")
                    .unwrap();
                self.builder
                    .build_call(
                        printf,
                        &[format_str.as_pointer_value().into(), int_val.into()],
                        "printf_call",
                    )
                    .unwrap();
            }
        }
    }

    /// Print a variable by name
    pub fn print_variable(&self, var_name: &str) {
        if let Some(value) = self.get_variable(var_name) {
            self.print_value(value);
        } else {
            panic!("Variable '{}' not found", var_name);
        }
    }

    /// Concatenate strings at runtime
    pub fn concat_strings(&mut self, var_name: &str, parts: Vec<StringPart>) -> PointerValue<'ctx> {
        let sprintf = self.module.get_function("sprintf").unwrap();
        let i8_type = self.context.i8_type();

        // Allocate buffer for result
        let buffer = self
            .builder
            .build_array_alloca(i8_type, i8_type.const_int(1024, false), "concat_buffer")
            .unwrap();

        // Build format string and collect arguments
        let mut format = String::new();
        let mut args = vec![buffer.into()];

        for part in &parts {
            match part {
                StringPart::Literal(s) => {
                    format.push_str(s);
                }
                StringPart::Variable(v) => {
                    format.push_str("%s");
                    if let Some(RuntimeValue::String(ptr)) = self.get_variable(v) {
                        args.push(ptr.into());
                    }
                }
            }
        }

        let fmt_name = self.gen_temp_name("fmt");
        let format_str = self
            .builder
            .build_global_string_ptr(&format, &fmt_name)
            .unwrap();
        args.insert(1, format_str.as_pointer_value().into());

        // Call sprintf
        self.builder
            .build_call(sprintf, &args, "sprintf_call")
            .unwrap();

        // Store result
        self.symbols.insert(
            var_name.to_string(),
            Symbol {
                value: RuntimeValue::String(buffer),
                is_mutable: true,
            },
        );

        buffer
    }

    /// Build arithmetic operations
    pub fn build_arithmetic(
        &self,
        op: &str,
        left: IntValue<'ctx>,
        right: IntValue<'ctx>,
    ) -> IntValue<'ctx> {
        match op {
            "+" | "⊕" | "⊞" | "⨁" => self.builder.build_int_add(left, right, "add").unwrap(),
            "-" | "⊟" | "⨂" => self.builder.build_int_sub(left, right, "sub").unwrap(),
            "*" | "⊗" | "⊠" => self.builder.build_int_mul(left, right, "mul").unwrap(),
            "/" | "⊘" | "⨸" => self
                .builder
                .build_int_signed_div(left, right, "div")
                .unwrap(),
            _ => panic!("Unknown arithmetic operator: {}", op),
        }
    }

    /// Build comparison operations
    pub fn build_comparison(
        &self,
        op: &str,
        left: IntValue<'ctx>,
        right: IntValue<'ctx>,
    ) -> IntValue<'ctx> {
        let predicate = match op {
            "==" | "⊙" | "≡" => IntPredicate::EQ,
            "!=" | "⊗" | "≢" => IntPredicate::NE,
            "<" | "◁" | "⊲" => IntPredicate::SLT,
            ">" | "▷" | "⊳" => IntPredicate::SGT,
            "<=" => IntPredicate::SLE,
            ">=" => IntPredicate::SGE,
            _ => panic!("Unknown comparison operator: {}", op),
        };

        self.builder
            .build_int_compare(predicate, left, right, "cmp")
            .unwrap()
    }

    /// Output LLVM IR to file
    pub fn write_llvm_ir(&self, path: &str) {
        self.module.print_to_file(path).unwrap();
    }

    /// Output object file
    pub fn write_object_file(&self, path: &str) {
        use inkwell::targets::{
            CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
        };

        Target::initialize_native(&InitializationConfig::default()).unwrap();
        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple).unwrap();
        let target_machine = target
            .create_target_machine(
                &target_triple,
                "generic",
                "",
                inkwell::OptimizationLevel::Default,
                RelocMode::PIC,
                CodeModel::Default,
            )
            .unwrap();

        target_machine
            .write_to_file(&self.module, FileType::Object, path.as_ref())
            .unwrap();
    }
}
