use std::collections::HashMap;
use std::path::Path;

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetData, TargetTriple,
};
use inkwell::types::{AnyTypeEnum, BasicMetadataTypeEnum, BasicType, BasicTypeEnum};
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, PointerValue};
use inkwell::AddressSpace;
use inkwell::FloatPredicate;
use inkwell::IntPredicate;
use inkwell::OptimizationLevel;

use crate::ast::{BinaryOp, TypeName, UnaryOp};
use crate::ir::{IrConstValue, IrFunction, IrInstr, IrOperand, IrProgram, TempId};
use crate::semantic::{SemanticType, SymbolId};

pub struct BackendArtifacts {
    pub llvm_ir: String,
    pub triple: String,
    pub assembly_path: String,
}

pub fn generate_backend_artifacts(program: &IrProgram) -> Result<BackendArtifacts, String> {
    Target::initialize_aarch64(&InitializationConfig::default());

    let triple = TargetTriple::create("aarch64-unknown-linux-gnu");
    let target = Target::from_triple(&triple).map_err(|err| err.to_string())?;
    let machine = target
        .create_target_machine(
            &triple,
            "generic",
            "",
            OptimizationLevel::Default,
            RelocMode::Default,
            CodeModel::Default,
        )
        .ok_or_else(|| "failed to create AArch64 target machine".to_string())?;

    let context = Context::create();
    let module = context.create_module("sysy_module");
    module.set_triple(&triple);
    module.set_data_layout(&machine.get_target_data().get_data_layout());

    let target_data = machine.get_target_data();
    let mut codegen = Codegen::new(&context, &module, &target_data);
    codegen.declare_runtime();
    codegen.declare_program(program)?;
    codegen.emit_globals(program)?;
    codegen.emit_functions(program)?;

    module.verify().map_err(|err| err.to_string())?;

    let assembly_path = Path::new("output_aarch64.s");
    machine
        .write_to_file(&module, FileType::Assembly, assembly_path)
        .map_err(|err| err.to_string())?;

    Ok(BackendArtifacts {
        llvm_ir: module.print_to_string().to_string(),
        triple: triple.as_str().to_string_lossy().into_owned(),
        assembly_path: assembly_path.display().to_string(),
    })
}

struct Codegen<'ctx, 'm> {
    context: &'ctx Context,
    module: &'m Module<'ctx>,
    runtime: HashMap<String, FunctionValue<'ctx>>,
    functions: HashMap<String, FunctionValue<'ctx>>,
    globals: HashMap<SymbolId, Storage<'ctx>>,
}

struct FunctionCodegen<'ctx, 'm> {
    context: &'ctx Context,
    function: FunctionValue<'ctx>,
    builder: Builder<'ctx>,
    entry_builder: Builder<'ctx>,
    blocks: HashMap<String, BasicBlock<'ctx>>,
    storage: HashMap<SymbolId, Storage<'ctx>>,
    temps: HashMap<TempId, BasicValueEnum<'ctx>>,
    functions: &'m HashMap<String, FunctionValue<'ctx>>,
    globals: &'m HashMap<SymbolId, Storage<'ctx>>,
}

#[derive(Clone)]
enum Storage<'ctx> {
    Scalar {
        ptr: PointerValue<'ctx>,
        ty: SemanticType,
    },
    Array {
        ptr: PointerValue<'ctx>,
        element: TypeName,
        dimensions: Vec<usize>,
    },
    ArrayParam {
        ptr: PointerValue<'ctx>,
        element: TypeName,
    },
}

impl<'ctx, 'm> Codegen<'ctx, 'm> {
    fn new(context: &'ctx Context, module: &'m Module<'ctx>, _target_data: &'m TargetData) -> Self {
        Self {
            context,
            module,
            runtime: HashMap::new(),
            functions: HashMap::new(),
            globals: HashMap::new(),
        }
    }

    fn declare_runtime(&mut self) {
        let i32_type = self.context.i32_type();
        let f32_type = self.context.f32_type();
        let void_type = self.context.void_type();
        let ptr_type = self.context.ptr_type(AddressSpace::default());

        self.add_runtime("getint", i32_type.fn_type(&[], false));
        self.add_runtime("getch", i32_type.fn_type(&[], false));
        self.add_runtime("getfloat", f32_type.fn_type(&[], false));
        self.add_runtime("getarray", i32_type.fn_type(&[ptr_type.into()], false));
        self.add_runtime("getfarray", i32_type.fn_type(&[ptr_type.into()], false));
        self.add_runtime("putint", void_type.fn_type(&[i32_type.into()], false));
        self.add_runtime("putch", void_type.fn_type(&[i32_type.into()], false));
        self.add_runtime("putfloat", void_type.fn_type(&[f32_type.into()], false));
        self.add_runtime(
            "putarray",
            void_type.fn_type(&[i32_type.into(), ptr_type.into()], false),
        );
        self.add_runtime(
            "putfarray",
            void_type.fn_type(&[i32_type.into(), ptr_type.into()], false),
        );
        self.add_runtime("starttime", void_type.fn_type(&[], false));
        self.add_runtime("stoptime", void_type.fn_type(&[], false));
    }

    fn add_runtime(&mut self, name: &str, ty: inkwell::types::FunctionType<'ctx>) {
        let function = self.module.add_function(name, ty, None);
        self.runtime.insert(name.to_string(), function);
        self.functions.insert(name.to_string(), function);
    }

    fn declare_program(&mut self, program: &IrProgram) -> Result<(), String> {
        for function in &program.functions {
            let fn_type = self.function_type(function)?;
            let llvm_fn = self.module.add_function(&function.name, fn_type, None);
            self.functions.insert(function.name.clone(), llvm_fn);
        }
        Ok(())
    }

    fn emit_globals(&mut self, program: &IrProgram) -> Result<(), String> {
        for global in &program.globals {
            let global_value = match &global.ty {
                SemanticType::Int => {
                    let llvm_global = self.module.add_global(self.context.i32_type(), None, &global.name);
                    let initializer = match &global.init {
                        Some(IrConstValue::Int(value)) => self.context.i32_type().const_int(*value as u64, true),
                        None => self.context.i32_type().const_zero(),
                        _ => return Err(format!("invalid int initializer for '{}'", global.name)),
                    };
                    llvm_global.set_initializer(&initializer);
                    llvm_global.set_constant(global.is_const);
                    llvm_global
                }
                SemanticType::Float => {
                    let llvm_global = self.module.add_global(self.context.f32_type(), None, &global.name);
                    let initializer = match &global.init {
                        Some(IrConstValue::Float(value)) => self.context.f32_type().const_float(*value as f64),
                        None => self.context.f32_type().const_zero(),
                        _ => return Err(format!("invalid float initializer for '{}'", global.name)),
                    };
                    llvm_global.set_initializer(&initializer);
                    llvm_global.set_constant(global.is_const);
                    llvm_global
                }
                SemanticType::Array { element, .. } => {
                    let array_type = self.array_type(*element, &global.dimensions)?;
                    let llvm_global = self.module.add_global(array_type, None, &global.name);
                    let initializer = match &global.init {
                        Some(init) => self.const_array_initializer(*element, &global.dimensions, init)?,
                        None => array_type.const_zero(),
                    };
                    llvm_global.set_initializer(&initializer);
                    llvm_global.set_constant(global.is_const);
                    llvm_global
                }
                other => return Err(format!("unsupported global type: {:?}", other)),
            };

            let storage = match &global.ty {
                SemanticType::Int | SemanticType::Float => Storage::Scalar {
                    ptr: global_value.as_pointer_value(),
                    ty: global.ty.clone(),
                },
                SemanticType::Array { element, .. } => Storage::Array {
                    ptr: global_value.as_pointer_value(),
                    element: *element,
                    dimensions: global.dimensions.clone(),
                },
                _ => continue,
            };
            self.globals.insert(global.symbol_id, storage);
        }
        Ok(())
    }

    fn emit_functions(&self, program: &IrProgram) -> Result<(), String> {
        for function in &program.functions {
            let llvm_fn = *self
                .functions
                .get(&function.name)
                .ok_or_else(|| format!("missing declared function '{}'", function.name))?;
            let mut function_codegen = FunctionCodegen::new(
                self.context,
                llvm_fn,
                &self.functions,
                &self.globals,
            );
            function_codegen.emit(function)?;
        }
        Ok(())
    }

    fn function_type(&self, function: &IrFunction) -> Result<inkwell::types::FunctionType<'ctx>, String> {
        let params = function
            .params
            .iter()
            .map(|param| self.param_type(&param.ty))
            .collect::<Result<Vec<_>, _>>()?;
        let params = params
            .iter()
            .copied()
            .map(Into::into)
            .collect::<Vec<BasicMetadataTypeEnum<'ctx>>>();

        match function.return_type {
            TypeName::Int => Ok(self.context.i32_type().fn_type(&params, false)),
            TypeName::Float => Ok(self.context.f32_type().fn_type(&params, false)),
            TypeName::Void => Ok(self.context.void_type().fn_type(&params, false)),
        }
    }

    fn param_type(&self, ty: &SemanticType) -> Result<BasicTypeEnum<'ctx>, String> {
        match ty {
            SemanticType::Int => Ok(self.context.i32_type().into()),
            SemanticType::Float => Ok(self.context.f32_type().into()),
            SemanticType::Array { .. } => Ok(self.context.ptr_type(AddressSpace::default()).into()),
            other => Err(format!("unsupported parameter type: {:?}", other)),
        }
    }

    fn element_basic_type(&self, ty: TypeName) -> BasicTypeEnum<'ctx> {
        match ty {
            TypeName::Int => self.context.i32_type().into(),
            TypeName::Float => self.context.f32_type().into(),
            TypeName::Void => self.context.i8_type().into(),
        }
    }

    fn array_type(&self, element: TypeName, dimensions: &[usize]) -> Result<inkwell::types::ArrayType<'ctx>, String> {
        let first = *dimensions.first().ok_or_else(|| "array dimensions missing".to_string())? as u32;
        let mut any = AnyTypeEnum::ArrayType(self.element_basic_type(element).array_type(first));
        for dim in dimensions.iter().skip(1) {
            any = match any {
                AnyTypeEnum::ArrayType(array) => AnyTypeEnum::ArrayType(array.array_type(*dim as u32)),
                _ => unreachable!(),
            };
        }
        match any {
            AnyTypeEnum::ArrayType(array) => Ok(array),
            _ => Err("failed to build array type".to_string()),
        }
    }

    fn const_array_initializer(
        &self,
        element: TypeName,
        dimensions: &[usize],
        init: &IrConstValue,
    ) -> Result<inkwell::values::ArrayValue<'ctx>, String> {
        let elem_ty = self.element_basic_type(element);
        self.const_array_recursive(elem_ty, dimensions, init)
    }

    fn const_array_recursive(
        &self,
        element_type: BasicTypeEnum<'ctx>,
        dimensions: &[usize],
        init: &IrConstValue,
    ) -> Result<inkwell::values::ArrayValue<'ctx>, String> {
        let len = *dimensions.first().ok_or_else(|| "missing array dimensions".to_string())?;
        if dimensions.len() == 1 {
            let values = match init {
                IrConstValue::List(values) => values,
                _ => return Err("expected list initializer".to_string()),
            };
            let mut elements = Vec::with_capacity(len);
            for index in 0..len {
                let value = values.get(index).cloned().unwrap_or(match element_type {
                    BasicTypeEnum::IntType(_) => IrConstValue::Int(0),
                    BasicTypeEnum::FloatType(_) => IrConstValue::Float(0.0),
                    _ => return Err("unsupported element type".to_string()),
                });
                elements.push(self.const_scalar(&value, element_type)?);
            }
            Ok(match element_type {
                BasicTypeEnum::IntType(ty) => ty.const_array(
                    &elements.into_iter().map(|v| v.into_int_value()).collect::<Vec<_>>(),
                ),
                BasicTypeEnum::FloatType(ty) => ty.const_array(
                    &elements
                        .into_iter()
                        .map(|v| v.into_float_value())
                        .collect::<Vec<_>>(),
                ),
                _ => return Err("unsupported array element type".to_string()),
            })
        } else {
            let values = match init {
                IrConstValue::List(values) => values,
                _ => return Err("expected nested list initializer".to_string()),
            };
            let inner = &dimensions[1..];
            let inner_ty = self.array_type_from_basic(element_type, inner)?;
            let mut elements = Vec::with_capacity(len);
            for index in 0..len {
                let value = values.get(index).cloned().unwrap_or_else(|| zero_list(inner.len()));
                elements.push(self.const_array_recursive(element_type, inner, &value)?);
            }
            Ok(inner_ty.const_array(&elements))
        }
    }

    fn array_type_from_basic(
        &self,
        element_type: BasicTypeEnum<'ctx>,
        dimensions: &[usize],
    ) -> Result<inkwell::types::ArrayType<'ctx>, String> {
        let first = *dimensions.first().ok_or_else(|| "missing inner dimensions".to_string())? as u32;
        let mut any = AnyTypeEnum::ArrayType(element_type.array_type(first));
        for dim in dimensions.iter().skip(1) {
            any = match any {
                AnyTypeEnum::ArrayType(array) => AnyTypeEnum::ArrayType(array.array_type(*dim as u32)),
                _ => unreachable!(),
            };
        }
        match any {
            AnyTypeEnum::ArrayType(array) => Ok(array),
            _ => Err("failed to build nested array type".to_string()),
        }
    }

    fn const_scalar(
        &self,
        value: &IrConstValue,
        expected: BasicTypeEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        match (value, expected) {
            (IrConstValue::Int(value), BasicTypeEnum::IntType(ty)) => Ok(ty.const_int(*value as u64, true).into()),
            (IrConstValue::Float(value), BasicTypeEnum::FloatType(ty)) => Ok(ty.const_float(*value as f64).into()),
            (IrConstValue::Int(value), BasicTypeEnum::FloatType(ty)) => Ok(ty.const_float(*value as f64).into()),
            _ => Err("constant type mismatch".to_string()),
        }
    }
}

impl<'ctx, 'm> FunctionCodegen<'ctx, 'm> {
    fn new(
        context: &'ctx Context,
        function: FunctionValue<'ctx>,
        functions: &'m HashMap<String, FunctionValue<'ctx>>,
        globals: &'m HashMap<SymbolId, Storage<'ctx>>,
    ) -> Self {
        let entry = context.append_basic_block(function, "entry");
        let builder = context.create_builder();
        builder.position_at_end(entry);
        let entry_builder = context.create_builder();
        entry_builder.position_at_end(entry);

        let mut blocks = HashMap::new();
        blocks.insert("entry".to_string(), entry);

        Self {
            context,
            function,
            builder,
            entry_builder,
            blocks,
            storage: HashMap::new(),
            temps: HashMap::new(),
            functions,
            globals,
        }
    }

    fn emit(&mut self, function: &IrFunction) -> Result<(), String> {
        self.create_blocks(function);
        self.bind_params(function)?;

        for instruction in &function.instructions {
            self.emit_instr(instruction)?;
        }

        Ok(())
    }

    fn create_blocks(&mut self, function: &IrFunction) {
        for instruction in &function.instructions {
            if let IrInstr::Label(label) = instruction {
                if !self.blocks.contains_key(label) {
                    let block = self.context.append_basic_block(self.function, label);
                    self.blocks.insert(label.clone(), block);
                }
            }
        }
    }

    fn bind_params(&mut self, function: &IrFunction) -> Result<(), String> {
        for (index, param) in function.params.iter().enumerate() {
            let value = self
                .function
                .get_nth_param(index as u32)
                .ok_or_else(|| format!("missing param {} for '{}'", index, function.name))?;
            value.set_name(&param.name);
            match &param.ty {
                SemanticType::Int | SemanticType::Float => {
                    let ptr = self.alloca_in_entry(self.scalar_type(&param.ty)?, &param.name)?;
                    self.builder.build_store(ptr, value).map_err(|err| err.to_string())?;
                    self.storage.insert(
                        param.symbol_id,
                        Storage::Scalar {
                            ptr,
                            ty: param.ty.clone(),
                        },
                    );
                }
                SemanticType::Array { element, .. } => {
                    self.storage.insert(
                        param.symbol_id,
                        Storage::ArrayParam {
                            ptr: value.into_pointer_value(),
                            element: *element,
                        },
                    );
                }
                other => return Err(format!("unsupported parameter type: {:?}", other)),
            }
        }
        Ok(())
    }

    fn emit_instr(&mut self, instruction: &IrInstr) -> Result<(), String> {
        match instruction {
            IrInstr::Label(label) => {
                let block = *self
                    .blocks
                    .get(label)
                    .ok_or_else(|| format!("missing block '{}'", label))?;
                self.builder.position_at_end(block);
            }
            IrInstr::Declare {
                symbol_id,
                name,
                ty,
                dimensions,
                ..
            } => match ty {
                SemanticType::Int | SemanticType::Float => {
                    let ptr = self.alloca_in_entry(self.scalar_type(ty)?, name)?;
                    self.storage.insert(
                        *symbol_id,
                        Storage::Scalar {
                            ptr,
                            ty: ty.clone(),
                        },
                    );
                }
                SemanticType::Array { element, .. } => {
                    let array_ty = self.array_type(*element, dimensions)?;
                    let ptr = self.alloca_in_entry(array_ty, name)?;
                    self.storage.insert(
                        *symbol_id,
                        Storage::Array {
                            ptr,
                            element: *element,
                            dimensions: dimensions.clone(),
                        },
                    );
                }
                other => return Err(format!("unsupported local type: {:?}", other)),
            },
            IrInstr::LoadVar { dest, symbol_id, .. } => {
                let value = match self.lookup_storage(*symbol_id)? {
                    Storage::Scalar { ptr, ty } => self
                        .builder
                        .build_load(self.scalar_type(&ty)?, ptr, &format!("t{}", dest.0))
                        .map_err(|err| err.to_string())?,
                    _ => return Err("load var expected scalar storage".to_string()),
                };
                self.temps.insert(*dest, value);
            }
            IrInstr::StoreVar { symbol_id, value, .. } => {
                let value = self.operand_value(value)?;
                match self.lookup_storage(*symbol_id)? {
                    Storage::Scalar { ptr, ty } => {
                        let coerced = self.coerce_scalar(value, &ty)?;
                        self.builder.build_store(ptr, coerced).map_err(|err| err.to_string())?;
                    }
                    _ => return Err("store var expected scalar storage".to_string()),
                }
            }
            IrInstr::LoadIndex {
                dest,
                symbol_id,
                indices,
                element_type,
                ..
            } => {
                let ptr = self.index_ptr(*symbol_id, indices, *element_type)?;
                let value = self
                    .builder
                    .build_load(self.element_basic_type(*element_type), ptr, &format!("t{}", dest.0))
                    .map_err(|err| err.to_string())?;
                self.temps.insert(*dest, value);
            }
            IrInstr::StoreIndex {
                symbol_id,
                indices,
                value,
                ..
            } => {
                let element_type = self.lookup_element_type(*symbol_id)?;
                let ptr = self.index_ptr(*symbol_id, indices, element_type)?;
                let value = self.coerce_scalar(self.operand_value(value)?, &semantic_from_type(element_type))?;
                self.builder.build_store(ptr, value).map_err(|err| err.to_string())?;
            }
            IrInstr::Unary {
                dest,
                op,
                operand,
                ty,
            } => {
                let value = self.operand_value(operand)?;
                let result = self.emit_unary(*op, value, ty)?;
                self.temps.insert(*dest, result);
            }
            IrInstr::Binary {
                dest,
                op,
                left,
                right,
                ty,
            } => {
                let left = self.operand_value(left)?;
                let right = self.operand_value(right)?;
                let result = self.emit_binary(*op, left, right, ty)?;
                self.temps.insert(*dest, result);
            }
            IrInstr::Call {
                dest,
                function,
                args,
                return_type,
            } => {
                let callee = *self
                    .functions
                    .get(function)
                    .ok_or_else(|| format!("unknown function '{}'", function))?;
                let params = callee.get_type().get_param_types();
                let mut lowered_args = Vec::with_capacity(args.len());
                for (arg, param_ty) in args.iter().zip(params.iter()) {
                    let value = self.operand_value(arg)?;
                    lowered_args.push(self.coerce_metadata(value, *param_ty)?);
                }
                let call = self
                    .builder
                    .build_call(callee, &lowered_args, "calltmp")
                    .map_err(|err| err.to_string())?;
                if let Some(dest) = dest {
                    let value = match call.try_as_basic_value() {
                        inkwell::values::ValueKind::Basic(value) => value,
                        inkwell::values::ValueKind::Instruction(_) => {
                            return Err(format!("call to '{}' returned void", function))
                        }
                    };
                    self.temps.insert(*dest, value);
                } else if !matches!(return_type, SemanticType::Void) {
                    return Err(format!("non-void call to '{}' missing destination", function));
                }
            }
            IrInstr::Jump { label } => {
                let block = *self
                    .blocks
                    .get(label)
                    .ok_or_else(|| format!("missing jump target '{}'", label))?;
                if self.builder.get_insert_block().and_then(|b| b.get_terminator()).is_none() {
                    self.builder.build_unconditional_branch(block).map_err(|err| err.to_string())?;
                }
            }
            IrInstr::Branch {
                condition,
                then_label,
                else_label,
            } => {
                let condition = self.operand_value(condition)?;
                let condition = self.to_bool(condition)?;
                let then_block = *self
                    .blocks
                    .get(then_label)
                    .ok_or_else(|| format!("missing branch target '{}'", then_label))?;
                let else_block = *self
                    .blocks
                    .get(else_label)
                    .ok_or_else(|| format!("missing branch target '{}'", else_label))?;
                if self.builder.get_insert_block().and_then(|b| b.get_terminator()).is_none() {
                    self.builder
                        .build_conditional_branch(condition, then_block, else_block)
                        .map_err(|err| err.to_string())?;
                }
            }
            IrInstr::Return(value) => {
                if self.builder.get_insert_block().and_then(|b| b.get_terminator()).is_none() {
                    match value {
                        Some(value) => {
                            let value = self.operand_value(value)?;
                            self.builder.build_return(Some(&value)).map_err(|err| err.to_string())?
                        }
                        None => self.builder.build_return(None).map_err(|err| err.to_string())?,
                    };
                }
            }
        }
        Ok(())
    }

    fn operand_value(&self, operand: &IrOperand) -> Result<BasicValueEnum<'ctx>, String> {
        match operand {
            IrOperand::Temp(temp) => self
                .temps
                .get(temp)
                .copied()
                .ok_or_else(|| format!("missing temp %t{}", temp.0)),
            IrOperand::Int(value) => Ok(self.context.i32_type().const_int(*value as u64, true).into()),
            IrOperand::Float(value) => Ok(self.context.f32_type().const_float(*value as f64).into()),
            IrOperand::Symbol { symbol_id, .. } => self.symbol_pointer(*symbol_id).map(Into::into),
        }
    }

    fn symbol_pointer(&self, symbol_id: SymbolId) -> Result<PointerValue<'ctx>, String> {
        match self.lookup_storage(symbol_id)? {
            Storage::Array {
                ptr,
                element,
                dimensions,
            } => self.array_decay_ptr(ptr, element, &dimensions),
            Storage::ArrayParam { ptr, .. } => Ok(ptr),
            Storage::Scalar { ptr, .. } => Ok(ptr),
        }
    }

    fn array_decay_ptr(
        &self,
        ptr: PointerValue<'ctx>,
        element: TypeName,
        dimensions: &[usize],
    ) -> Result<PointerValue<'ctx>, String> {
        let array_ty = self.array_type(element, dimensions)?;
        let zero = self.context.i32_type().const_zero();
        unsafe {
            self.builder
                .build_gep(array_ty, ptr, &[zero, zero], "arraydecay")
                .map_err(|err| err.to_string())
        }
    }

    fn index_ptr(
        &self,
        symbol_id: SymbolId,
        indices: &[IrOperand],
        element_type: TypeName,
    ) -> Result<PointerValue<'ctx>, String> {
        match self.lookup_storage(symbol_id)? {
            Storage::Array { ptr, dimensions, .. } => {
                let array_ty = self.array_type(element_type, &dimensions)?;
                let mut llvm_indices = vec![self.context.i32_type().const_zero()];
                for operand in indices {
                    llvm_indices.push(self.as_i32(self.operand_value(operand)?)?);
                }
                unsafe {
                    self.builder
                        .build_gep(array_ty, ptr, &llvm_indices, "idxptr")
                        .map_err(|err| err.to_string())
                }
            }
            Storage::ArrayParam { ptr, .. } => {
                let mut llvm_indices = Vec::with_capacity(indices.len());
                for operand in indices {
                    llvm_indices.push(self.as_i32(self.operand_value(operand)?)?);
                }
                let element_ty = self.element_basic_type(element_type);
                unsafe {
                    self.builder
                        .build_gep(element_ty, ptr, &llvm_indices, "idxptr")
                        .map_err(|err| err.to_string())
                }
            }
            _ => Err("indexed symbol is not an array".to_string()),
        }
    }

    fn emit_unary(
        &self,
        op: UnaryOp,
        value: BasicValueEnum<'ctx>,
        ty: &SemanticType,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        match (op, ty) {
            (UnaryOp::Plus, _) => Ok(value),
            (UnaryOp::Minus, SemanticType::Int) => Ok(self
                .builder
                .build_int_neg(self.as_i32(value)?, "ineg")
                .map_err(|err| err.to_string())?
                .into()),
            (UnaryOp::Minus, SemanticType::Float) => Ok(self
                .builder
                .build_float_neg(self.as_f32(value)?, "fneg")
                .map_err(|err| err.to_string())?
                .into()),
            (UnaryOp::Not, _) => {
                let cond = self.to_bool(value)?;
                let inverted = self.builder.build_not(cond, "nottmp").map_err(|err| err.to_string())?;
                Ok(self
                    .builder
                    .build_int_z_extend(inverted, self.context.i32_type(), "booltoi32")
                    .map_err(|err| err.to_string())?
                    .into())
            }
            _ => Err(format!("unsupported unary op {:?}", op)),
        }
    }

    fn emit_binary(
        &self,
        op: BinaryOp,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
        ty: &SemanticType,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        let is_float = matches!(left, BasicValueEnum::FloatValue(_))
            || matches!(right, BasicValueEnum::FloatValue(_))
            || matches!(ty, SemanticType::Float);

        if is_float {
            let left = self.as_f32(left)?;
            let right = self.as_f32(right)?;
            let value = match op {
                BinaryOp::Add => self.builder.build_float_add(left, right, "fadd").map_err(|err| err.to_string())?.into(),
                BinaryOp::Sub => self.builder.build_float_sub(left, right, "fsub").map_err(|err| err.to_string())?.into(),
                BinaryOp::Mul => self.builder.build_float_mul(left, right, "fmul").map_err(|err| err.to_string())?.into(),
                BinaryOp::Div => self.builder.build_float_div(left, right, "fdiv").map_err(|err| err.to_string())?.into(),
                BinaryOp::Eq | BinaryOp::Neq | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                    let pred = match op {
                        BinaryOp::Eq => FloatPredicate::OEQ,
                        BinaryOp::Neq => FloatPredicate::ONE,
                        BinaryOp::Lt => FloatPredicate::OLT,
                        BinaryOp::Le => FloatPredicate::OLE,
                        BinaryOp::Gt => FloatPredicate::OGT,
                        BinaryOp::Ge => FloatPredicate::OGE,
                        _ => unreachable!(),
                    };
                    let cmp = self.builder.build_float_compare(pred, left, right, "fcmp").map_err(|err| err.to_string())?;
                    self.builder.build_int_z_extend(cmp, self.context.i32_type(), "fcmpi32").map_err(|err| err.to_string())?.into()
                }
                BinaryOp::And | BinaryOp::Or => {
                    let left = self.to_bool(left.into())?;
                    let right = self.to_bool(right.into())?;
                    let value = match op {
                        BinaryOp::And => self.builder.build_and(left, right, "and").map_err(|err| err.to_string())?,
                        BinaryOp::Or => self.builder.build_or(left, right, "or").map_err(|err| err.to_string())?,
                        _ => unreachable!(),
                    };
                    self.builder.build_int_z_extend(value, self.context.i32_type(), "logic").map_err(|err| err.to_string())?.into()
                }
                BinaryOp::Mod => return Err("float modulo is unsupported".to_string()),
            };
            Ok(value)
        } else {
            let left = self.as_i32(left)?;
            let right = self.as_i32(right)?;
            let value = match op {
                BinaryOp::Add => self.builder.build_int_add(left, right, "iadd").map_err(|err| err.to_string())?.into(),
                BinaryOp::Sub => self.builder.build_int_sub(left, right, "isub").map_err(|err| err.to_string())?.into(),
                BinaryOp::Mul => self.builder.build_int_mul(left, right, "imul").map_err(|err| err.to_string())?.into(),
                BinaryOp::Div => self.builder.build_int_signed_div(left, right, "idiv").map_err(|err| err.to_string())?.into(),
                BinaryOp::Mod => self.builder.build_int_signed_rem(left, right, "irem").map_err(|err| err.to_string())?.into(),
                BinaryOp::Eq | BinaryOp::Neq | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                    let pred = match op {
                        BinaryOp::Eq => IntPredicate::EQ,
                        BinaryOp::Neq => IntPredicate::NE,
                        BinaryOp::Lt => IntPredicate::SLT,
                        BinaryOp::Le => IntPredicate::SLE,
                        BinaryOp::Gt => IntPredicate::SGT,
                        BinaryOp::Ge => IntPredicate::SGE,
                        _ => unreachable!(),
                    };
                    let cmp = self.builder.build_int_compare(pred, left, right, "icmp").map_err(|err| err.to_string())?;
                    self.builder.build_int_z_extend(cmp, self.context.i32_type(), "icmpi32").map_err(|err| err.to_string())?.into()
                }
                BinaryOp::And => {
                    let value = self
                        .builder
                        .build_and(self.to_bool(left.into())?, self.to_bool(right.into())?, "and")
                        .map_err(|err| err.to_string())?;
                    self.builder.build_int_z_extend(value, self.context.i32_type(), "andi32").map_err(|err| err.to_string())?.into()
                }
                BinaryOp::Or => {
                    let value = self
                        .builder
                        .build_or(self.to_bool(left.into())?, self.to_bool(right.into())?, "or")
                        .map_err(|err| err.to_string())?;
                    self.builder.build_int_z_extend(value, self.context.i32_type(), "ori32").map_err(|err| err.to_string())?.into()
                }
            };
            Ok(value)
        }
    }

    fn coerce_metadata(
        &self,
        value: BasicValueEnum<'ctx>,
        target: BasicMetadataTypeEnum<'ctx>,
    ) -> Result<BasicMetadataValueEnum<'ctx>, String> {
        let value = match target {
            BasicMetadataTypeEnum::IntType(_) => self.as_i32(value)?.into(),
            BasicMetadataTypeEnum::FloatType(_) => self.as_f32(value)?.into(),
            BasicMetadataTypeEnum::PointerType(_) => match value {
                BasicValueEnum::PointerValue(value) => value.into(),
                _ => return Err("expected pointer argument".to_string()),
            },
            _ => return Err("unsupported argument type".to_string()),
        };
        Ok(value)
    }

    fn coerce_scalar(
        &self,
        value: BasicValueEnum<'ctx>,
        target: &SemanticType,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        match target {
            SemanticType::Int => Ok(self.as_i32(value)?.into()),
            SemanticType::Float => Ok(self.as_f32(value)?.into()),
            _ => Err("unsupported scalar coercion target".to_string()),
        }
    }

    fn to_bool(&self, value: BasicValueEnum<'ctx>) -> Result<inkwell::values::IntValue<'ctx>, String> {
        match value {
            BasicValueEnum::IntValue(value) => self
                .builder
                .build_int_compare(IntPredicate::NE, value, value.get_type().const_zero(), "tobool")
                .map_err(|err| err.to_string()),
            BasicValueEnum::FloatValue(value) => self
                .builder
                .build_float_compare(FloatPredicate::ONE, value, value.get_type().const_zero(), "tobool")
                .map_err(|err| err.to_string()),
            _ => Err("unsupported boolean conversion".to_string()),
        }
    }

    fn as_i32(&self, value: BasicValueEnum<'ctx>) -> Result<inkwell::values::IntValue<'ctx>, String> {
        match value {
            BasicValueEnum::IntValue(value) if value.get_type().get_bit_width() == 32 => Ok(value),
            BasicValueEnum::IntValue(value) if value.get_type().get_bit_width() == 1 => self
                .builder
                .build_int_z_extend(value, self.context.i32_type(), "i1toi32")
                .map_err(|err| err.to_string()),
            BasicValueEnum::FloatValue(value) => self
                .builder
                .build_float_to_signed_int(value, self.context.i32_type(), "ftoi")
                .map_err(|err| err.to_string()),
            _ => Err("expected integer-compatible value".to_string()),
        }
    }

    fn as_f32(&self, value: BasicValueEnum<'ctx>) -> Result<inkwell::values::FloatValue<'ctx>, String> {
        match value {
            BasicValueEnum::FloatValue(value) => Ok(value),
            BasicValueEnum::IntValue(value) => self
                .builder
                .build_signed_int_to_float(value, self.context.f32_type(), "itof")
                .map_err(|err| err.to_string()),
            _ => Err("expected float-compatible value".to_string()),
        }
    }

    fn lookup_storage(&self, symbol_id: SymbolId) -> Result<Storage<'ctx>, String> {
        self.storage
            .get(&symbol_id)
            .cloned()
            .or_else(|| self.globals.get(&symbol_id).cloned())
            .ok_or_else(|| format!("missing storage for symbol {}", symbol_id.0))
    }

    fn lookup_element_type(&self, symbol_id: SymbolId) -> Result<TypeName, String> {
        match self.lookup_storage(symbol_id)? {
            Storage::Array { element, .. } | Storage::ArrayParam { element, .. } => Ok(element),
            Storage::Scalar { ty, .. } => match ty {
                SemanticType::Int => Ok(TypeName::Int),
                SemanticType::Float => Ok(TypeName::Float),
                _ => Err("scalar element type unsupported".to_string()),
            },
        }
    }

    fn alloca_in_entry<T: BasicType<'ctx>>(&self, ty: T, name: &str) -> Result<PointerValue<'ctx>, String> {
        self.entry_builder.build_alloca(ty, name).map_err(|err| err.to_string())
    }

    fn scalar_type(&self, ty: &SemanticType) -> Result<BasicTypeEnum<'ctx>, String> {
        match ty {
            SemanticType::Int => Ok(self.context.i32_type().into()),
            SemanticType::Float => Ok(self.context.f32_type().into()),
            other => Err(format!("unsupported scalar type: {:?}", other)),
        }
    }

    fn element_basic_type(&self, ty: TypeName) -> BasicTypeEnum<'ctx> {
        match ty {
            TypeName::Int => self.context.i32_type().into(),
            TypeName::Float => self.context.f32_type().into(),
            TypeName::Void => self.context.i8_type().into(),
        }
    }

    fn array_type(&self, element: TypeName, dimensions: &[usize]) -> Result<inkwell::types::ArrayType<'ctx>, String> {
        let first = *dimensions.first().ok_or_else(|| "array dimensions missing".to_string())? as u32;
        let mut any = AnyTypeEnum::ArrayType(self.element_basic_type(element).array_type(first));
        for dim in dimensions.iter().skip(1) {
            any = match any {
                AnyTypeEnum::ArrayType(array) => AnyTypeEnum::ArrayType(array.array_type(*dim as u32)),
                _ => unreachable!(),
            };
        }
        match any {
            AnyTypeEnum::ArrayType(array) => Ok(array),
            _ => Err("failed to build array type".to_string()),
        }
    }
}

fn semantic_from_type(ty: TypeName) -> SemanticType {
    match ty {
        TypeName::Int => SemanticType::Int,
        TypeName::Float => SemanticType::Float,
        TypeName::Void => SemanticType::Void,
    }
}

fn zero_list(depth: usize) -> IrConstValue {
    if depth <= 1 {
        IrConstValue::List(Vec::new())
    } else {
        IrConstValue::List(vec![zero_list(depth - 1)])
    }
}
