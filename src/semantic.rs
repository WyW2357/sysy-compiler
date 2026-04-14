#![allow(dead_code)]

use std::collections::HashMap;

use crate::ast::{
    BinaryOp, Block, Expr, ExprKind, ForInit, FunctionDef, Initializer, Item, Param, PostfixOp,
    Program, SourceLocation, Stmt, TypeName, UnaryOp, VarDecl,
};

#[derive(Debug, Clone)]
pub struct SemanticAnalysis {
    pub program: SemanticProgram,
    pub errors: Vec<SemanticError>,
}

#[derive(Debug, Clone)]
pub struct SemanticProgram {
    pub items: Vec<SemanticItem>,
    pub symbols: Vec<Symbol>,
    pub scopes: Vec<ScopeInfo>,
}

#[derive(Debug, Clone)]
pub enum SemanticItem {
    Function(SemanticFunction),
    GlobalDecl(Vec<SemanticVarDecl>),
}

#[derive(Debug, Clone)]
pub struct SemanticFunction {
    pub symbol_id: Option<SymbolId>,
    pub return_type: TypeName,
    pub name: String,
    pub params: Vec<SemanticParam>,
    pub scope_id: ScopeId,
    pub body: SemanticBlock,
}

#[derive(Debug, Clone)]
pub struct SemanticParam {
    pub symbol_id: Option<SymbolId>,
    pub ty: SemanticType,
    pub name: String,
    pub dimensions: Vec<Option<SemanticExpr>>,
}

#[derive(Debug, Clone)]
pub struct SemanticBlock {
    pub scope_id: ScopeId,
    pub statements: Vec<SemanticStmt>,
}

#[derive(Debug, Clone)]
pub enum SemanticStmt {
    VarDecl(Vec<SemanticVarDecl>),
    If {
        condition: SemanticExpr,
        then_branch: Box<SemanticStmt>,
        else_branch: Option<Box<SemanticStmt>>,
    },
    While {
        condition: SemanticExpr,
        body: Box<SemanticStmt>,
    },
    For {
        scope_id: Option<ScopeId>,
        init: Option<SemanticForInit>,
        condition: Option<SemanticExpr>,
        update: Option<SemanticExpr>,
        body: Box<SemanticStmt>,
    },
    Return(Option<SemanticExpr>),
    Break,
    Continue,
    Expr(SemanticExpr),
    Block(SemanticBlock),
    Empty,
}

#[derive(Debug, Clone)]
pub enum SemanticForInit {
    Decl(Vec<SemanticVarDecl>),
    Expr(SemanticExpr),
}

#[derive(Debug, Clone)]
pub struct SemanticVarDecl {
    pub symbol_id: Option<SymbolId>,
    pub is_const: bool,
    pub ty: SemanticType,
    pub name: String,
    pub dimensions: Vec<Option<SemanticExpr>>,
    pub init: Option<SemanticInitializer>,
}

#[derive(Debug, Clone)]
pub enum SemanticInitializer {
    Expr(SemanticExpr),
    List(Vec<SemanticInitializer>),
}

#[derive(Debug, Clone)]
pub struct SemanticExpr {
    pub location: SourceLocation,
    pub ty: SemanticType,
    pub kind: SemanticExprKind,
}

#[derive(Debug, Clone)]
pub enum SemanticExprKind {
    Ident {
        name: String,
        symbol_id: Option<SymbolId>,
    },
    IntLiteral(String),
    FloatLiteral(String),
    Unary {
        op: UnaryOp,
        expr: Box<SemanticExpr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<SemanticExpr>,
        right: Box<SemanticExpr>,
    },
    Assign {
        target: Box<SemanticExpr>,
        value: Box<SemanticExpr>,
    },
    Call {
        callee: Box<SemanticExpr>,
        args: Vec<SemanticExpr>,
        function_symbol: Option<SymbolId>,
    },
    Index {
        array: Box<SemanticExpr>,
        index: Box<SemanticExpr>,
    },
    Postfix {
        op: PostfixOp,
        expr: Box<SemanticExpr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(pub usize);

#[derive(Debug, Clone)]
pub struct Symbol {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub is_const: bool,
    pub ty: SemanticType,
    pub scope_id: ScopeId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Variable,
    Parameter,
    Function,
}

#[derive(Debug, Clone)]
pub struct ScopeInfo {
    pub id: ScopeId,
    pub parent: Option<ScopeId>,
    pub symbols: HashMap<String, SymbolId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticType {
    Int,
    Float,
    Void,
    Array { element: TypeName, rank: usize },
    Function {
        return_type: TypeName,
        params: Vec<SemanticType>,
    },
    Error,
}

#[derive(Debug, Clone)]
pub struct SemanticError {
    pub location: SourceLocation,
    pub message: String,
}

// 执行语义分析
pub fn analyze(program: &Program) -> SemanticAnalysis {
    let mut analyzer = Analyzer::new();
    analyzer.register_builtins();
    let predeclared = analyzer.predeclare_program(program);
    let items = analyzer.analyze_program(program, &predeclared);

    SemanticAnalysis {
        program: SemanticProgram {
            items,
            symbols: analyzer.symbols,
            scopes: analyzer.scopes,
        },
        errors: analyzer.errors,
    }
}

#[derive(Debug, Clone)]
enum PredeclaredItem {
    Function(Option<SymbolId>),
    Global(Vec<Option<SymbolId>>),
}

struct Analyzer {
    symbols: Vec<Symbol>,
    scopes: Vec<ScopeInfo>,
    scope_stack: Vec<ScopeId>,
    errors: Vec<SemanticError>,
    current_return_type: Option<TypeName>,
    loop_depth: usize,
}

impl Analyzer {
    // 创建语义分析器
    fn new() -> Self {
        let global_scope = ScopeInfo {
            id: ScopeId(0),
            parent: None,
            symbols: HashMap::new(),
        };

        Self {
            symbols: Vec::new(),
            scopes: vec![global_scope],
            scope_stack: vec![ScopeId(0)],
            errors: Vec::new(),
            current_return_type: None,
            loop_depth: 0,
        }
    }

    // 注册内置函数
    fn register_builtins(&mut self) {
        let builtins = [
            ("getint", TypeName::Int, vec![]),
            ("getch", TypeName::Int, vec![]),
            ("getfloat", TypeName::Float, vec![]),
            (
                "getarray",
                TypeName::Int,
                vec![SemanticType::Array {
                    element: TypeName::Int,
                    rank: 1,
                }],
            ),
            (
                "getfarray",
                TypeName::Int,
                vec![SemanticType::Array {
                    element: TypeName::Float,
                    rank: 1,
                }],
            ),
            ("putint", TypeName::Void, vec![SemanticType::Int]),
            ("putch", TypeName::Void, vec![SemanticType::Int]),
            ("putfloat", TypeName::Void, vec![SemanticType::Float]),
            (
                "putarray",
                TypeName::Void,
                vec![
                    SemanticType::Int,
                    SemanticType::Array {
                        element: TypeName::Int,
                        rank: 1,
                    },
                ],
            ),
            (
                "putfarray",
                TypeName::Void,
                vec![
                    SemanticType::Int,
                    SemanticType::Array {
                        element: TypeName::Float,
                        rank: 1,
                    },
                ],
            ),
            ("starttime", TypeName::Void, vec![]),
            ("stoptime", TypeName::Void, vec![]),
        ];

        for (name, return_type, params) in builtins {
            let ty = SemanticType::Function {
                return_type,
                params,
            };
            let _ = self.declare_symbol(
                name.to_string(),
                SymbolKind::Function,
                false,
                ty,
                SourceLocation { line: 0, column: 0 },
            );
        }
    }

    // 预声明顶层符号
    fn predeclare_program(&mut self, program: &Program) -> Vec<PredeclaredItem> {
        let mut predeclared = Vec::with_capacity(program.items.len());

        for item in &program.items {
            match item {
                Item::Function(function) => {
                    let ty = SemanticType::Function {
                        return_type: function.return_type,
                        params: function
                            .params
                            .iter()
                            .map(|param| self.declared_type(param.ty, param.dimensions.len()))
                            .collect(),
                    };
                    let symbol_id = self.declare_symbol(
                        function.name.clone(),
                        SymbolKind::Function,
                        false,
                        ty,
                        function.location,
                    );
                    predeclared.push(PredeclaredItem::Function(symbol_id));
                }
                Item::GlobalDecl(decls) => {
                    let mut symbol_ids = Vec::with_capacity(decls.len());
                    for decl in decls {
                        let ty = self.declared_type(decl.ty, decl.dimensions.len());
                        if decl.ty == TypeName::Void {
                            self.report_error(
                                decl.location,
                                "variables cannot have type void".to_string(),
                            );
                        }
                        let symbol_id = self.declare_symbol(
                            decl.name.clone(),
                            SymbolKind::Variable,
                            decl.is_const,
                            ty,
                            decl.location,
                        );
                        symbol_ids.push(symbol_id);
                    }
                    predeclared.push(PredeclaredItem::Global(symbol_ids));
                }
            }
        }

        predeclared
    }

    // 分析整个程序
    fn analyze_program(
        &mut self,
        program: &Program,
        predeclared: &[PredeclaredItem],
    ) -> Vec<SemanticItem> {
        let mut items = Vec::with_capacity(program.items.len());

        for (item, predeclared_item) in program.items.iter().zip(predeclared.iter()) {
            match (item, predeclared_item) {
                (Item::Function(function), PredeclaredItem::Function(symbol_id)) => {
                    items.push(SemanticItem::Function(self.analyze_function(function, *symbol_id)));
                }
                (Item::GlobalDecl(decls), PredeclaredItem::Global(symbol_ids)) => {
                    items.push(SemanticItem::GlobalDecl(
                        decls
                            .iter()
                            .zip(symbol_ids.iter())
                            .map(|(decl, symbol_id)| self.analyze_global_decl(decl, *symbol_id))
                            .collect(),
                    ));
                }
                _ => unreachable!(),
            }
        }

        items
    }

    // 分析函数定义
    fn analyze_function(
        &mut self,
        function: &FunctionDef,
        symbol_id: Option<SymbolId>,
    ) -> SemanticFunction {
        let scope_id = self.push_scope();
        let previous_return_type = self.current_return_type;
        self.current_return_type = Some(function.return_type);

        let mut params = Vec::with_capacity(function.params.len());
        for param in &function.params {
            params.push(self.analyze_param(param));
        }

        let body = self.analyze_block_in_current_scope(&function.body);

        self.current_return_type = previous_return_type;
        self.pop_scope();

        SemanticFunction {
            symbol_id,
            return_type: function.return_type,
            name: function.name.clone(),
            params,
            scope_id,
            body,
        }
    }

    // 分析函数参数
    fn analyze_param(&mut self, param: &Param) -> SemanticParam {
        let dimensions = self.analyze_dimensions(&param.dimensions, param.location);
        let ty = self.declared_type(param.ty, param.dimensions.len());

        if param.ty == TypeName::Void {
            self.report_error(
                param.location,
                "parameters cannot have type void".to_string(),
            );
        }

        let symbol_id = self.declare_symbol(
            param.name.clone(),
            SymbolKind::Parameter,
            false,
            ty.clone(),
            param.location,
        );

        SemanticParam {
            symbol_id,
            ty,
            name: param.name.clone(),
            dimensions,
        }
    }

    // 分析全局变量声明
    fn analyze_global_decl(&mut self, decl: &VarDecl, symbol_id: Option<SymbolId>) -> SemanticVarDecl {
        let dimensions = self.analyze_dimensions(&decl.dimensions, decl.location);
        let ty = self.declared_type(decl.ty, decl.dimensions.len());
        if decl.ty == TypeName::Void {
            self.report_error(
                decl.location,
                "variables cannot have type void".to_string(),
            );
        }
        let init = decl
            .init
            .as_ref()
            .map(|initializer| self.analyze_initializer(initializer, &ty, decl.location));

        self.check_const_declaration(decl.is_const, init.as_ref(), decl.location);

        SemanticVarDecl {
            symbol_id,
            is_const: decl.is_const,
            ty,
            name: decl.name.clone(),
            dimensions,
            init,
        }
    }

    // 分析局部变量声明
    fn analyze_local_decl(&mut self, decl: &VarDecl) -> SemanticVarDecl {
        let dimensions = self.analyze_dimensions(&decl.dimensions, decl.location);
        let ty = self.declared_type(decl.ty, decl.dimensions.len());

        if decl.ty == TypeName::Void {
            self.report_error(
                decl.location,
                "variables cannot have type void".to_string(),
            );
        }

        let symbol_id = self.declare_symbol(
            decl.name.clone(),
            SymbolKind::Variable,
            decl.is_const,
            ty.clone(),
            decl.location,
        );
        let init = decl
            .init
            .as_ref()
            .map(|initializer| self.analyze_initializer(initializer, &ty, decl.location));

        self.check_const_declaration(decl.is_const, init.as_ref(), decl.location);

        SemanticVarDecl {
            symbol_id,
            is_const: decl.is_const,
            ty,
            name: decl.name.clone(),
            dimensions,
            init,
        }
    }

    // 在当前作用域分析语句块
    fn analyze_block_in_current_scope(&mut self, block: &Block) -> SemanticBlock {
        let scope_id = self.current_scope();
        let statements = block
            .statements
            .iter()
            .map(|stmt| self.analyze_stmt(stmt))
            .collect();

        SemanticBlock { scope_id, statements }
    }

    // 在新作用域分析语句块
    fn analyze_block_with_new_scope(&mut self, block: &Block) -> SemanticBlock {
        self.push_scope();
        let analyzed = self.analyze_block_in_current_scope(block);
        self.pop_scope();
        analyzed
    }

    // 分析一条语句
    fn analyze_stmt(&mut self, stmt: &Stmt) -> SemanticStmt {
        match stmt {
            Stmt::VarDecl { decls, .. } => SemanticStmt::VarDecl(
                decls
                    .iter()
                    .map(|decl| self.analyze_local_decl(decl))
                    .collect(),
            ),
            Stmt::If {
                location: _,
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = self.analyze_expr(condition);
                self.require_condition(&condition);

                SemanticStmt::If {
                    condition,
                    then_branch: Box::new(self.analyze_stmt(then_branch)),
                    else_branch: else_branch
                        .as_ref()
                        .map(|branch| Box::new(self.analyze_stmt(branch))),
                }
            }
            Stmt::While {
                location: _,
                condition,
                body,
            } => {
                let condition = self.analyze_expr(condition);
                self.require_condition(&condition);
                self.loop_depth += 1;
                let body = Box::new(self.analyze_stmt(body));
                self.loop_depth -= 1;

                SemanticStmt::While { condition, body }
            }
            Stmt::For {
                location: _,
                init,
                condition,
                update,
                body,
            } => {
                let mut scope_id = None;
                if matches!(init, Some(ForInit::Decl(_))) {
                    scope_id = Some(self.push_scope());
                }

                let init = init.as_ref().map(|init| match init {
                    ForInit::Decl(decls) => SemanticForInit::Decl(
                        decls
                            .iter()
                            .map(|decl| self.analyze_local_decl(decl))
                            .collect(),
                    ),
                    ForInit::Expr(expr) => SemanticForInit::Expr(self.analyze_expr(expr)),
                });

                let condition = condition.as_ref().map(|expr| self.analyze_expr(expr));
                if let Some(condition) = &condition {
                    self.require_condition(condition);
                }
                let update = update.as_ref().map(|expr| self.analyze_expr(expr));

                self.loop_depth += 1;
                let body = Box::new(self.analyze_stmt(body));
                self.loop_depth -= 1;

                if scope_id.is_some() {
                    self.pop_scope();
                }

                SemanticStmt::For {
                    scope_id,
                    init,
                    condition,
                    update,
                    body,
                }
            }
            Stmt::Return { location, expr } => {
                let analyzed = expr.as_ref().map(|expr| self.analyze_expr(expr));
                self.check_return(*location, analyzed.as_ref());
                SemanticStmt::Return(analyzed)
            }
            Stmt::Break { location } => {
                if self.loop_depth == 0 {
                    self.report_error(
                        *location,
                        "break can only appear inside a loop".to_string(),
                    );
                }
                SemanticStmt::Break
            }
            Stmt::Continue { location } => {
                if self.loop_depth == 0 {
                    self.report_error(
                        *location,
                        "continue can only appear inside a loop".to_string(),
                    );
                }
                SemanticStmt::Continue
            }
            Stmt::Expr { expr, .. } => SemanticStmt::Expr(self.analyze_expr(expr)),
            Stmt::Block(block) => SemanticStmt::Block(self.analyze_block_with_new_scope(block)),
            Stmt::Empty { .. } => SemanticStmt::Empty,
        }
    }

    // 分析初始化器
    fn analyze_initializer(
        &mut self,
        initializer: &Initializer,
        expected_ty: &SemanticType,
        location: SourceLocation,
    ) -> SemanticInitializer {
        match initializer {
            Initializer::Expr(expr) => {
                let expr = self.analyze_expr(expr);
                if !self.is_assignable(expected_ty, &expr.ty) {
                    self.report_error(
                        location,
                        format!(
                            "cannot initialize {} with {}",
                            self.type_to_string(expected_ty),
                            self.type_to_string(&expr.ty)
                        ),
                    );
                }
                SemanticInitializer::Expr(expr)
            }
            Initializer::List(items) => {
                let element_ty = match expected_ty {
                    SemanticType::Array { element, rank } if *rank > 1 => SemanticType::Array {
                        element: *element,
                        rank: rank - 1,
                    },
                    SemanticType::Array { element, rank: 1 } => self.scalar_type(*element),
                    SemanticType::Error => SemanticType::Error,
                    _ => {
                        self.report_error(
                            location,
                            format!(
                                "brace initializer requires an array type, found {}",
                                self.type_to_string(expected_ty)
                            ),
                        );
                        SemanticType::Error
                    }
                };

                SemanticInitializer::List(
                    items
                        .iter()
                        .map(|item| self.analyze_initializer(item, &element_ty, location))
                        .collect(),
                )
            }
        }
    }

    // 分析表达式
    fn analyze_expr(&mut self, expr: &Expr) -> SemanticExpr {
        match &expr.kind {
            ExprKind::Ident(name) => {
                let symbol_id = self.lookup_symbol(name);
                let ty = symbol_id
                    .map(|id| self.symbols[id.0].ty.clone())
                    .unwrap_or_else(|| {
                        self.report_error(
                            expr.location,
                            "use of undeclared identifier".to_string(),
                        );
                        SemanticType::Error
                    });

                SemanticExpr {
                    location: expr.location,
                    ty,
                    kind: SemanticExprKind::Ident {
                        name: name.clone(),
                        symbol_id,
                    },
                }
            }
            ExprKind::IntLiteral(value) => SemanticExpr {
                location: expr.location,
                ty: SemanticType::Int,
                kind: SemanticExprKind::IntLiteral(value.clone()),
            },
            ExprKind::FloatLiteral(value) => SemanticExpr {
                location: expr.location,
                ty: SemanticType::Float,
                kind: SemanticExprKind::FloatLiteral(value.clone()),
            },
            ExprKind::Unary { op, expr } => self.analyze_unary(expr.location, *op, expr),
            ExprKind::Binary { op, left, right } => self.analyze_binary(expr.location, *op, left, right),
            ExprKind::Assign { target, value } => self.analyze_assign(expr.location, target, value),
            ExprKind::Call { callee, args } => self.analyze_call(expr.location, callee, args),
            ExprKind::Index { array, index } => self.analyze_index(expr.location, array, index),
            ExprKind::Postfix { op, expr } => self.analyze_postfix(expr.location, *op, expr),
        }
    }

    // 分析一元表达式
    fn analyze_unary(&mut self, location: SourceLocation, op: UnaryOp, expr: &Expr) -> SemanticExpr {
        let expr = self.analyze_expr(expr);
        let ty = match op {
            UnaryOp::Plus | UnaryOp::Minus => {
                if self.is_numeric_scalar(&expr.ty) {
                    expr.ty.clone()
                } else {
                    self.report_error(
                        location,
                        format!(
                            "operator requires a numeric operand, found {}",
                            self.type_to_string(&expr.ty)
                        ),
                    );
                    SemanticType::Error
                }
            }
            UnaryOp::Not => {
                if self.is_scalar(&expr.ty) {
                    SemanticType::Int
                } else {
                    self.report_error(
                        location,
                        format!(
                            "operator requires a scalar operand, found {}",
                            self.type_to_string(&expr.ty)
                        ),
                    );
                    SemanticType::Error
                }
            }
            UnaryOp::PreInc | UnaryOp::PreDec => {
                if !self.is_mutable_lvalue(&expr) {
                    self.report_error(
                        location,
                        "operand must be a modifiable lvalue".to_string(),
                    );
                    SemanticType::Error
                } else if !self.is_numeric_scalar(&expr.ty) {
                    self.report_error(
                        location,
                        format!(
                            "operand must be a numeric scalar, found {}",
                            self.type_to_string(&expr.ty)
                        ),
                    );
                    SemanticType::Error
                } else {
                    expr.ty.clone()
                }
            }
        };

        SemanticExpr {
            location,
            ty,
            kind: SemanticExprKind::Unary {
                op,
                expr: Box::new(expr),
            },
        }
    }

    // 分析二元表达式
    fn analyze_binary(&mut self, location: SourceLocation, op: BinaryOp, left: &Expr, right: &Expr) -> SemanticExpr {
        let left = self.analyze_expr(left);
        let right = self.analyze_expr(right);
        let ty = match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                if self.is_numeric_scalar(&left.ty) && self.is_numeric_scalar(&right.ty) {
                    self.promote_numeric(&left.ty, &right.ty)
                } else {
                    self.report_error(
                        location,
                        format!(
                            "operator requires numeric scalars, found {} and {}",
                            self.type_to_string(&left.ty),
                            self.type_to_string(&right.ty)
                        ),
                    );
                    SemanticType::Error
                }
            }
            BinaryOp::Mod => {
                if left.ty == SemanticType::Int && right.ty == SemanticType::Int {
                    SemanticType::Int
                } else {
                    self.report_error(
                        location,
                        format!(
                            "operator % requires int operands, found {} and {}",
                            self.type_to_string(&left.ty),
                            self.type_to_string(&right.ty)
                        ),
                    );
                    SemanticType::Error
                }
            }
            BinaryOp::Eq
            | BinaryOp::Neq
            | BinaryOp::Lt
            | BinaryOp::Le
            | BinaryOp::Gt
            | BinaryOp::Ge => {
                if self.is_numeric_scalar(&left.ty) && self.is_numeric_scalar(&right.ty) {
                    SemanticType::Int
                } else {
                    self.report_error(
                        location,
                        format!(
                            "comparison requires numeric scalars, found {} and {}",
                            self.type_to_string(&left.ty),
                            self.type_to_string(&right.ty)
                        ),
                    );
                    SemanticType::Error
                }
            }
            BinaryOp::And | BinaryOp::Or => {
                if self.is_scalar(&left.ty) && self.is_scalar(&right.ty) {
                    SemanticType::Int
                } else {
                    self.report_error(
                        location,
                        format!(
                            "logical operator requires scalar operands, found {} and {}",
                            self.type_to_string(&left.ty),
                            self.type_to_string(&right.ty)
                        ),
                    );
                    SemanticType::Error
                }
            }
        };

        SemanticExpr {
            location,
            ty,
            kind: SemanticExprKind::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            },
        }
    }

    // 分析赋值表达式
    fn analyze_assign(&mut self, location: SourceLocation, target: &Expr, value: &Expr) -> SemanticExpr {
        let target = self.analyze_expr(target);
        let value = self.analyze_expr(value);

        let ty = if !self.is_mutable_lvalue(&target) {
            self.report_error(
                location,
                "left-hand side must be a modifiable lvalue".to_string(),
            );
            SemanticType::Error
        } else if !self.is_assignable(&target.ty, &value.ty) {
            self.report_error(
                location,
                format!(
                    "cannot assign {} to {}",
                    self.type_to_string(&value.ty),
                    self.type_to_string(&target.ty)
                ),
            );
            SemanticType::Error
        } else {
            target.ty.clone()
        };

        SemanticExpr {
            location,
            ty,
            kind: SemanticExprKind::Assign {
                target: Box::new(target),
                value: Box::new(value),
            },
        }
    }

    // 分析函数调用
    fn analyze_call(&mut self, location: SourceLocation, callee: &Expr, args: &[Expr]) -> SemanticExpr {
        let callee = self.analyze_expr(callee);
        let args: Vec<_> = args.iter().map(|arg| self.analyze_expr(arg)).collect();

        let (ty, function_symbol) = match &callee.ty {
            SemanticType::Function { return_type, params } => {
                if params.len() != args.len() {
                    self.report_error(
                        location,
                        format!(
                            "expected {} arguments, found {}",
                            params.len(),
                            args.len()
                        ),
                    );
                }

                for (index, (expected, actual)) in params.iter().zip(args.iter()).enumerate() {
                    if !self.is_assignable(expected, &actual.ty) {
                        self.report_error(
                            args[index].location,
                            format!(
                                "expected {}, found {}",
                                self.type_to_string(expected),
                                self.type_to_string(&actual.ty)
                            ),
                        );
                    }
                }

                (
                    self.scalar_type(*return_type),
                    match &callee.kind {
                        SemanticExprKind::Ident { symbol_id, .. } => *symbol_id,
                        _ => None,
                    },
                )
            }
            SemanticType::Error => (SemanticType::Error, None),
            other => {
                self.report_error(
                    location,
                    format!("callee is not a function, found {}", self.type_to_string(other)),
                );
                (SemanticType::Error, None)
            }
        };

        SemanticExpr {
            location,
            ty,
            kind: SemanticExprKind::Call {
                callee: Box::new(callee),
                args,
                function_symbol,
            },
        }
    }

    // 分析数组下标访问
    fn analyze_index(&mut self, location: SourceLocation, array: &Expr, index: &Expr) -> SemanticExpr {
        let array = self.analyze_expr(array);
        let index = self.analyze_expr(index);

        if index.ty != SemanticType::Int && index.ty != SemanticType::Error {
            self.report_error(
                index.location,
                format!("array index must be int, found {}", self.type_to_string(&index.ty)),
            );
        }

        let ty = match &array.ty {
            SemanticType::Array { element, rank } if *rank > 1 => SemanticType::Array {
                element: *element,
                rank: rank - 1,
            },
            SemanticType::Array { element, rank: 1 } => self.scalar_type(*element),
            SemanticType::Error => SemanticType::Error,
            other => {
                self.report_error(
                    location,
                    format!("cannot index into {}", self.type_to_string(other)),
                );
                SemanticType::Error
            }
        };

        SemanticExpr {
            location,
            ty,
            kind: SemanticExprKind::Index {
                array: Box::new(array),
                index: Box::new(index),
            },
        }
    }

    // 分析后缀表达式
    fn analyze_postfix(&mut self, location: SourceLocation, op: PostfixOp, expr: &Expr) -> SemanticExpr {
        let expr = self.analyze_expr(expr);
        let ty = if !self.is_mutable_lvalue(&expr) {
            self.report_error(
                location,
                "operand must be a modifiable lvalue".to_string(),
            );
            SemanticType::Error
        } else if !self.is_numeric_scalar(&expr.ty) {
            self.report_error(
                location,
                format!(
                    "operand must be a numeric scalar, found {}",
                    self.type_to_string(&expr.ty)
                ),
            );
            SemanticType::Error
        } else {
            expr.ty.clone()
        };

        SemanticExpr {
            location,
            ty,
            kind: SemanticExprKind::Postfix {
                op,
                expr: Box::new(expr),
            },
        }
    }

    // 分析数组维度
    fn analyze_dimensions(
        &mut self,
        dimensions: &[Option<Expr>],
        _location: SourceLocation,
    ) -> Vec<Option<SemanticExpr>> {
        dimensions
            .iter()
            .map(|dimension| {
                dimension.as_ref().map(|expr| {
                    let expr = self.analyze_expr(expr);
                    if expr.ty != SemanticType::Int && expr.ty != SemanticType::Error {
                        self.report_error(
                            expr.location,
                            format!(
                                "array dimension must be int, found {}",
                                self.type_to_string(&expr.ty)
                            ),
                        );
                    }
                    expr
                })
            })
            .collect()
    }

    // 检查返回语句
    fn check_return(&mut self, location: SourceLocation, expr: Option<&SemanticExpr>) {
        match (self.current_return_type, expr) {
            (Some(TypeName::Void), Some(expr)) => self.report_error(
                location,
                format!(
                    "void function cannot return a value of type {}",
                    self.type_to_string(&expr.ty)
                ),
            ),
            (Some(TypeName::Void), None) => {}
            (Some(expected), Some(expr)) => {
                let expected = self.scalar_type(expected);
                if !self.is_assignable(&expected, &expr.ty) {
                    self.report_error(
                        location,
                        format!(
                            "expected return type {}, found {}",
                            self.type_to_string(&expected),
                            self.type_to_string(&expr.ty)
                        ),
                    );
                }
            }
            (Some(expected), None) => self.report_error(
                location,
                format!(
                    "missing return value for function returning {}",
                    self.type_to_string(&self.scalar_type(expected))
                ),
            ),
            (None, _) => {}
        }
    }

    // 检查条件表达式
    fn require_condition(&mut self, expr: &SemanticExpr) {
        if !self.is_scalar(&expr.ty) && expr.ty != SemanticType::Error {
            self.report_error(
                expr.location,
                format!(
                    "condition must be a scalar expression, found {}",
                    self.type_to_string(&expr.ty)
                ),
            );
        }
    }

    // 计算声明类型
    fn declared_type(&self, base: TypeName, rank: usize) -> SemanticType {
        if rank == 0 {
            self.scalar_type(base)
        } else {
            SemanticType::Array { element: base, rank }
        }
    }

    // 转换为标量类型
    fn scalar_type(&self, ty: TypeName) -> SemanticType {
        match ty {
            TypeName::Int => SemanticType::Int,
            TypeName::Float => SemanticType::Float,
            TypeName::Void => SemanticType::Void,
        }
    }

    // 推导数值提升结果
    fn promote_numeric(&self, left: &SemanticType, right: &SemanticType) -> SemanticType {
        if left == &SemanticType::Float || right == &SemanticType::Float {
            SemanticType::Float
        } else if left == &SemanticType::Error || right == &SemanticType::Error {
            SemanticType::Error
        } else {
            SemanticType::Int
        }
    }

    // 判断是否为标量类型
    fn is_scalar(&self, ty: &SemanticType) -> bool {
        matches!(ty, SemanticType::Int | SemanticType::Float)
    }

    // 判断是否为数值标量
    fn is_numeric_scalar(&self, ty: &SemanticType) -> bool {
        self.is_scalar(ty)
    }

    // 判断类型是否可赋值
    fn is_assignable(&self, expected: &SemanticType, actual: &SemanticType) -> bool {
        match (expected, actual) {
            (SemanticType::Error, _) | (_, SemanticType::Error) => true,
            (SemanticType::Int, SemanticType::Int)
            | (SemanticType::Int, SemanticType::Float)
            | (SemanticType::Float, SemanticType::Int)
            | (SemanticType::Float, SemanticType::Float) => true,
            (
                SemanticType::Array {
                    element: left_element,
                    rank: left_rank,
                },
                SemanticType::Array {
                    element: right_element,
                    rank: right_rank,
                },
            ) => left_element == right_element && left_rank == right_rank,
            _ => false,
        }
    }

    // 判断是否为可修改左值
    fn is_mutable_lvalue(&self, expr: &SemanticExpr) -> bool {
        match &expr.kind {
            SemanticExprKind::Ident { symbol_id, .. } => symbol_id
                .map(|id| {
                    let symbol = &self.symbols[id.0];
                    symbol.kind != SymbolKind::Function && !symbol.is_const
                })
                .unwrap_or(false),
            SemanticExprKind::Index { array, .. } => self.is_mutable_lvalue(array),
            _ => false,
        }
    }

    // 检查常量声明是否合法
    fn check_const_declaration(
        &mut self,
        is_const: bool,
        init: Option<&SemanticInitializer>,
        location: SourceLocation,
    ) {
        if !is_const {
            return;
        }

        let Some(init) = init else {
            self.report_error(
                location,
                "const declarations require an initializer".to_string(),
            );
            return;
        };

        if !self.is_const_initializer(init) {
            self.report_error(
                location,
                "const initializer must be a constant expression".to_string(),
            );
        }
    }

    // 判断是否为常量初始化器
    fn is_const_initializer(&self, initializer: &SemanticInitializer) -> bool {
        match initializer {
            SemanticInitializer::Expr(expr) => self.is_const_expr(expr),
            SemanticInitializer::List(items) => items.iter().all(|item| self.is_const_initializer(item)),
        }
    }

    // 判断是否为常量表达式
    fn is_const_expr(&self, expr: &SemanticExpr) -> bool {
        match &expr.kind {
            SemanticExprKind::Ident { symbol_id, .. } => symbol_id
                .map(|id| self.symbols[id.0].is_const)
                .unwrap_or(false),
            SemanticExprKind::IntLiteral(_) | SemanticExprKind::FloatLiteral(_) => true,
            SemanticExprKind::Unary { op, expr } => {
                matches!(op, UnaryOp::Plus | UnaryOp::Minus | UnaryOp::Not)
                    && self.is_const_expr(expr)
            }
            SemanticExprKind::Binary { left, right, .. } => {
                self.is_const_expr(left) && self.is_const_expr(right)
            }
            SemanticExprKind::Index { array, index } => {
                self.is_const_expr(array) && self.is_const_expr(index)
            }
            SemanticExprKind::Assign { .. }
            | SemanticExprKind::Call { .. }
            | SemanticExprKind::Postfix { .. } => false,
        }
    }

    // 格式化语义类型
    fn type_to_string(&self, ty: &SemanticType) -> String {
        match ty {
            SemanticType::Int => "int".to_string(),
            SemanticType::Float => "float".to_string(),
            SemanticType::Void => "void".to_string(),
            SemanticType::Array { element, rank } => {
                format!("{}{}", self.type_name_to_string(*element), "[]".repeat(*rank))
            }
            SemanticType::Function { return_type, params } => format!(
                "fn({}) -> {}",
                params
                    .iter()
                    .map(|param| self.type_to_string(param))
                    .collect::<Vec<_>>()
                    .join(", "),
                self.type_name_to_string(*return_type)
            ),
            SemanticType::Error => "<error>".to_string(),
        }
    }

    // 格式化基础类型名
    fn type_name_to_string(&self, ty: TypeName) -> &'static str {
        match ty {
            TypeName::Int => "int",
            TypeName::Float => "float",
            TypeName::Void => "void",
        }
    }

    // 声明符号
    fn declare_symbol(
        &mut self,
        name: String,
        kind: SymbolKind,
        is_const: bool,
        ty: SemanticType,
        context: SourceLocation,
    ) -> Option<SymbolId> {
        let current_scope = self.current_scope();
        let scope_symbols = &mut self.scopes[current_scope.0].symbols;
        if scope_symbols.contains_key(&name) {
            self.report_error(
                context,
                format!("duplicate declaration of '{}'", name),
            );
            return None;
        }

        let id = SymbolId(self.symbols.len());
        self.symbols.push(Symbol {
            id,
            name: name.clone(),
            kind,
            is_const,
            ty,
            scope_id: current_scope,
        });
        scope_symbols.insert(name, id);
        Some(id)
    }

    // 查找符号
    fn lookup_symbol(&self, name: &str) -> Option<SymbolId> {
        for scope_id in self.scope_stack.iter().rev() {
            if let Some(symbol_id) = self.scopes[scope_id.0].symbols.get(name) {
                return Some(*symbol_id);
            }
        }
        None
    }

    // 进入新作用域
    fn push_scope(&mut self) -> ScopeId {
        let parent = Some(self.current_scope());
        let scope_id = ScopeId(self.scopes.len());
        self.scopes.push(ScopeInfo {
            id: scope_id,
            parent,
            symbols: HashMap::new(),
        });
        self.scope_stack.push(scope_id);
        scope_id
    }

    // 退出当前作用域
    fn pop_scope(&mut self) {
        if self.scope_stack.len() > 1 {
            self.scope_stack.pop();
        }
    }

    // 获取当前作用域
    fn current_scope(&self) -> ScopeId {
        *self.scope_stack.last().expect("scope stack should not be empty")
    }

    // 记录语义错误
    fn report_error(&mut self, location: SourceLocation, message: String) {
        self.errors.push(SemanticError { location, message });
    }
}