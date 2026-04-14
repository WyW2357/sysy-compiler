mod ast;
mod ir;
mod lexer;
#[cfg(feature = "llvm-backend")]
mod llvm_backend;
mod optimizer;
mod parser;
mod semantic;

use crate::ast::Program;
use crate::ir::{format_program, lower_program, IrProgram};
use crate::lexer::{tokenize, Token, TokenKind};
#[cfg(feature = "llvm-backend")]
use crate::llvm_backend::{generate_backend_artifacts, BackendArtifacts};
use crate::optimizer::{optimize_program, OptimizationReport};
use crate::parser::{ParseError, Parser};
use crate::semantic::{analyze, SemanticError, SemanticProgram};
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};

fn main() {
    let src = fs::read_to_string("sysy.c").expect("failed to read sysy.c");

    let tokens = tokenize(&src);
    write_tokens(&tokens);

    let mut parser = Parser::new(tokens);
    let program = match parser.parse_program() {
        Ok(program) => {
            write_parse_errors(&[]);
            program
        }
        Err(error) => {
            write_parse_errors(&[error]);
            return;
        }
    };
    write_ast(&program);

    let semantic = analyze(&program);
    write_semantic_ast(&semantic.program);
    write_semantic_errors(&semantic.errors);

    if semantic.errors.is_empty() {
        let ir = lower_program(&semantic.program).expect("failed to lower semantic AST to IR");
        write_ir(&ir);

        let (optimized_ir, report) = optimize_program(&ir);
        write_optimized_ir(&optimized_ir);
        write_optimization_report(&report);
        run_llvm_backend(&optimized_ir);
    }
}

#[cfg(feature = "llvm-backend")]
fn run_llvm_backend(program: &IrProgram) {
    match generate_backend_artifacts(program) {
        Ok(artifacts) => {
            write_llvm_ir(&artifacts.llvm_ir);
            write_codegen_status(Some(&artifacts));
        }
        Err(error) => write_codegen_failure(&error),
    }
}

#[cfg(not(feature = "llvm-backend"))]
fn run_llvm_backend(_program: &IrProgram) {
    write_codegen_status(None);
}

fn write_tokens(tokens: &[Token]) {
    let out_file = File::create("tokens.txt").expect("failed to create tokens.txt");
    let mut out = BufWriter::new(out_file);

    writeln!(out, "{:<15} {:<30} {:<6} {:<6}", "Kind", "Lexeme", "Line", "Col").expect("failed to write header");
    writeln!(out, "{:-<15} {:-<30} {:-<6} {:-<6}", "", "", "", "").expect("failed to write separator");

    for token in tokens {
        writeln!(out, "{:<15} {:<30} {:<6} {:<6}", format!("{:?}", token.kind), token.lexeme, token.line, token.column).expect("failed to write token");
        if token.kind == TokenKind::Eof {
            break;
        }
    }
}

fn write_ast(program: &Program) {
    let out_file = File::create("ast.txt").expect("failed to create ast.txt");
    let mut out = BufWriter::new(out_file);
    writeln!(out, "{:#?}", program).expect("failed to write ast");
}

fn write_parse_errors(errors: &[ParseError]) {
    let out_file = File::create("parse_errors.txt").expect("failed to create parse_errors.txt");
    let mut out = BufWriter::new(out_file);

    if errors.is_empty() {
        writeln!(out, "No parse errors.").expect("failed to write parse errors");
        return;
    }

    for error in errors {
        writeln!(
            out,
            "line {}, col {}: {}",
            error.location.line,
            error.location.column,
            error.message
        )
        .expect("failed to write parse error");
    }
}

fn write_semantic_ast(program: &SemanticProgram) {
    let out_file = File::create("semantic_ast.txt").expect("failed to create semantic_ast.txt");
    let mut out = BufWriter::new(out_file);
    writeln!(out, "{:#?}", program).expect("failed to write semantic ast");
}

fn write_semantic_errors(errors: &[SemanticError]) {
    let out_file = File::create("semantic_errors.txt").expect("failed to create semantic_errors.txt");
    let mut out = BufWriter::new(out_file);

    if errors.is_empty() {
        writeln!(out, "No semantic errors.").expect("failed to write semantic errors");
        return;
    }

    for error in errors {
        writeln!(
            out,
            "line {}, col {}: {}",
            error.location.line,
            error.location.column,
            error.message
        )
            .expect("failed to write semantic error");
    }
}

#[cfg(feature = "llvm-backend")]
fn write_codegen_status(artifacts: Option<&BackendArtifacts>) {
    let out_file = File::create("codegen_status.txt").expect("failed to create codegen_status.txt");
    let mut out = BufWriter::new(out_file);

    writeln!(out, "Selected backend: LLVM/inkwell").expect("failed to write backend header");
    writeln!(out, "Current repository now lowers semantic AST into a structured internal IR and optimizes it before LLVM lowering.")
        .expect("failed to write pipeline status");
    writeln!(out, "Generated files: ir.txt, optimized_ir.txt, optimization_report.txt.")
        .expect("failed to write generated files status");
    if let Some(artifacts) = artifacts {
        writeln!(out, "LLVM backend succeeded.").expect("failed to write backend success");
        writeln!(out, "target_triple: {}", artifacts.triple).expect("failed to write triple");
        writeln!(out, "generated_llvm_ir: output.ll").expect("failed to write llvm ir path");
        writeln!(out, "generated_assembly: {}", artifacts.assembly_path).expect("failed to write assembly path");
    } else {
        writeln!(out, "LLVM backend was not executed.").expect("failed to write backend pending");
    }
}

#[cfg(not(feature = "llvm-backend"))]
fn write_codegen_status(_artifacts: Option<&()>) {
    let out_file = File::create("codegen_status.txt").expect("failed to create codegen_status.txt");
    let mut out = BufWriter::new(out_file);

    writeln!(out, "Selected backend: LLVM/inkwell").expect("failed to write backend header");
    writeln!(out, "Current repository now lowers semantic AST into a structured internal IR and optimizes it before LLVM lowering.")
        .expect("failed to write pipeline status");
    writeln!(out, "Generated files: ir.txt, optimized_ir.txt, optimization_report.txt.")
        .expect("failed to write generated files status");
    writeln!(out, "LLVM backend code is present but disabled in the default build.")
        .expect("failed to write backend disabled status");
    writeln!(out, "To enable it later: cargo run --features llvm-backend").expect("failed to write enable command");
    writeln!(out, "Current local LLVM install is insufficient for llvm-sys: missing llvm-config and full LLVM libraries.")
        .expect("failed to write environment note");
}

#[cfg(feature = "llvm-backend")]
fn write_codegen_failure(error: &str) {
    let out_file = File::create("codegen_status.txt").expect("failed to create codegen_status.txt");
    let mut out = BufWriter::new(out_file);

    writeln!(out, "Selected backend: LLVM/inkwell").expect("failed to write backend header");
    writeln!(out, "Current repository now lowers semantic AST into a structured internal IR and optimizes it before LLVM lowering.")
        .expect("failed to write pipeline status");
    writeln!(out, "LLVM backend failed: {}", error).expect("failed to write failure reason");
    writeln!(out, "Check LLVM_SYS_181_PREFIX and the installed LLVM libraries.")
        .expect("failed to write troubleshooting hint");
}

fn write_ir(program: &IrProgram) {
    let out_file = File::create("ir.txt").expect("failed to create ir.txt");
    let mut out = BufWriter::new(out_file);
    writeln!(out, "{}", format_program(program)).expect("failed to write ir");
}

fn write_optimized_ir(program: &IrProgram) {
    let out_file = File::create("optimized_ir.txt").expect("failed to create optimized_ir.txt");
    let mut out = BufWriter::new(out_file);
    writeln!(out, "{}", format_program(program)).expect("failed to write optimized ir");
}

#[cfg(feature = "llvm-backend")]
fn write_llvm_ir(ir: &str) {
    let out_file = File::create("output.ll").expect("failed to create output.ll");
    let mut out = BufWriter::new(out_file);
    writeln!(out, "{}", ir).expect("failed to write llvm ir");
}

fn write_optimization_report(report: &OptimizationReport) {
    let out_file = File::create("optimization_report.txt").expect("failed to create optimization_report.txt");
    let mut out = BufWriter::new(out_file);

    writeln!(out, "IR optimization report").expect("failed to write report title");
    writeln!(out, "constant_folds: {}", report.constant_folds).expect("failed to write constant fold count");
    writeln!(out, "cse_eliminations: {}", report.cse_eliminations).expect("failed to write cse count");
    writeln!(out, "dead_code_eliminations: {}", report.dead_code_eliminations).expect("failed to write dce count");
    writeln!(out, "instructions_before: {}", report.instructions_before).expect("failed to write count before");
    writeln!(out, "instructions_after: {}", report.instructions_after).expect("failed to write count after");
    writeln!(out, "removed_instructions: {}", report.instructions_before.saturating_sub(report.instructions_after))
        .expect("failed to write removed instruction count");
}