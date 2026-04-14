#![allow(dead_code)]

use std::collections::HashMap;
use std::fmt::{self, Write};

use crate::ast::{BinaryOp, PostfixOp, TypeName, UnaryOp};
use crate::semantic::{
    SemanticBlock, SemanticExpr, SemanticExprKind, SemanticForInit, SemanticInitializer,
    SemanticItem, SemanticProgram, SemanticStmt, SemanticType, SemanticVarDecl, Symbol, SymbolId,
};

#[derive(Debug, Clone)]
pub struct IrProgram {
    pub globals: Vec<IrGlobal>,
    pub functions: Vec<IrFunction>,
}

#[derive(Debug, Clone)]
pub struct IrGlobal {
    pub symbol_id: SymbolId,
    pub name: String,
    pub is_const: bool,
    pub ty: SemanticType,
    pub dimensions: Vec<usize>,
    pub init: Option<IrConstValue>,
}

#[derive(Debug, Clone)]
pub struct IrFunction {
    pub name: String,
    pub return_type: TypeName,
    pub params: Vec<IrParam>,
    pub instructions: Vec<IrInstr>,
}

#[derive(Debug, Clone)]
pub struct IrParam {
    pub symbol_id: SymbolId,
    pub name: String,
    pub ty: SemanticType,
    pub dimensions: Vec<Option<usize>>,
}

#[derive(Debug, Clone)]
pub enum IrInstr {
    Label(String),
    Declare {
        symbol_id: SymbolId,
        name: String,
        is_const: bool,
        ty: SemanticType,
        dimensions: Vec<usize>,
    },
    LoadVar {
        dest: TempId,
        symbol_id: SymbolId,
        name: String,
        ty: SemanticType,
    },
    StoreVar {
        symbol_id: SymbolId,
        name: String,
        value: IrOperand,
    },
    LoadIndex {
        dest: TempId,
        symbol_id: SymbolId,
        name: String,
        indices: Vec<IrOperand>,
        element_type: TypeName,
    },
    StoreIndex {
        symbol_id: SymbolId,
        name: String,
        indices: Vec<IrOperand>,
        value: IrOperand,
    },
    Unary {
        dest: TempId,
        op: UnaryOp,
        operand: IrOperand,
        ty: SemanticType,
    },
    Binary {
        dest: TempId,
        op: BinaryOp,
        left: IrOperand,
        right: IrOperand,
        ty: SemanticType,
    },
    Call {
        dest: Option<TempId>,
        function: String,
        args: Vec<IrOperand>,
        return_type: SemanticType,
    },
    Jump {
        label: String,
    },
    Branch {
        condition: IrOperand,
        then_label: String,
        else_label: String,
    },
    Return(Option<IrOperand>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TempId(pub usize);

#[derive(Debug, Clone, PartialEq)]
pub enum IrOperand {
    Temp(TempId),
    Int(i32),
    Float(f32),
    Symbol {
        symbol_id: SymbolId,
        name: String,
    },
}

#[derive(Debug, Clone)]
pub enum IrConstValue {
    Int(i32),
    Float(f32),
    List(Vec<IrConstValue>),
}

// 将语义程序降低为 IR
pub fn lower_program(program: &SemanticProgram) -> Result<IrProgram, String> {
    let mut lowerer = Lowerer::new(&program.symbols);
    lowerer.lower_program(program)
}

// 格式化整个 IR 程序
pub fn format_program(program: &IrProgram) -> String {
    let mut out = String::new();

    if !program.globals.is_empty() {
        out.push_str("globals:\n");
        for global in &program.globals {
            let _ = writeln!(
                out,
                "  @{} : {}{}{}",
                global.name,
                format_semantic_type(&global.ty),
                if global.is_const { " const" } else { "" },
                format_global_init(global)
            );
        }
        out.push('\n');
    }

    for function in &program.functions {
        let params = function
            .params
            .iter()
            .map(|param| format!("{} {}", format_semantic_type(&param.ty), param.name))
            .collect::<Vec<_>>()
            .join(", ");
        let _ = writeln!(
            out,
            "fn {}({}) -> {} {{",
            function.name,
            params,
            format_type_name(function.return_type)
        );
        for instruction in &function.instructions {
            let _ = writeln!(out, "{}", format_instruction(instruction));
        }
        out.push_str("}\n\n");
    }

    out
}

// 格式化全局变量初值
fn format_global_init(global: &IrGlobal) -> String {
    match &global.init {
        Some(init) => format!(" = {}", format_const_value(init)),
        None => String::new(),
    }
}

// 格式化常量值
fn format_const_value(value: &IrConstValue) -> String {
    match value {
        IrConstValue::Int(value) => value.to_string(),
        IrConstValue::Float(value) => format_float(*value),
        IrConstValue::List(values) => {
            let parts = values.iter().map(format_const_value).collect::<Vec<_>>().join(", ");
            format!("[{}]", parts)
        }
    }
}

// 格式化单条 IR 指令
pub fn format_instruction(instruction: &IrInstr) -> String {
    match instruction {
        IrInstr::Label(label) => format!("{}:", label),
        IrInstr::Declare {
            name,
            is_const,
            ty,
            dimensions,
            ..
        } => {
            if dimensions.is_empty() {
                format!(
                    "  decl {}{} {}",
                    if *is_const { "const " } else { "" },
                    format_semantic_type(ty),
                    name
                )
            } else {
                let dims = dimensions
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>()
                    .join("][");
                format!(
                    "  decl {}{} {}[{}]",
                    if *is_const { "const " } else { "" },
                    format_semantic_type(ty),
                    name,
                    dims
                )
            }
        }
        IrInstr::LoadVar { dest, name, .. } => {
            format!("  %t{} = load {}", dest.0, name)
        }
        IrInstr::StoreVar { name, value, .. } => {
            format!("  store {} -> {}", format_operand(value), name)
        }
        IrInstr::LoadIndex {
            dest,
            name,
            indices,
            ..
        } => {
            format!(
                "  %t{} = load {}{}",
                dest.0,
                name,
                format_indices(indices)
            )
        }
        IrInstr::StoreIndex {
            name,
            indices,
            value,
            ..
        } => {
            format!(
                "  store {} -> {}{}",
                format_operand(value),
                name,
                format_indices(indices)
            )
        }
        IrInstr::Unary { dest, op, operand, .. } => format!(
            "  %t{} = {:?} {}",
            dest.0,
            op,
            format_operand(operand)
        ),
        IrInstr::Binary {
            dest,
            op,
            left,
            right,
            ..
        } => format!(
            "  %t{} = {:?} {}, {}",
            dest.0,
            op,
            format_operand(left),
            format_operand(right)
        ),
        IrInstr::Call {
            dest,
            function,
            args,
            ..
        } => {
            let args = args.iter().map(format_operand).collect::<Vec<_>>().join(", ");
            match dest {
                Some(dest) => format!("  %t{} = call {}({})", dest.0, function, args),
                None => format!("  call {}({})", function, args),
            }
        }
        IrInstr::Jump { label } => format!("  jump {}", label),
        IrInstr::Branch {
            condition,
            then_label,
            else_label,
        } => format!(
            "  branch {} ? {} : {}",
            format_operand(condition),
            then_label,
            else_label
        ),
        IrInstr::Return(Some(value)) => format!("  return {}", format_operand(value)),
        IrInstr::Return(None) => "  return".to_string(),
    }
}

// 格式化操作数
pub fn format_operand(operand: &IrOperand) -> String {
    match operand {
        IrOperand::Temp(temp) => format!("%t{}", temp.0),
        IrOperand::Int(value) => value.to_string(),
        IrOperand::Float(value) => format_float(*value),
        IrOperand::Symbol { name, .. } => format!("&{}", name),
    }
}

// 格式化数组下标
fn format_indices(indices: &[IrOperand]) -> String {
    let mut out = String::new();
    for index in indices {
        let _ = write!(out, "[{}]", format_operand(index));
    }
    out
}

// 格式化类型名称
fn format_type_name(ty: TypeName) -> &'static str {
    match ty {
        TypeName::Int => "int",
        TypeName::Float => "float",
        TypeName::Void => "void",
    }
}

// 格式化语义类型
pub fn format_semantic_type(ty: &SemanticType) -> String {
    match ty {
        SemanticType::Int => "int".to_string(),
        SemanticType::Float => "float".to_string(),
        SemanticType::Void => "void".to_string(),
        SemanticType::Array { element, rank } => {
            format!("{}[rank={}]", format_type_name(*element), rank)
        }
        SemanticType::Function {
            return_type,
            params,
        } => {
            let params = params
                .iter()
                .map(format_semantic_type)
                .collect::<Vec<_>>()
                .join(", ");
            format!("fn({}) -> {}", params, format_type_name(*return_type))
        }
        SemanticType::Error => "error".to_string(),
    }
}

// 格式化浮点数
fn format_float(value: f32) -> String {
    format!("{:.6}", value)
}

struct Lowerer<'a> {
    symbols: &'a [Symbol],
    const_values: HashMap<SymbolId, IrConstValue>,
}

struct FunctionLowerer<'a> {
    symbols: &'a [Symbol],
    const_values: &'a mut HashMap<SymbolId, IrConstValue>,
    instructions: Vec<IrInstr>,
    temp_counter: usize,
    label_counter: usize,
    loop_stack: Vec<LoopLabels>,
}

#[derive(Clone)]
struct LoopLabels {
    break_label: String,
    continue_label: String,
}

enum LValue {
    Var {
        symbol_id: SymbolId,
        name: String,
    },
    Index {
        symbol_id: SymbolId,
        name: String,
        indices: Vec<IrOperand>,
        element_type: TypeName,
    },
}

impl<'a> Lowerer<'a> {
    // 创建 IR 降低器
    fn new(symbols: &'a [Symbol]) -> Self {
        Self {
            symbols,
            const_values: HashMap::new(),
        }
    }

    // 降低整个程序
    fn lower_program(&mut self, program: &SemanticProgram) -> Result<IrProgram, String> {
        let mut globals = Vec::new();
        let mut functions = Vec::new();

        for item in &program.items {
            match item {
                SemanticItem::GlobalDecl(decls) => {
                    for decl in decls {
                        globals.push(self.lower_global(decl)?);
                    }
                }
                SemanticItem::Function(function) => {
                    functions.push(self.lower_function(function)?);
                }
            }
        }

        Ok(IrProgram { globals, functions })
    }

    // 降低全局变量声明
    fn lower_global(&mut self, decl: &SemanticVarDecl) -> Result<IrGlobal, String> {
        let symbol_id = decl
            .symbol_id
            .ok_or_else(|| format!("missing symbol for global '{}'", decl.name))?;
        let dimensions = self.dimensions_to_usize(&decl.dimensions)?;
        let init = match &decl.init {
            Some(init) => Some(self.lower_const_initializer(init)?),
            None => None,
        };

        if decl.is_const {
            if let Some(init) = &init {
                self.const_values.insert(symbol_id, init.clone());
            }
        }

        Ok(IrGlobal {
            symbol_id,
            name: decl.name.clone(),
            is_const: decl.is_const,
            ty: decl.ty.clone(),
            dimensions,
            init,
        })
    }

    // 降低函数定义
    fn lower_function(
        &mut self,
        function: &crate::semantic::SemanticFunction,
    ) -> Result<IrFunction, String> {
        let params = function
            .params
            .iter()
            .map(|param| {
                Ok(IrParam {
                    symbol_id: param
                        .symbol_id
                        .ok_or_else(|| format!("missing symbol for parameter '{}'", param.name))?,
                    name: param.name.clone(),
                    ty: param.ty.clone(),
                    dimensions: param
                        .dimensions
                        .iter()
                        .map(|dimension| {
                            dimension
                                .as_ref()
                                .map(|expr| self.eval_const_expr(expr).and_then(as_usize_const))
                                .transpose()
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        let mut function_lowerer = FunctionLowerer {
            symbols: self.symbols,
            const_values: &mut self.const_values,
            instructions: vec![IrInstr::Label("entry".to_string())],
            temp_counter: 0,
            label_counter: 0,
            loop_stack: Vec::new(),
        };
        function_lowerer.lower_block(&function.body)?;
        if !function_lowerer.is_terminated() {
            match function.return_type {
                TypeName::Void => function_lowerer.instructions.push(IrInstr::Return(None)),
                TypeName::Int => function_lowerer
                    .instructions
                    .push(IrInstr::Return(Some(IrOperand::Int(0)))),
                TypeName::Float => function_lowerer
                    .instructions
                    .push(IrInstr::Return(Some(IrOperand::Float(0.0)))),
            }
        }

        Ok(IrFunction {
            name: function.name.clone(),
            return_type: function.return_type,
            params,
            instructions: function_lowerer.instructions,
        })
    }

    // 计算数组维度值
    fn dimensions_to_usize(&self, dimensions: &[Option<SemanticExpr>]) -> Result<Vec<usize>, String> {
        dimensions
            .iter()
            .map(|dimension| {
                let expr = dimension
                    .as_ref()
                    .ok_or_else(|| "missing array dimension".to_string())?;
                let value = self.eval_const_expr(expr)?;
                as_usize_const(value)
            })
            .collect()
    }

    // 降低常量初始化器
    fn lower_const_initializer(&self, init: &SemanticInitializer) -> Result<IrConstValue, String> {
        match init {
            SemanticInitializer::Expr(expr) => self.eval_const_expr(expr),
            SemanticInitializer::List(items) => Ok(IrConstValue::List(
                items.iter()
                    .map(|item| self.lower_const_initializer(item))
                    .collect::<Result<Vec<_>, _>>()?,
            )),
        }
    }

    // 计算常量表达式
    fn eval_const_expr(&self, expr: &SemanticExpr) -> Result<IrConstValue, String> {
        match &expr.kind {
            SemanticExprKind::IntLiteral(value) => value
                .parse::<i32>()
                .map(IrConstValue::Int)
                .map_err(|_| format!("invalid int literal '{}'", value)),
            SemanticExprKind::FloatLiteral(value) => value
                .parse::<f32>()
                .map(IrConstValue::Float)
                .map_err(|_| format!("invalid float literal '{}'", value)),
            SemanticExprKind::Ident { symbol_id, .. } => {
                let symbol_id = symbol_id.ok_or_else(|| "missing symbol in const expr".to_string())?;
                self.const_values
                    .get(&symbol_id)
                    .cloned()
                    .ok_or_else(|| "identifier is not a known constant".to_string())
            }
            SemanticExprKind::Unary { op, expr } => {
                let value = self.eval_const_expr(expr)?;
                fold_unary_const(*op, value)
            }
            SemanticExprKind::Binary { op, left, right } => {
                let left = self.eval_const_expr(left)?;
                let right = self.eval_const_expr(right)?;
                fold_binary_const(*op, left, right)
            }
            SemanticExprKind::Index { array, index } => {
                let array = self.eval_const_expr(array)?;
                let index = match self.eval_const_expr(index)? {
                    IrConstValue::Int(value) if value >= 0 => value as usize,
                    _ => return Err("constant index must be a non-negative integer".to_string()),
                };
                match array {
                    IrConstValue::List(values) => values
                        .get(index)
                        .cloned()
                        .ok_or_else(|| "constant array index out of bounds".to_string()),
                    _ => Err("indexed constant is not an array".to_string()),
                }
            }
            _ => Err("unsupported constant expression".to_string()),
        }
    }
}

impl<'a> FunctionLowerer<'a> {
    // 降低语句块
    fn lower_block(&mut self, block: &SemanticBlock) -> Result<(), String> {
        for stmt in &block.statements {
            if self.is_terminated() {
                break;
            }
            self.lower_stmt(stmt)?;
        }
        Ok(())
    }

    // 降低单条语句
    fn lower_stmt(&mut self, stmt: &SemanticStmt) -> Result<(), String> {
        match stmt {
            SemanticStmt::VarDecl(decls) => {
                for decl in decls {
                    self.lower_decl(decl)?;
                }
            }
            SemanticStmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let then_label = self.new_label("if_then");
                let else_label = self.new_label("if_else");
                let end_label = self.new_label("if_end");
                let condition = self.lower_expr(condition)?;
                self.instructions.push(IrInstr::Branch {
                    condition,
                    then_label: then_label.clone(),
                    else_label: else_label.clone(),
                });

                self.instructions.push(IrInstr::Label(then_label));
                self.lower_stmt(then_branch)?;
                if !self.is_terminated() {
                    self.instructions.push(IrInstr::Jump {
                        label: end_label.clone(),
                    });
                }

                self.instructions.push(IrInstr::Label(else_label));
                if let Some(else_branch) = else_branch {
                    self.lower_stmt(else_branch)?;
                }
                if !self.is_terminated() {
                    self.instructions.push(IrInstr::Jump {
                        label: end_label.clone(),
                    });
                }

                self.instructions.push(IrInstr::Label(end_label));
            }
            SemanticStmt::While { condition, body } => {
                let cond_label = self.new_label("while_cond");
                let body_label = self.new_label("while_body");
                let end_label = self.new_label("while_end");

                self.instructions.push(IrInstr::Jump {
                    label: cond_label.clone(),
                });
                self.instructions.push(IrInstr::Label(cond_label.clone()));
                let condition = self.lower_expr(condition)?;
                self.instructions.push(IrInstr::Branch {
                    condition,
                    then_label: body_label.clone(),
                    else_label: end_label.clone(),
                });

                self.loop_stack.push(LoopLabels {
                    break_label: end_label.clone(),
                    continue_label: cond_label.clone(),
                });
                self.instructions.push(IrInstr::Label(body_label));
                self.lower_stmt(body)?;
                if !self.is_terminated() {
                    self.instructions.push(IrInstr::Jump { label: cond_label });
                }
                self.loop_stack.pop();
                self.instructions.push(IrInstr::Label(end_label));
            }
            SemanticStmt::For {
                init,
                condition,
                update,
                body,
                ..
            } => {
                if let Some(init) = init {
                    match init {
                        SemanticForInit::Decl(decls) => {
                            for decl in decls {
                                self.lower_decl(decl)?;
                            }
                        }
                        SemanticForInit::Expr(expr) => {
                            let _ = self.lower_expr(expr)?;
                        }
                    }
                }

                let cond_label = self.new_label("for_cond");
                let body_label = self.new_label("for_body");
                let update_label = self.new_label("for_update");
                let end_label = self.new_label("for_end");
                self.instructions.push(IrInstr::Jump {
                    label: cond_label.clone(),
                });
                self.instructions.push(IrInstr::Label(cond_label.clone()));
                match condition {
                    Some(condition) => {
                        let condition = self.lower_expr(condition)?;
                        self.instructions.push(IrInstr::Branch {
                            condition,
                            then_label: body_label.clone(),
                            else_label: end_label.clone(),
                        });
                    }
                    None => self.instructions.push(IrInstr::Jump {
                        label: body_label.clone(),
                    }),
                }

                self.loop_stack.push(LoopLabels {
                    break_label: end_label.clone(),
                    continue_label: update_label.clone(),
                });
                self.instructions.push(IrInstr::Label(body_label));
                self.lower_stmt(body)?;
                if !self.is_terminated() {
                    self.instructions.push(IrInstr::Jump {
                        label: update_label.clone(),
                    });
                }

                self.loop_stack.pop();
                self.instructions.push(IrInstr::Label(update_label.clone()));
                if let Some(update) = update {
                    let _ = self.lower_expr(update)?;
                }
                if !self.is_terminated() {
                    self.instructions.push(IrInstr::Jump { label: cond_label });
                }
                self.instructions.push(IrInstr::Label(end_label));
            }
            SemanticStmt::Return(expr) => {
                let value = expr.as_ref().map(|expr| self.lower_expr(expr)).transpose()?;
                self.instructions.push(IrInstr::Return(value));
            }
            SemanticStmt::Break => {
                let label = self
                    .loop_stack
                    .last()
                    .ok_or_else(|| "break outside loop".to_string())?
                    .break_label
                    .clone();
                self.instructions.push(IrInstr::Jump { label });
            }
            SemanticStmt::Continue => {
                let label = self
                    .loop_stack
                    .last()
                    .ok_or_else(|| "continue outside loop".to_string())?
                    .continue_label
                    .clone();
                self.instructions.push(IrInstr::Jump { label });
            }
            SemanticStmt::Expr(expr) => {
                let _ = self.lower_expr(expr)?;
            }
            SemanticStmt::Block(block) => self.lower_block(block)?,
            SemanticStmt::Empty => {}
        }
        Ok(())
    }

    // 降低变量声明
    fn lower_decl(&mut self, decl: &SemanticVarDecl) -> Result<(), String> {
        let symbol_id = decl
            .symbol_id
            .ok_or_else(|| format!("missing symbol for variable '{}'", decl.name))?;
        let dimensions = decl
            .dimensions
            .iter()
            .map(|dimension| {
                dimension
                    .as_ref()
                    .map(|expr| self.eval_const_expr(expr).and_then(as_usize_const))
                    .transpose()
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        self.instructions.push(IrInstr::Declare {
            symbol_id,
            name: decl.name.clone(),
            is_const: decl.is_const,
            ty: decl.ty.clone(),
            dimensions: dimensions.clone(),
        });

        if let Some(init) = &decl.init {
            self.lower_initializer(decl, init, &[])?;
        }

        if decl.is_const {
            if let Some(init) = &decl.init {
                if let Ok(value) = self.lower_const_initializer(init) {
                    self.const_values.insert(symbol_id, value);
                }
            }
        }
        Ok(())
    }

    // 降低初始化器内容
    fn lower_initializer(
        &mut self,
        decl: &SemanticVarDecl,
        init: &SemanticInitializer,
        prefix: &[usize],
    ) -> Result<(), String> {
        let symbol_id = decl
            .symbol_id
            .ok_or_else(|| format!("missing symbol for variable '{}'", decl.name))?;
        match init {
            SemanticInitializer::Expr(expr) => {
                let value = self.lower_expr(expr)?;
                if prefix.is_empty() && !matches!(decl.ty, SemanticType::Array { .. }) {
                    self.instructions.push(IrInstr::StoreVar {
                        symbol_id,
                        name: decl.name.clone(),
                        value,
                    });
                } else {
                    let indices = prefix
                        .iter()
                        .map(|value| IrOperand::Int(*value as i32))
                        .collect::<Vec<_>>();
                    self.instructions.push(IrInstr::StoreIndex {
                        symbol_id,
                        name: decl.name.clone(),
                        indices,
                        value,
                    });
                }
            }
            SemanticInitializer::List(items) => {
                for (index, item) in items.iter().enumerate() {
                    let mut next_prefix = prefix.to_vec();
                    next_prefix.push(index);
                    self.lower_initializer(decl, item, &next_prefix)?;
                }
            }
        }
        Ok(())
    }

    // 降低表达式
    fn lower_expr(&mut self, expr: &SemanticExpr) -> Result<IrOperand, String> {
        if let Ok(constant) = self.eval_const_expr(expr) {
            return const_to_operand(&constant);
        }

        match &expr.kind {
            SemanticExprKind::Ident { name, symbol_id } => {
                let symbol_id = symbol_id.ok_or_else(|| format!("missing symbol for '{}'", name))?;
                if matches!(expr.ty, SemanticType::Array { .. }) {
                    Ok(IrOperand::Symbol {
                        symbol_id,
                        name: name.clone(),
                    })
                } else {
                    let dest = self.new_temp();
                    self.instructions.push(IrInstr::LoadVar {
                        dest,
                        symbol_id,
                        name: name.clone(),
                        ty: expr.ty.clone(),
                    });
                    Ok(IrOperand::Temp(dest))
                }
            }
            SemanticExprKind::IntLiteral(value) => value
                .parse::<i32>()
                .map(IrOperand::Int)
                .map_err(|_| format!("invalid int literal '{}'", value)),
            SemanticExprKind::FloatLiteral(value) => value
                .parse::<f32>()
                .map(IrOperand::Float)
                .map_err(|_| format!("invalid float literal '{}'", value)),
            SemanticExprKind::Unary { op, expr: inner } => {
                match op {
                    UnaryOp::PreInc | UnaryOp::PreDec => {
                        let lvalue = self.resolve_lvalue(inner)?;
                        let current = self.load_lvalue(&lvalue)?;
                        let updated = self.emit_binary(
                            if matches!(op, UnaryOp::PreInc) {
                                BinaryOp::Add
                            } else {
                                BinaryOp::Sub
                            },
                            current.clone(),
                            one_for_type(&expr.ty),
                            expr.ty.clone(),
                        );
                        self.store_lvalue(&lvalue, updated.clone());
                        Ok(updated)
                    }
                    UnaryOp::Plus | UnaryOp::Minus | UnaryOp::Not => {
                        let operand = self.lower_expr(inner)?;
                        let dest = self.new_temp();
                        self.instructions.push(IrInstr::Unary {
                            dest,
                            op: *op,
                            operand,
                            ty: expr.ty.clone(),
                        });
                        Ok(IrOperand::Temp(dest))
                    }
                }
            }
            SemanticExprKind::Binary { op, left, right } => {
                let left = self.lower_expr(left)?;
                let right = self.lower_expr(right)?;
                Ok(self.emit_binary(*op, left, right, expr.ty.clone()))
            }
            SemanticExprKind::Assign { target, value } => {
                let lvalue = self.resolve_lvalue(target)?;
                let value = self.lower_expr(value)?;
                self.store_lvalue(&lvalue, value.clone());
                Ok(value)
            }
            SemanticExprKind::Call {
                callee,
                args,
                function_symbol: _,
            } => {
                let function = match &callee.kind {
                    SemanticExprKind::Ident { name, .. } => name.clone(),
                    _ => return Err("unsupported callee expression".to_string()),
                };
                let args = args
                    .iter()
                    .map(|arg| self.lower_expr(arg))
                    .collect::<Result<Vec<_>, _>>()?;
                let dest = if matches!(expr.ty, SemanticType::Void) {
                    None
                } else {
                    Some(self.new_temp())
                };
                self.instructions.push(IrInstr::Call {
                    dest,
                    function,
                    args,
                    return_type: expr.ty.clone(),
                });
                Ok(dest.map(IrOperand::Temp).unwrap_or(IrOperand::Int(0)))
            }
            SemanticExprKind::Index { .. } => {
                let lvalue = self.resolve_lvalue(expr)?;
                self.load_lvalue(&lvalue)
            }
            SemanticExprKind::Postfix { op, expr: inner } => {
                let lvalue = self.resolve_lvalue(inner)?;
                let current = self.load_lvalue(&lvalue)?;
                let updated = self.emit_binary(
                    if matches!(op, PostfixOp::PostInc) {
                        BinaryOp::Add
                    } else {
                        BinaryOp::Sub
                    },
                    current.clone(),
                    one_for_type(&expr.ty),
                    expr.ty.clone(),
                );
                self.store_lvalue(&lvalue, updated);
                Ok(current)
            }
        }
    }

    // 生成二元运算指令
    fn emit_binary(
        &mut self,
        op: BinaryOp,
        left: IrOperand,
        right: IrOperand,
        ty: SemanticType,
    ) -> IrOperand {
        let dest = self.new_temp();
        self.instructions.push(IrInstr::Binary {
            dest,
            op,
            left,
            right,
            ty,
        });
        IrOperand::Temp(dest)
    }

    // 解析左值信息
    fn resolve_lvalue(&mut self, expr: &SemanticExpr) -> Result<LValue, String> {
        match &expr.kind {
            SemanticExprKind::Ident { name, symbol_id } => Ok(LValue::Var {
                symbol_id: symbol_id.ok_or_else(|| format!("missing symbol for '{}'", name))?,
                name: name.clone(),
            }),
            SemanticExprKind::Index { .. } => self.resolve_index_lvalue(expr),
            _ => Err("expression is not assignable".to_string()),
        }
    }

    // 解析数组左值
    fn resolve_index_lvalue(&mut self, expr: &SemanticExpr) -> Result<LValue, String> {
        let mut indices = Vec::new();
        let mut current = expr;

        loop {
            match &current.kind {
                SemanticExprKind::Index { array, index } => {
                    indices.push(self.lower_expr(index)?);
                    current = array;
                }
                SemanticExprKind::Ident { name, symbol_id } => {
                    indices.reverse();
                    let element_type = match expr.ty {
                        SemanticType::Int => TypeName::Int,
                        SemanticType::Float => TypeName::Float,
                        SemanticType::Array { element, .. } => element,
                        _ => return Err("unsupported indexed element type".to_string()),
                    };
                    return Ok(LValue::Index {
                        symbol_id: symbol_id.ok_or_else(|| format!("missing symbol for '{}'", name))?,
                        name: name.clone(),
                        indices,
                        element_type,
                    });
                }
                _ => return Err("unsupported array base expression".to_string()),
            }
        }
    }

    // 加载左值内容
    fn load_lvalue(&mut self, lvalue: &LValue) -> Result<IrOperand, String> {
        match lvalue {
            LValue::Var { symbol_id, name } => {
                let symbol = &self.symbols[symbol_id.0];
                if symbol.is_const {
                    if let Some(value) = self.const_values.get(symbol_id) {
                        return const_to_operand(value);
                    }
                }
                let dest = self.new_temp();
                self.instructions.push(IrInstr::LoadVar {
                    dest,
                    symbol_id: *symbol_id,
                    name: name.clone(),
                    ty: symbol.ty.clone(),
                });
                Ok(IrOperand::Temp(dest))
            }
            LValue::Index {
                symbol_id,
                name,
                indices,
                element_type,
            } => {
                let dest = self.new_temp();
                self.instructions.push(IrInstr::LoadIndex {
                    dest,
                    symbol_id: *symbol_id,
                    name: name.clone(),
                    indices: indices.clone(),
                    element_type: *element_type,
                });
                Ok(IrOperand::Temp(dest))
            }
        }
    }

    // 写回左值内容
    fn store_lvalue(&mut self, lvalue: &LValue, value: IrOperand) {
        match lvalue {
            LValue::Var { symbol_id, name } => self.instructions.push(IrInstr::StoreVar {
                symbol_id: *symbol_id,
                name: name.clone(),
                value,
            }),
            LValue::Index {
                symbol_id,
                name,
                indices,
                ..
            } => self.instructions.push(IrInstr::StoreIndex {
                symbol_id: *symbol_id,
                name: name.clone(),
                indices: indices.clone(),
                value,
            }),
        }
    }

    // 创建新的临时变量
    fn new_temp(&mut self) -> TempId {
        let temp = TempId(self.temp_counter);
        self.temp_counter += 1;
        temp
    }

    // 创建新的标签
    fn new_label(&mut self, prefix: &str) -> String {
        let label = format!("{}.{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    // 判断当前块是否已终结
    fn is_terminated(&self) -> bool {
        matches!(
            self.instructions.last(),
            Some(IrInstr::Jump { .. } | IrInstr::Branch { .. } | IrInstr::Return(_))
        )
    }

    // 在函数级别计算常量表达式
    fn eval_const_expr(&self, expr: &SemanticExpr) -> Result<IrConstValue, String> {
        match &expr.kind {
            SemanticExprKind::IntLiteral(value) => value
                .parse::<i32>()
                .map(IrConstValue::Int)
                .map_err(|_| format!("invalid int literal '{}'", value)),
            SemanticExprKind::FloatLiteral(value) => value
                .parse::<f32>()
                .map(IrConstValue::Float)
                .map_err(|_| format!("invalid float literal '{}'", value)),
            SemanticExprKind::Ident { symbol_id, .. } => {
                let symbol_id = symbol_id.ok_or_else(|| "missing symbol in const expr".to_string())?;
                self.const_values
                    .get(&symbol_id)
                    .cloned()
                    .ok_or_else(|| "identifier is not a known constant".to_string())
            }
            SemanticExprKind::Unary { op, expr } => {
                let value = self.eval_const_expr(expr)?;
                fold_unary_const(*op, value)
            }
            SemanticExprKind::Binary { op, left, right } => {
                let left = self.eval_const_expr(left)?;
                let right = self.eval_const_expr(right)?;
                fold_binary_const(*op, left, right)
            }
            SemanticExprKind::Index { array, index } => {
                let array = self.eval_const_expr(array)?;
                let index = match self.eval_const_expr(index)? {
                    IrConstValue::Int(value) if value >= 0 => value as usize,
                    _ => return Err("constant index must be a non-negative integer".to_string()),
                };
                match array {
                    IrConstValue::List(values) => values
                        .get(index)
                        .cloned()
                        .ok_or_else(|| "constant array index out of bounds".to_string()),
                    _ => Err("indexed constant is not an array".to_string()),
                }
            }
            _ => Err("unsupported constant expression".to_string()),
        }
    }

    // 在函数级别降低常量初始化器
    fn lower_const_initializer(&self, init: &SemanticInitializer) -> Result<IrConstValue, String> {
        match init {
            SemanticInitializer::Expr(expr) => self.eval_const_expr(expr),
            SemanticInitializer::List(items) => Ok(IrConstValue::List(
                items.iter()
                    .map(|item| self.lower_const_initializer(item))
                    .collect::<Result<Vec<_>, _>>()?,
            )),
        }
    }
}

// 将常量值转换为操作数
fn const_to_operand(value: &IrConstValue) -> Result<IrOperand, String> {
    match value {
        IrConstValue::Int(value) => Ok(IrOperand::Int(*value)),
        IrConstValue::Float(value) => Ok(IrOperand::Float(*value)),
        IrConstValue::List(_) => Err("array constant cannot be used as a scalar operand".to_string()),
    }
}

// 将常量转为无符号尺寸
fn as_usize_const(value: IrConstValue) -> Result<usize, String> {
    match value {
        IrConstValue::Int(value) if value >= 0 => Ok(value as usize),
        _ => Err("expected non-negative integer constant".to_string()),
    }
}

// 获取类型对应的单位值
fn one_for_type(ty: &SemanticType) -> IrOperand {
    match ty {
        SemanticType::Float => IrOperand::Float(1.0),
        _ => IrOperand::Int(1),
    }
}

// 折叠一元常量表达式
fn fold_unary_const(op: UnaryOp, value: IrConstValue) -> Result<IrConstValue, String> {
    match (op, value) {
        (UnaryOp::Plus, value) => Ok(value),
        (UnaryOp::Minus, IrConstValue::Int(value)) => Ok(IrConstValue::Int(-value)),
        (UnaryOp::Minus, IrConstValue::Float(value)) => Ok(IrConstValue::Float(-value)),
        (UnaryOp::Not, IrConstValue::Int(value)) => Ok(IrConstValue::Int((value == 0) as i32)),
        (UnaryOp::Not, IrConstValue::Float(value)) => Ok(IrConstValue::Int((value == 0.0) as i32)),
        _ => Err("unsupported unary constant expression".to_string()),
    }
}

// 折叠二元常量表达式
fn fold_binary_const(
    op: BinaryOp,
    left: IrConstValue,
    right: IrConstValue,
) -> Result<IrConstValue, String> {
    match (left, right) {
        (IrConstValue::Int(left), IrConstValue::Int(right)) => fold_int_binary(op, left, right),
        (IrConstValue::Float(left), IrConstValue::Float(right)) => fold_float_binary(op, left, right),
        (IrConstValue::Int(left), IrConstValue::Float(right)) => {
            fold_float_binary(op, left as f32, right)
        }
        (IrConstValue::Float(left), IrConstValue::Int(right)) => {
            fold_float_binary(op, left, right as f32)
        }
        _ => Err("unsupported binary constant expression".to_string()),
    }
}

// 计算整型二元常量
fn fold_int_binary(op: BinaryOp, left: i32, right: i32) -> Result<IrConstValue, String> {
    let value = match op {
        BinaryOp::Add => IrConstValue::Int(left + right),
        BinaryOp::Sub => IrConstValue::Int(left - right),
        BinaryOp::Mul => IrConstValue::Int(left * right),
        BinaryOp::Div => IrConstValue::Int(left / right),
        BinaryOp::Mod => IrConstValue::Int(left % right),
        BinaryOp::Eq => IrConstValue::Int((left == right) as i32),
        BinaryOp::Neq => IrConstValue::Int((left != right) as i32),
        BinaryOp::Lt => IrConstValue::Int((left < right) as i32),
        BinaryOp::Le => IrConstValue::Int((left <= right) as i32),
        BinaryOp::Gt => IrConstValue::Int((left > right) as i32),
        BinaryOp::Ge => IrConstValue::Int((left >= right) as i32),
        BinaryOp::And => IrConstValue::Int(((left != 0) && (right != 0)) as i32),
        BinaryOp::Or => IrConstValue::Int(((left != 0) || (right != 0)) as i32),
    };
    Ok(value)
}

// 计算浮点二元常量
fn fold_float_binary(op: BinaryOp, left: f32, right: f32) -> Result<IrConstValue, String> {
    let value = match op {
        BinaryOp::Add => IrConstValue::Float(left + right),
        BinaryOp::Sub => IrConstValue::Float(left - right),
        BinaryOp::Mul => IrConstValue::Float(left * right),
        BinaryOp::Div => IrConstValue::Float(left / right),
        BinaryOp::Eq => IrConstValue::Int((left == right) as i32),
        BinaryOp::Neq => IrConstValue::Int((left != right) as i32),
        BinaryOp::Lt => IrConstValue::Int((left < right) as i32),
        BinaryOp::Le => IrConstValue::Int((left <= right) as i32),
        BinaryOp::Gt => IrConstValue::Int((left > right) as i32),
        BinaryOp::Ge => IrConstValue::Int((left >= right) as i32),
        BinaryOp::And => IrConstValue::Int(((left != 0.0) && (right != 0.0)) as i32),
        BinaryOp::Or => IrConstValue::Int(((left != 0.0) || (right != 0.0)) as i32),
        BinaryOp::Mod => return Err("float modulo is unsupported".to_string()),
    };
    Ok(value)
}

impl fmt::Display for IrProgram {
    // 以字符串形式输出 IR
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format_program(self))
    }
}