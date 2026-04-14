#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, Copy)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub enum Item {
    Function(FunctionDef),
    GlobalDecl(Vec<VarDecl>),
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub location: SourceLocation,
    pub return_type: TypeName,
    pub name: String,
    pub params: Vec<Param>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub location: SourceLocation,
    pub ty: TypeName,
    pub name: String,
    pub dimensions: Vec<Option<Expr>>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub location: SourceLocation,
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    VarDecl {
        location: SourceLocation,
        decls: Vec<VarDecl>,
    },
    If {
        location: SourceLocation,
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        location: SourceLocation,
        condition: Expr,
        body: Box<Stmt>,
    },
    For {
        location: SourceLocation,
        init: Option<ForInit>,
        condition: Option<Expr>,
        update: Option<Expr>,
        body: Box<Stmt>,
    },
    Return {
        location: SourceLocation,
        expr: Option<Expr>,
    },
    Break {
        location: SourceLocation,
    },
    Continue {
        location: SourceLocation,
    },
    Expr {
        location: SourceLocation,
        expr: Expr,
    },
    Block(Block),
    Empty {
        location: SourceLocation,
    },
}

#[derive(Debug, Clone)]
pub enum ForInit {
    Decl(Vec<VarDecl>),
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub location: SourceLocation,
    pub is_const: bool,
    pub ty: TypeName,
    pub name: String,
    pub dimensions: Vec<Option<Expr>>,
    pub init: Option<Initializer>,
}

#[derive(Debug, Clone)]
pub enum Initializer {
    Expr(Expr),
    List(Vec<Initializer>),
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub location: SourceLocation,
    pub kind: ExprKind,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Ident(String),
    IntLiteral(String),
    FloatLiteral(String),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Index {
        array: Box<Expr>,
        index: Box<Expr>,
    },
    Postfix {
        op: PostfixOp,
        expr: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeName {
    Int,
    Float,
    Void,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Plus,
    Minus,
    Not,
    PreInc,
    PreDec,
}

#[derive(Debug, Clone, Copy)]
pub enum PostfixOp {
    PostInc,
    PostDec,
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}