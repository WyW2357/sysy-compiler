use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Ident,
    Keyword,
    IntLiteral,
    FloatLiteral,

    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Assign,
    Eq,
    Neq,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Not,
    Inc,
    Dec,

    Semicolon,
    Comma,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,

    Eof,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

pub struct Scanner {
    src: Vec<u8>,
    len: usize,
    pos: usize,
    line: usize,
    column: usize,
    keywords: HashSet<&'static str>,
}

impl Scanner {
    pub fn new(src: &str) -> Self {
        let keywords: HashSet<&'static str> = [
            "int", "float", "void", "const", "if", "else", "while", "for", "return",
            "break", "continue",
        ].iter().copied().collect();

        Scanner {
            src: src.as_bytes().to_vec(),
            len: src.len(),
            pos: 0,
            line: 1,
            column: 1,
            keywords,
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        if self.pos < self.len { Some(self.src[self.pos]) } else { None }
    }

    fn peek_next_byte(&self) -> Option<u8> {
        if self.pos + 1 < self.len { Some(self.src[self.pos + 1]) } else { None }
    }

    fn bump(&mut self) -> Option<u8> {
        if self.pos >= self.len { return None; }
        let b = self.src[self.pos];
        self.pos += 1;
        if b == b'\n' { self.line += 1; self.column = 1; } else { self.column += 1; }
        Some(b)
    }

    fn make_token(&self, kind: TokenKind, start: usize, end: usize, line: usize, column: usize) -> Token {
        let lexeme = String::from_utf8_lossy(&self.src[start..end]).to_string();
        Token { kind, lexeme, line, column }
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek_byte() {
                Some(b) if b.is_ascii_whitespace() => { self.bump(); }
                Some(b'/') => match self.peek_next_byte() {
                    Some(b'/') => {
                        self.bump(); self.bump();
                        while let Some(c) = self.peek_byte() { if c == b'\n' { break; } self.bump(); }
                    }
                    Some(b'*') => {
                        self.bump(); self.bump();
                        loop {
                            match self.peek_byte() {
                                None => break,
                                Some(b'*') => { self.bump(); if let Some(b'/') = self.peek_byte() { self.bump(); break; } }
                                _ => { self.bump(); }
                            }
                        }
                    }
                    _ => break,
                },
                _ => break,
            }
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();

        let start = self.pos;
        let line = self.line;
        let column = self.column;

        let b_opt = self.peek_byte();
        if b_opt.is_none() { return self.make_token(TokenKind::Eof, start, start, line, column); }
        let b = b_opt.unwrap();

        if is_alpha(b) || b == b'_' {
            self.bump();
            while let Some(c) = self.peek_byte() { if is_alnum(c) || c == b'_' { self.bump(); } else { break; } }
            let end = self.pos;
            let lex = String::from_utf8_lossy(&self.src[start..end]).to_string();
            if self.keywords.contains(lex.as_str()) { return self.make_token(TokenKind::Keyword, start, end, line, column); }
            return self.make_token(TokenKind::Ident, start, end, line, column);
        }

        if is_digit(b) || (b == b'.' && self.peek_next_byte().map_or(false, |n| is_digit(n))) {
            let mut seen_dot = false;
            let mut seen_exp = false;

            if b == b'.' {
                seen_dot = true; self.bump();
                while let Some(c) = self.peek_byte() { if is_digit(c) { self.bump(); } else { break; } }
            } else {
                while let Some(c) = self.peek_byte() { if is_digit(c) { self.bump(); } else { break; } }
                if let Some(b'.') = self.peek_byte() { seen_dot = true; self.bump(); while let Some(c) = self.peek_byte() { if is_digit(c) { self.bump(); } else { break; } } }
            }

            if let Some(c) = self.peek_byte() {
                if c == b'e' || c == b'E' {
                    seen_exp = true; self.bump();
                    if let Some(sign) = self.peek_byte() { if sign == b'+' || sign == b'-' { self.bump(); } }
                    let mut exp_digits = 0;
                    while let Some(d) = self.peek_byte() { if is_digit(d) { self.bump(); exp_digits += 1; } else { break; } }
                    if exp_digits == 0 { /* malformed exponent, leave as is */ }
                }
            }

            let end = self.pos;
            if seen_dot || seen_exp { return self.make_token(TokenKind::FloatLiteral, start, end, line, column); }
            return self.make_token(TokenKind::IntLiteral, start, end, line, column);
        }

        match b {
            b'+' => { self.bump(); if let Some(b'+') = self.peek_byte() { self.bump(); return self.make_token(TokenKind::Inc, start, self.pos, line, column); } return self.make_token(TokenKind::Plus, start, self.pos, line, column); }
            b'-' => { self.bump(); if let Some(b'-') = self.peek_byte() { self.bump(); return self.make_token(TokenKind::Dec, start, self.pos, line, column); } return self.make_token(TokenKind::Minus, start, self.pos, line, column); }
            b'*' => { self.bump(); return self.make_token(TokenKind::Star, start, self.pos, line, column); }
            b'%' => { self.bump(); return self.make_token(TokenKind::Percent, start, self.pos, line, column); }
            b'/' => { self.bump(); return self.make_token(TokenKind::Slash, start, self.pos, line, column); }
            b'=' => { self.bump(); if let Some(b'=') = self.peek_byte() { self.bump(); return self.make_token(TokenKind::Eq, start, self.pos, line, column); } return self.make_token(TokenKind::Assign, start, self.pos, line, column); }
            b'!' => { self.bump(); if let Some(b'=') = self.peek_byte() { self.bump(); return self.make_token(TokenKind::Neq, start, self.pos, line, column); } return self.make_token(TokenKind::Not, start, self.pos, line, column); }
            b'<' => { self.bump(); if let Some(b'=') = self.peek_byte() { self.bump(); return self.make_token(TokenKind::Le, start, self.pos, line, column); } return self.make_token(TokenKind::Lt, start, self.pos, line, column); }
            b'>' => { self.bump(); if let Some(b'=') = self.peek_byte() { self.bump(); return self.make_token(TokenKind::Ge, start, self.pos, line, column); } return self.make_token(TokenKind::Gt, start, self.pos, line, column); }
            b'&' => {
                self.bump();
                if let Some(b'&') = self.peek_byte() {
                    self.bump();
                    return self.make_token(TokenKind::And, start, self.pos, line, column);
                }
                return self.make_token(TokenKind::Unknown, start, self.pos, line, column);
            }
            b'|' => {
                self.bump();
                if let Some(b'|') = self.peek_byte() {
                    self.bump();
                    return self.make_token(TokenKind::Or, start, self.pos, line, column);
                }
                return self.make_token(TokenKind::Unknown, start, self.pos, line, column);
            }
            b';' => { self.bump(); return self.make_token(TokenKind::Semicolon, start, self.pos, line, column); }
            b',' => { self.bump(); return self.make_token(TokenKind::Comma, start, self.pos, line, column); }
            b'(' => { self.bump(); return self.make_token(TokenKind::LParen, start, self.pos, line, column); }
            b')' => { self.bump(); return self.make_token(TokenKind::RParen, start, self.pos, line, column); }
            b'[' => { self.bump(); return self.make_token(TokenKind::LBracket, start, self.pos, line, column); }
            b']' => { self.bump(); return self.make_token(TokenKind::RBracket, start, self.pos, line, column); }            
            b'{' => { self.bump(); return self.make_token(TokenKind::LBrace, start, self.pos, line, column); }
            b'}' => { self.bump(); return self.make_token(TokenKind::RBrace, start, self.pos, line, column); }
            _ => {}
        }

        self.bump();
        self.make_token(TokenKind::Unknown, start, self.pos, line, column)
    }
}

pub fn tokenize(src: &str) -> Vec<Token> {
    let mut scanner = Scanner::new(src);
    let mut tokens = Vec::new();

    loop {
        let token = scanner.next_token();
        let is_eof = token.kind == TokenKind::Eof;
        tokens.push(token);
        if is_eof { break; }
    }

    tokens
}

fn is_alpha(b: u8) -> bool { (b'A' <= b && b <= b'Z') || (b'a' <= b && b <= b'z') }
fn is_digit(b: u8) -> bool { b'0' <= b && b <= b'9' }
fn is_alnum(b: u8) -> bool { is_alpha(b) || is_digit(b) }