use crate::ast::{
    BinaryOp, Block, Expr, ExprKind, ForInit, FunctionDef, Initializer, Item, Param, PostfixOp,
    Program, SourceLocation, Stmt, TypeName, UnaryOp, VarDecl,
};
use crate::lexer::{Token, TokenKind};

#[derive(Debug, Clone)]
pub struct ParseError {
    pub location: SourceLocation,
    pub message: String,
}

type ParseResult<T> = Result<T, ParseError>;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    // 创建语法分析器
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    // 解析完整程序
    pub fn parse_program(&mut self) -> ParseResult<Program> {
        let mut items = Vec::new();

        while !self.at_kind(TokenKind::Eof) {
            items.push(self.parse_item()?);
        }

        Ok(Program { items })
    }

    // 解析顶层条目
    fn parse_item(&mut self) -> ParseResult<Item> {
        let (is_const, ty) = self.parse_decl_specifier()?;
        let (name, name_location) = self.expect_ident()?;

        if self.match_kind(TokenKind::LParen) {
            if is_const {
                return Err(self.error_here("functions cannot be const-qualified"));
            }
            let params = self.parse_param_list()?;
            let body = self.parse_block()?;
            Ok(Item::Function(FunctionDef {
                location: name_location,
                return_type: ty,
                name,
                params,
                body,
            }))
        } else {
            let decls = self.parse_var_decl_list_after_first(is_const, ty, name, name_location)?;
            self.expect_kind(TokenKind::Semicolon)?;
            Ok(Item::GlobalDecl(decls))
        }
    }

    // 解析语句块
    fn parse_block(&mut self) -> ParseResult<Block> {
        let location = self.current_location();
        self.expect_kind(TokenKind::LBrace)?;
        let mut statements = Vec::new();

        while !self.at_kind(TokenKind::RBrace) {
            if self.at_kind(TokenKind::Eof) {
                return Err(self.error_here("unexpected end of file while parsing block"));
            }
            statements.push(self.parse_stmt()?);
        }

        self.expect_kind(TokenKind::RBrace)?;
        Ok(Block { location, statements })
    }

    // 解析一条语句
    fn parse_stmt(&mut self) -> ParseResult<Stmt> {
        if self.at_kind(TokenKind::LBrace) {
            return Ok(Stmt::Block(self.parse_block()?));
        }

        if self.check_keyword("return") {
            let location = self.current_location();
            self.advance();
            if self.match_kind(TokenKind::Semicolon) {
                return Ok(Stmt::Return {
                    location,
                    expr: None,
                });
            }
            let expr = self.parse_expr()?;
            self.expect_kind(TokenKind::Semicolon)?;
            return Ok(Stmt::Return {
                location,
                expr: Some(expr),
            });
        }

        if self.check_keyword("if") {
            let location = self.current_location();
            self.advance();
            self.expect_kind(TokenKind::LParen)?;
            let condition = self.parse_expr()?;
            self.expect_kind(TokenKind::RParen)?;
            let then_branch = Box::new(self.parse_stmt()?);
            let else_branch = if self.check_keyword("else") {
                self.advance();
                Some(Box::new(self.parse_stmt()?))
            } else {
                None
            };
            return Ok(Stmt::If {
                location,
                condition,
                then_branch,
                else_branch,
            });
        }

        if self.check_keyword("while") {
            let location = self.current_location();
            self.advance();
            self.expect_kind(TokenKind::LParen)?;
            let condition = self.parse_expr()?;
            self.expect_kind(TokenKind::RParen)?;
            let body = Box::new(self.parse_stmt()?);
            return Ok(Stmt::While {
                location,
                condition,
                body,
            });
        }

        if self.check_keyword("for") {
            let location = self.current_location();
            self.advance();
            self.expect_kind(TokenKind::LParen)?;

            let init = if self.at_kind(TokenKind::Semicolon) {
                None
            } else if self.is_decl_start() {
                Some(ForInit::Decl(self.parse_var_decl_list()?))
            } else {
                Some(ForInit::Expr(self.parse_expr()?))
            };
            self.expect_kind(TokenKind::Semicolon)?;

            let condition = if self.at_kind(TokenKind::Semicolon) {
                None
            } else {
                Some(self.parse_expr()?)
            };
            self.expect_kind(TokenKind::Semicolon)?;

            let update = if self.at_kind(TokenKind::RParen) {
                None
            } else {
                Some(self.parse_expr()?)
            };
            self.expect_kind(TokenKind::RParen)?;

            let body = Box::new(self.parse_stmt()?);
            return Ok(Stmt::For {
                location,
                init,
                condition,
                update,
                body,
            });
        }

        if self.check_keyword("break") {
            let location = self.current_location();
            self.advance();
            self.expect_kind(TokenKind::Semicolon)?;
            return Ok(Stmt::Break { location });
        }

        if self.check_keyword("continue") {
            let location = self.current_location();
            self.advance();
            self.expect_kind(TokenKind::Semicolon)?;
            return Ok(Stmt::Continue { location });
        }

        if self.is_decl_start() {
            let decl = self.parse_var_decl_list()?;
            let location = decl[0].location;
            self.expect_kind(TokenKind::Semicolon)?;
            return Ok(Stmt::VarDecl {
                location,
                decls: decl,
            });
        }

        if self.at_kind(TokenKind::Semicolon) {
            let location = self.current_location();
            self.advance();
            return Ok(Stmt::Empty { location });
        }

        let expr = self.parse_expr()?;
        self.expect_kind(TokenKind::Semicolon)?;
        Ok(Stmt::Expr {
            location: expr.location,
            expr,
        })
    }

    // 解析变量声明列表
    fn parse_var_decl_list(&mut self) -> ParseResult<Vec<VarDecl>> {
        let (is_const, ty) = self.parse_decl_specifier()?;
        let (name, name_location) = self.expect_ident()?;
        self.parse_var_decl_list_after_first(is_const, ty, name, name_location)
    }

    // 解析剩余变量声明
    fn parse_var_decl_list_after_first(
        &mut self,
        is_const: bool,
        ty: TypeName,
        first_name: String,
        first_location: SourceLocation,
    ) -> ParseResult<Vec<VarDecl>> {
        let mut decls = vec![self.parse_var_decl_after_name(is_const, ty, first_name, first_location)?];

        while self.match_kind(TokenKind::Comma) {
            let (name, name_location) = self.expect_ident()?;
            decls.push(self.parse_var_decl_after_name(is_const, ty, name, name_location)?);
        }

        Ok(decls)
    }

    // 解析单个变量声明
    fn parse_var_decl_after_name(
        &mut self,
        is_const: bool,
        ty: TypeName,
        name: String,
        location: SourceLocation,
    ) -> ParseResult<VarDecl> {
        let dimensions = self.parse_dimensions()?;
        let init = if self.match_kind(TokenKind::Assign) {
            Some(self.parse_initializer()?)
        } else {
            None
        };

        Ok(VarDecl {
            location,
            is_const,
            ty,
            name,
            dimensions,
            init,
        })
    }

    // 解析声明说明符
    fn parse_decl_specifier(&mut self) -> ParseResult<(bool, TypeName)> {
        let is_const = self.match_keyword("const");
        let ty = self.parse_type_name()?;
        Ok((is_const, ty))
    }

    // 解析初始化器
    fn parse_initializer(&mut self) -> ParseResult<Initializer> {
        if self.match_kind(TokenKind::LBrace) {
            let mut items = Vec::new();

            if !self.at_kind(TokenKind::RBrace) {
                loop {
                    items.push(self.parse_initializer()?);
                    if !self.match_kind(TokenKind::Comma) {
                        break;
                    }
                    if self.at_kind(TokenKind::RBrace) {
                        break;
                    }
                }
            }

            self.expect_kind(TokenKind::RBrace)?;
            return Ok(Initializer::List(items));
        }

        Ok(Initializer::Expr(self.parse_expr()?))
    }

    // 解析表达式
    fn parse_expr(&mut self) -> ParseResult<Expr> {
        self.parse_assignment()
    }

    // 解析赋值表达式
    fn parse_assignment(&mut self) -> ParseResult<Expr> {
        let left = self.parse_logical_or()?;

        if self.match_kind(TokenKind::Assign) {
            let value = self.parse_assignment()?;
            return Ok(Expr {
                location: left.location,
                kind: ExprKind::Assign {
                    target: Box::new(left),
                    value: Box::new(value),
                },
            });
        }

        Ok(left)
    }

    // 解析逻辑或表达式
    fn parse_logical_or(&mut self) -> ParseResult<Expr> {
        self.parse_binary_left_assoc(Self::parse_logical_and, &[TokenKind::Or])
    }

    // 解析逻辑与表达式
    fn parse_logical_and(&mut self) -> ParseResult<Expr> {
        self.parse_binary_left_assoc(Self::parse_equality, &[TokenKind::And])
    }

    // 解析相等性表达式
    fn parse_equality(&mut self) -> ParseResult<Expr> {
        self.parse_binary_left_assoc(Self::parse_relational, &[TokenKind::Eq, TokenKind::Neq])
    }

    // 解析关系表达式
    fn parse_relational(&mut self) -> ParseResult<Expr> {
        self.parse_binary_left_assoc(
            Self::parse_additive,
            &[TokenKind::Lt, TokenKind::Le, TokenKind::Gt, TokenKind::Ge],
        )
    }

    // 解析加减表达式
    fn parse_additive(&mut self) -> ParseResult<Expr> {
        self.parse_binary_left_assoc(
            Self::parse_multiplicative,
            &[TokenKind::Plus, TokenKind::Minus],
        )
    }

    // 解析乘除模表达式
    fn parse_multiplicative(&mut self) -> ParseResult<Expr> {
        self.parse_binary_left_assoc(
            Self::parse_unary,
            &[TokenKind::Star, TokenKind::Slash, TokenKind::Percent],
        )
    }

    // 按左结合规则解析二元表达式
    fn parse_binary_left_assoc(
        &mut self,
        subparser: fn(&mut Self) -> ParseResult<Expr>,
        operators: &[TokenKind],
    ) -> ParseResult<Expr> {
        let mut expr = subparser(self)?;

        while operators.iter().any(|kind| self.at_kind(kind.clone())) {
            let op_kind = self.current().kind.clone();
            let location = self.current_location();
            self.advance();
            let rhs = subparser(self)?;
            expr = Expr {
                location,
                kind: ExprKind::Binary {
                    op: Self::binary_op_from_token(&op_kind)?,
                    left: Box::new(expr),
                    right: Box::new(rhs),
                },
            };
        }

        Ok(expr)
    }

    // 解析一元表达式
    fn parse_unary(&mut self) -> ParseResult<Expr> {
        if self.match_kind(TokenKind::Plus) {
            let location = self.previous_location();
            return Ok(Expr {
                location,
                kind: ExprKind::Unary {
                    op: UnaryOp::Plus,
                    expr: Box::new(self.parse_unary()?),
                },
            });
        }
        if self.match_kind(TokenKind::Minus) {
            let location = self.previous_location();
            return Ok(Expr {
                location,
                kind: ExprKind::Unary {
                    op: UnaryOp::Minus,
                    expr: Box::new(self.parse_unary()?),
                },
            });
        }
        if self.match_kind(TokenKind::Not) {
            let location = self.previous_location();
            return Ok(Expr {
                location,
                kind: ExprKind::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(self.parse_unary()?),
                },
            });
        }
        if self.match_kind(TokenKind::Inc) {
            let location = self.previous_location();
            return Ok(Expr {
                location,
                kind: ExprKind::Unary {
                    op: UnaryOp::PreInc,
                    expr: Box::new(self.parse_unary()?),
                },
            });
        }
        if self.match_kind(TokenKind::Dec) {
            let location = self.previous_location();
            return Ok(Expr {
                location,
                kind: ExprKind::Unary {
                    op: UnaryOp::PreDec,
                    expr: Box::new(self.parse_unary()?),
                },
            });
        }

        self.parse_postfix()
    }

    // 解析后缀表达式
    fn parse_postfix(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_kind(TokenKind::LParen) {
                let args = self.parse_argument_list()?;
                let location = expr.location;
                expr = Expr {
                    location,
                    kind: ExprKind::Call {
                        callee: Box::new(expr),
                        args,
                    },
                };
                continue;
            }

            if self.match_kind(TokenKind::LBracket) {
                let index = self.parse_expr()?;
                self.expect_kind(TokenKind::RBracket)?;
                let location = expr.location;
                expr = Expr {
                    location,
                    kind: ExprKind::Index {
                        array: Box::new(expr),
                        index: Box::new(index),
                    },
                };
                continue;
            }

            if self.match_kind(TokenKind::Inc) {
                let location = expr.location;
                expr = Expr {
                    location,
                    kind: ExprKind::Postfix {
                        op: PostfixOp::PostInc,
                        expr: Box::new(expr),
                    },
                };
                continue;
            }

            if self.match_kind(TokenKind::Dec) {
                let location = expr.location;
                expr = Expr {
                    location,
                    kind: ExprKind::Postfix {
                        op: PostfixOp::PostDec,
                        expr: Box::new(expr),
                    },
                };
                continue;
            }

            break;
        }

        Ok(expr)
    }

    // 解析基本表达式
    fn parse_primary(&mut self) -> ParseResult<Expr> {
        let token = self.current().clone();
        let location = self.current_location();

        match token.kind {
            TokenKind::Ident => {
                self.advance();
                Ok(Expr {
                    location,
                    kind: ExprKind::Ident(token.lexeme),
                })
            }
            TokenKind::IntLiteral => {
                self.advance();
                Ok(Expr {
                    location,
                    kind: ExprKind::IntLiteral(token.lexeme),
                })
            }
            TokenKind::FloatLiteral => {
                self.advance();
                Ok(Expr {
                    location,
                    kind: ExprKind::FloatLiteral(token.lexeme),
                })
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect_kind(TokenKind::RParen)?;
                Ok(expr)
            }
            _ => Err(self.error_here("expected expression")),
        }
    }

    // 解析函数参数列表
    fn parse_param_list(&mut self) -> ParseResult<Vec<Param>> {
        let mut params = Vec::new();

        if self.match_kind(TokenKind::RParen) {
            return Ok(params);
        }

        loop {
            let ty = self.parse_type_name()?;
            let (name, location) = self.expect_ident()?;
            let dimensions = self.parse_dimensions()?;
            params.push(Param {
                location,
                ty,
                name,
                dimensions,
            });

            if !self.match_kind(TokenKind::Comma) {
                break;
            }
        }

        self.expect_kind(TokenKind::RParen)?;
        Ok(params)
    }

    // 解析实参列表
    fn parse_argument_list(&mut self) -> ParseResult<Vec<Expr>> {
        let mut args = Vec::new();

        if self.match_kind(TokenKind::RParen) {
            return Ok(args);
        }

        loop {
            args.push(self.parse_expr()?);
            if !self.match_kind(TokenKind::Comma) {
                break;
            }
        }

        self.expect_kind(TokenKind::RParen)?;
        Ok(args)
    }

    // 解析数组维度
    fn parse_dimensions(&mut self) -> ParseResult<Vec<Option<Expr>>> {
        let mut dimensions = Vec::new();

        while self.match_kind(TokenKind::LBracket) {
            let size = if self.at_kind(TokenKind::RBracket) {
                None
            } else {
                Some(self.parse_expr()?)
            };
            self.expect_kind(TokenKind::RBracket)?;
            dimensions.push(size);
        }

        Ok(dimensions)
    }

    // 解析类型名称
    fn parse_type_name(&mut self) -> ParseResult<TypeName> {
        let token = self.current().clone();
        if token.kind != TokenKind::Keyword {
            return Err(self.error_here("expected type keyword"));
        }

        let ty = match token.lexeme.as_str() {
            "int" => TypeName::Int,
            "float" => TypeName::Float,
            "void" => TypeName::Void,
            _ => return Err(self.error_here("expected type keyword")),
        };

        self.advance();
        Ok(ty)
    }

    // 读取并校验标识符
    fn expect_ident(&mut self) -> ParseResult<(String, SourceLocation)> {
        let token = self.current().clone();
        if token.kind != TokenKind::Ident {
            return Err(self.error_here("expected identifier"));
        }
        let location = self.current_location();
        self.advance();
        Ok((token.lexeme, location))
    }

    // 校验当前记号类型
    fn expect_kind(&mut self, expected: TokenKind) -> ParseResult<()> {
        if self.match_kind(expected.clone()) {
            return Ok(());
        }
        Err(ParseError {
            location: self.current_location(),
            message: format!(
                "expected {:?}, found {:?} '{}'",
                expected,
                self.current().kind,
                self.current().lexeme
            ),
        })
    }

    // 尝试匹配指定记号
    fn match_kind(&mut self, expected: TokenKind) -> bool {
        if self.at_kind(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    // 判断当前是否为指定记号
    fn at_kind(&self, expected: TokenKind) -> bool {
        self.current().kind == expected
    }

    // 检查当前关键字
    fn check_keyword(&self, keyword: &str) -> bool {
        self.current().kind == TokenKind::Keyword && self.current().lexeme == keyword
    }

    // 尝试匹配关键字
    fn match_keyword(&mut self, keyword: &str) -> bool {
        if self.check_keyword(keyword) {
            self.advance();
            true
        } else {
            false
        }
    }

    // 判断是否为类型关键字
    fn is_type_keyword(&self) -> bool {
        self.check_keyword("int") || self.check_keyword("float") || self.check_keyword("void")
    }

    // 判断是否为声明起始
    fn is_decl_start(&self) -> bool {
        self.check_keyword("const") || self.is_type_keyword()
    }

    // 获取当前记号
    fn current(&self) -> &Token {
        &self.tokens[self.pos]
    }

    // 获取当前位置
    fn current_location(&self) -> SourceLocation {
        SourceLocation {
            line: self.current().line,
            column: self.current().column,
        }
    }

    // 获取上一个位置
    fn previous_location(&self) -> SourceLocation {
        let index = self.pos.saturating_sub(1);
        SourceLocation {
            line: self.tokens[index].line,
            column: self.tokens[index].column,
        }
    }

    // 前进到下一个记号
    fn advance(&mut self) {
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
    }

    // 将记号映射为二元运算符
    fn binary_op_from_token(kind: &TokenKind) -> ParseResult<BinaryOp> {
        let op = match kind {
            TokenKind::Plus => BinaryOp::Add,
            TokenKind::Minus => BinaryOp::Sub,
            TokenKind::Star => BinaryOp::Mul,
            TokenKind::Slash => BinaryOp::Div,
            TokenKind::Percent => BinaryOp::Mod,
            TokenKind::Eq => BinaryOp::Eq,
            TokenKind::Neq => BinaryOp::Neq,
            TokenKind::Lt => BinaryOp::Lt,
            TokenKind::Le => BinaryOp::Le,
            TokenKind::Gt => BinaryOp::Gt,
            TokenKind::Ge => BinaryOp::Ge,
            TokenKind::And => BinaryOp::And,
            TokenKind::Or => BinaryOp::Or,
            _ => {
                return Err(ParseError {
                    location: SourceLocation { line: 0, column: 0 },
                    message: format!("unsupported binary operator: {:?}", kind),
                })
            }
        };
        Ok(op)
    }

    // 生成当前位置错误
    fn error_here(&self, message: &str) -> ParseError {
        ParseError {
            location: self.current_location(),
            message: message.to_string(),
        }
    }
}