use inkwell::AddressSpace;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{FunctionValue, PointerValue};
use std::collections::HashMap;

pub struct Compiler<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    variables: HashMap<String, PointerValue<'ctx>>,
}

pub enum StringPart {
    Literal(String),
    Variable(String),
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        Compiler {
            context,
            module,
            builder,
            variables: HashMap::new(),
        }
    }

    // Declare printf for output
    pub fn declare_printf(&self) {
        let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
        let printf_type = self.context.i32_type().fn_type(
            &[i8_ptr_type.into()],
            true, // variadic
        );
        self.module.add_function("printf", printf_type, None);
    }

    // Declare sprintf for string building
    pub fn declare_sprintf(&self) {
        let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
        let sprintf_type = self.context.i32_type().fn_type(
            &[i8_ptr_type.into(), i8_ptr_type.into()],
            true, // variadic
        );
        self.module.add_function("sprintf", sprintf_type, None);
    }

    // Create main function wrapper
    pub fn create_main_function(&self) -> FunctionValue<'ctx> {
        let i32_type = self.context.i32_type();
        let main_type = i32_type.fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_type, None);
        let entry = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(entry);
        main_fn
    }

    pub fn finish_main(&self) {
        let i32_type = self.context.i32_type();
        let zero = i32_type.const_int(0, false);
        self.builder.build_return(Some(&zero)).unwrap();
    }

    // Output LLVM IR to file
    pub fn write_llvm_ir(&self, path: &str) {
        self.module.print_to_file(path).unwrap();
    }

    // Output object file
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

    // Store string as global variable
    pub fn compile_string_var(&mut self, var_name: &str, value: &str) {
        let global = self
            .builder
            .build_global_string_ptr(value, var_name)
            .unwrap();
        self.variables
            .insert(var_name.to_string(), global.as_pointer_value());
    }

    // Print string variable
    pub fn compile_print_string(&self, var_name: &str) {
        let printf = self.module.get_function("printf").unwrap();

        // Get the string variable
        let string_ptr = self.variables.get(var_name).unwrap();

        // Call printf directly with the string (it already has format)
        let format_str = self.builder.build_global_string_ptr("%s\n", "fmt").unwrap();
        self.builder
            .build_call(
                printf,
                &[format_str.as_pointer_value().into(), (*string_ptr).into()],
                "printf_call",
            )
            .unwrap();
    }

    // Store a string literal and return its pointer
    pub fn create_string_literal(&mut self, value: &str, name: &str) -> PointerValue<'ctx> {
        let global = self.builder.build_global_string_ptr(value, name).unwrap();
        global.as_pointer_value()
    }

    // Concatenate strings at runtime
    pub fn compile_string_concat(&mut self, var_name: &str, parts: Vec<StringPart>) {
        let sprintf = self.module.get_function("sprintf").unwrap();
        let i8_type = self.context.i8_type();

        // Allocate buffer for result (1024 bytes should be enough)
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
                    let var_ptr = self.variables.get(v).unwrap();
                    args.push((*var_ptr).into());
                }
            }
        }

        let format_str = self.create_string_literal(&format, "concat_fmt");
        args.insert(1, format_str.into());

        // Call sprintf
        self.builder
            .build_call(sprintf, &args, "sprintf_call")
            .unwrap();

        // Store result pointer
        self.variables.insert(var_name.to_string(), buffer);
    }
}
