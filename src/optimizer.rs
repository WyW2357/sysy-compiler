use std::collections::{HashMap, HashSet};

use crate::ast::{BinaryOp, UnaryOp};
use crate::ir::{IrFunction, IrInstr, IrOperand, IrProgram, TempId};

#[derive(Debug, Default, Clone)]
pub struct OptimizationReport {
    pub constant_folds: usize,
    pub cse_eliminations: usize,
    pub dead_code_eliminations: usize,
    pub instructions_before: usize,
    pub instructions_after: usize,
}

// 优化整个中间表示程序
pub fn optimize_program(program: &IrProgram) -> (IrProgram, OptimizationReport) {
    let mut report = OptimizationReport {
        instructions_before: count_instructions(program),
        ..OptimizationReport::default()
    };

    let functions = program
        .functions
        .iter()
        .map(|function| optimize_function(function, &mut report))
        .collect::<Vec<_>>();

    let optimized = IrProgram {
        globals: program.globals.clone(),
        functions,
    };
    report.instructions_after = count_instructions(&optimized);

    (optimized, report)
}

// 优化单个函数
fn optimize_function(function: &IrFunction, report: &mut OptimizationReport) -> IrFunction {
    let mut instructions = Vec::new();
    let mut block = Vec::new();

    for instruction in &function.instructions {
        match instruction {
            IrInstr::Label(_) => {
                if !block.is_empty() {
                    instructions.extend(optimize_block(&block, report));
                    block.clear();
                }
                instructions.push(instruction.clone());
            }
            IrInstr::Jump { .. } | IrInstr::Branch { .. } | IrInstr::Return(_) => {
                block.push(instruction.clone());
                instructions.extend(optimize_block(&block, report));
                block.clear();
            }
            _ => block.push(instruction.clone()),
        }
    }

    if !block.is_empty() {
        instructions.extend(optimize_block(&block, report));
    }

    IrFunction {
        name: function.name.clone(),
        return_type: function.return_type,
        params: function.params.clone(),
        instructions,
    }
}

// 优化一个基本块
fn optimize_block(block: &[IrInstr], report: &mut OptimizationReport) -> Vec<IrInstr> {
    let mut replacements = HashMap::<TempId, IrOperand>::new();
    let mut expr_cache = HashMap::<String, TempId>::new();
    let mut rewritten = Vec::new();

    for instruction in block {
        let instruction = rewrite_instruction(instruction, &replacements);

        if let Some(jump) = fold_constant_branch(&instruction) {
            report.constant_folds += 1;
            rewritten.push(jump);
            continue;
        }

        if let Some((dest, constant)) = fold_instruction(&instruction) {
            replacements.insert(dest, constant);
            report.constant_folds += 1;
            continue;
        }

        if let Some(dest) = instruction_dest(&instruction) {
            if is_pure_instruction(&instruction) {
                let key = cse_key(&instruction);
                if let Some(existing) = expr_cache.get(&key) {
                    replacements.insert(dest, IrOperand::Temp(*existing));
                    report.cse_eliminations += 1;
                    continue;
                }
                expr_cache.insert(key, dest);
            }
        }

        if has_side_effects(&instruction) {
            expr_cache.clear();
        }

        rewritten.push(instruction);
    }

    eliminate_dead_code(&rewritten, report)
}

// 重写指令中的操作数
fn rewrite_instruction(instruction: &IrInstr, replacements: &HashMap<TempId, IrOperand>) -> IrInstr {
    match instruction {
        IrInstr::StoreVar {
            symbol_id,
            name,
            value,
        } => IrInstr::StoreVar {
            symbol_id: *symbol_id,
            name: name.clone(),
            value: rewrite_operand(value, replacements),
        },
        IrInstr::StoreIndex {
            symbol_id,
            name,
            indices,
            value,
        } => IrInstr::StoreIndex {
            symbol_id: *symbol_id,
            name: name.clone(),
            indices: indices
                .iter()
                .map(|operand| rewrite_operand(operand, replacements))
                .collect(),
            value: rewrite_operand(value, replacements),
        },
        IrInstr::LoadIndex {
            dest,
            symbol_id,
            name,
            indices,
            element_type,
        } => IrInstr::LoadIndex {
            dest: *dest,
            symbol_id: *symbol_id,
            name: name.clone(),
            indices: indices
                .iter()
                .map(|operand| rewrite_operand(operand, replacements))
                .collect(),
            element_type: *element_type,
        },
        IrInstr::Unary {
            dest,
            op,
            operand,
            ty,
        } => IrInstr::Unary {
            dest: *dest,
            op: *op,
            operand: rewrite_operand(operand, replacements),
            ty: ty.clone(),
        },
        IrInstr::Binary {
            dest,
            op,
            left,
            right,
            ty,
        } => IrInstr::Binary {
            dest: *dest,
            op: *op,
            left: rewrite_operand(left, replacements),
            right: rewrite_operand(right, replacements),
            ty: ty.clone(),
        },
        IrInstr::Call {
            dest,
            function,
            args,
            return_type,
        } => IrInstr::Call {
            dest: *dest,
            function: function.clone(),
            args: args
                .iter()
                .map(|operand| rewrite_operand(operand, replacements))
                .collect(),
            return_type: return_type.clone(),
        },
        IrInstr::Branch {
            condition,
            then_label,
            else_label,
        } => IrInstr::Branch {
            condition: rewrite_operand(condition, replacements),
            then_label: then_label.clone(),
            else_label: else_label.clone(),
        },
        IrInstr::Return(value) => {
            IrInstr::Return(value.as_ref().map(|operand| rewrite_operand(operand, replacements)))
        }
        other => other.clone(),
    }
}

// 替换操作数引用
fn rewrite_operand(operand: &IrOperand, replacements: &HashMap<TempId, IrOperand>) -> IrOperand {
    match operand {
        IrOperand::Temp(temp) => match replacements.get(temp) {
            Some(replacement) => rewrite_operand(replacement, replacements),
            None => IrOperand::Temp(*temp),
        },
        other => other.clone(),
    }
}

// 尝试进行常量折叠
fn fold_instruction(instruction: &IrInstr) -> Option<(TempId, IrOperand)> {
    match instruction {
        IrInstr::Unary { dest, op, operand, .. } => match (op, operand) {
            (UnaryOp::Plus, IrOperand::Int(value)) => Some((*dest, IrOperand::Int(*value))),
            (UnaryOp::Plus, IrOperand::Float(value)) => Some((*dest, IrOperand::Float(*value))),
            (UnaryOp::Minus, IrOperand::Int(value)) => Some((*dest, IrOperand::Int(-*value))),
            (UnaryOp::Minus, IrOperand::Float(value)) => Some((*dest, IrOperand::Float(-*value))),
            (UnaryOp::Not, IrOperand::Int(value)) => Some((*dest, IrOperand::Int((*value == 0) as i32))),
            (UnaryOp::Not, IrOperand::Float(value)) => {
                Some((*dest, IrOperand::Int((*value == 0.0) as i32)))
            }
            _ => None,
        },
        IrInstr::Binary {
            dest,
            op,
            left,
            right,
            ..
        } => fold_binary(*dest, *op, left, right),
        _ => None,
    }
}

// 折叠二元常量表达式
fn fold_binary(dest: TempId, op: BinaryOp, left: &IrOperand, right: &IrOperand) -> Option<(TempId, IrOperand)> {
    match (left, right) {
        (IrOperand::Int(left), IrOperand::Int(right)) => {
            let value = match op {
                BinaryOp::Add => IrOperand::Int(left.checked_add(*right)?),
                BinaryOp::Sub => IrOperand::Int(left.checked_sub(*right)?),
                BinaryOp::Mul => IrOperand::Int(left.checked_mul(*right)?),
                BinaryOp::Div => IrOperand::Int(left.checked_div(*right)?),
                BinaryOp::Mod => IrOperand::Int(left.checked_rem(*right)?),
                BinaryOp::Eq => IrOperand::Int((left == right) as i32),
                BinaryOp::Neq => IrOperand::Int((left != right) as i32),
                BinaryOp::Lt => IrOperand::Int((left < right) as i32),
                BinaryOp::Le => IrOperand::Int((left <= right) as i32),
                BinaryOp::Gt => IrOperand::Int((left > right) as i32),
                BinaryOp::Ge => IrOperand::Int((left >= right) as i32),
                BinaryOp::And => IrOperand::Int(((*left != 0) && (*right != 0)) as i32),
                BinaryOp::Or => IrOperand::Int(((*left != 0) || (*right != 0)) as i32),
            };
            Some((dest, value))
        }
        (IrOperand::Float(left), IrOperand::Float(right)) => {
            let value = match op {
                BinaryOp::Add => IrOperand::Float(*left + *right),
                BinaryOp::Sub => IrOperand::Float(*left - *right),
                BinaryOp::Mul => IrOperand::Float(*left * *right),
                BinaryOp::Div => IrOperand::Float(*left / *right),
                BinaryOp::Eq => IrOperand::Int((left == right) as i32),
                BinaryOp::Neq => IrOperand::Int((left != right) as i32),
                BinaryOp::Lt => IrOperand::Int((left < right) as i32),
                BinaryOp::Le => IrOperand::Int((left <= right) as i32),
                BinaryOp::Gt => IrOperand::Int((left > right) as i32),
                BinaryOp::Ge => IrOperand::Int((left >= right) as i32),
                BinaryOp::And => IrOperand::Int(((*left != 0.0) && (*right != 0.0)) as i32),
                BinaryOp::Or => IrOperand::Int(((*left != 0.0) || (*right != 0.0)) as i32),
                BinaryOp::Mod => return None,
            };
            Some((dest, value))
        }
        _ => None,
    }
}

// 生成公共子表达式键值
fn cse_key(instruction: &IrInstr) -> String {
    match instruction {
        IrInstr::LoadVar { symbol_id, .. } => format!("load:{}", symbol_id.0),
        IrInstr::LoadIndex {
            symbol_id,
            indices,
            ..
        } => format!(
            "load_index:{}:{}",
            symbol_id.0,
            indices
                .iter()
                .map(format_operand_key)
                .collect::<Vec<_>>()
                .join(",")
        ),
        IrInstr::Unary { op, operand, .. } => {
            format!("unary:{:?}:{}", op, format_operand_key(operand))
        }
        IrInstr::Binary {
            op,
            left,
            right,
            ..
        } => format!(
            "binary:{:?}:{}:{}",
            op,
            format_operand_key(left),
            format_operand_key(right)
        ),
        _ => String::new(),
    }
}

// 格式化操作数键名
fn format_operand_key(operand: &IrOperand) -> String {
    match operand {
        IrOperand::Temp(temp) => format!("t{}", temp.0),
        IrOperand::Int(value) => format!("i{}", value),
        IrOperand::Float(value) => format!("f{:e}", value),
        IrOperand::Symbol { symbol_id, .. } => format!("s{}", symbol_id.0),
    }
}

// 获取指令的目标临时变量
fn instruction_dest(instruction: &IrInstr) -> Option<TempId> {
    match instruction {
        IrInstr::LoadVar { dest, .. }
        | IrInstr::LoadIndex { dest, .. }
        | IrInstr::Unary { dest, .. }
        | IrInstr::Binary { dest, .. } => Some(*dest),
        IrInstr::Call { dest, .. } => *dest,
        _ => None,
    }
}

// 判断指令是否无副作用
fn is_pure_instruction(instruction: &IrInstr) -> bool {
    matches!(
        instruction,
        IrInstr::LoadVar { .. }
            | IrInstr::LoadIndex { .. }
            | IrInstr::Unary { .. }
            | IrInstr::Binary { .. }
    )
}

// 判断指令是否有副作用
fn has_side_effects(instruction: &IrInstr) -> bool {
    matches!(
        instruction,
        IrInstr::StoreVar { .. }
            | IrInstr::StoreIndex { .. }
            | IrInstr::Call { .. }
            | IrInstr::Jump { .. }
            | IrInstr::Branch { .. }
            | IrInstr::Return(_)
    )
}

// 删除无用代码
fn eliminate_dead_code(block: &[IrInstr], report: &mut OptimizationReport) -> Vec<IrInstr> {
    let mut live_temps = HashSet::<TempId>::new();
    let mut keep = vec![true; block.len()];

    for index in (0..block.len()).rev() {
        let instruction = &block[index];
        if let Some(dest) = instruction_dest(instruction) {
            if is_pure_instruction(instruction) && !live_temps.contains(&dest) {
                keep[index] = false;
                report.dead_code_eliminations += 1;
                continue;
            }
            live_temps.remove(&dest);
        }

        for used in used_temps(instruction) {
            live_temps.insert(used);
        }
    }

    block
        .iter()
        .zip(keep)
        .filter_map(|(instruction, keep)| keep.then_some(instruction.clone()))
        .collect()
}

// 收集指令使用的临时变量
fn used_temps(instruction: &IrInstr) -> Vec<TempId> {
    match instruction {
        IrInstr::StoreVar { value, .. } => operand_temps(value),
        IrInstr::LoadIndex { indices, .. } => indices.iter().flat_map(operand_temps).collect(),
        IrInstr::StoreIndex { indices, value, .. } => {
            let mut temps = indices.iter().flat_map(operand_temps).collect::<Vec<_>>();
            temps.extend(operand_temps(value));
            temps
        }
        IrInstr::Unary { operand, .. } => operand_temps(operand),
        IrInstr::Binary { left, right, .. } => {
            let mut temps = operand_temps(left);
            temps.extend(operand_temps(right));
            temps
        }
        IrInstr::Call { args, .. } => args.iter().flat_map(operand_temps).collect(),
        IrInstr::Branch { condition, .. } => operand_temps(condition),
        IrInstr::Return(Some(value)) => operand_temps(value),
        _ => Vec::new(),
    }
}

// 提取操作数中的临时变量
fn operand_temps(operand: &IrOperand) -> Vec<TempId> {
    match operand {
        IrOperand::Temp(temp) => vec![*temp],
        _ => Vec::new(),
    }
}

// 统计程序中的指令数量
fn count_instructions(program: &IrProgram) -> usize {
    program
        .functions
        .iter()
        .map(|function| function.instructions.len())
        .sum()
}

// 折叠常量分支
fn fold_constant_branch(instruction: &IrInstr) -> Option<IrInstr> {
    match instruction {
        IrInstr::Branch {
            condition,
            then_label,
            else_label,
        } => {
            let take_then = match condition {
                IrOperand::Int(value) => *value != 0,
                IrOperand::Float(value) => *value != 0.0,
                _ => return None,
            };
            Some(IrInstr::Jump {
                label: if take_then {
                    then_label.clone()
                } else {
                    else_label.clone()
                },
            })
        }
        _ => None,
    }
}