use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
enum TokenKind {
    Number(f64), String(String), Ident(String),
    Plus, Minus, Star, Slash, Percent,
    LeftParen, RightParen, LeftBrace, RightBrace, LeftBracket, RightBracket,
    Comma, Semicolon,
    Equal, EqualEqual, Bang, BangEqual, Less, LessEqual, Greater, GreaterEqual,
    And, Or,
    Let, Print, If, Else, While, For, Fn, Return, Break, Continue,
    True, False, Nil, Eof,
}
#[derive(Clone, Debug)]
struct Token { kind: TokenKind, line: usize }
struct Lexer { chars: Vec<char>, current: usize, line: usize }
impl Lexer {
    fn new(source: &str) -> Self { Self { chars: source.chars().collect(), current: 0, line: 1 } }
    fn scan(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        while self.current < self.chars.len() {
            let start = self.current;
            let c = self.advance();
            let kind = match c {
                ' ' | '\r' | '\t' => continue,
                '\n' => { self.line += 1; continue; }
                '/' if self.take('/') => { while self.peek() != '\n' && !self.done() { self.advance(); } continue; }
                '(' => TokenKind::LeftParen, ')' => TokenKind::RightParen,
                '{' => TokenKind::LeftBrace, '}' => TokenKind::RightBrace,
                '[' => TokenKind::LeftBracket, ']' => TokenKind::RightBracket,
                ',' => TokenKind::Comma, ';' => TokenKind::Semicolon,
                '+' => TokenKind::Plus, '-' => TokenKind::Minus, '*' => TokenKind::Star,
                '/' => TokenKind::Slash, '%' => TokenKind::Percent,
                '!' => if self.take('=') { TokenKind::BangEqual } else { TokenKind::Bang },
                '=' => if self.take('=') { TokenKind::EqualEqual } else { TokenKind::Equal },
                '<' => if self.take('=') { TokenKind::LessEqual } else { TokenKind::Less },
                '>' => if self.take('=') { TokenKind::GreaterEqual } else { TokenKind::Greater },
                '&' if self.take('&') => TokenKind::And,
                '|' if self.take('|') => TokenKind::Or,
                '"' => {
                    let mut value = String::new();
                    let mut closed = false;
                    while !self.done() {
                        let ch = self.advance();
                        if ch == '"' { closed = true; break; }
                        if ch == '\n' { self.line += 1; }
                        if ch == '\\' {
                            if self.done() { break; }
                            let escaped = self.advance();
                            value.push(match escaped { 'n' => '\n', 't' => '\t', 'r' => '\r', '"' => '"', '\\' => '\\', other => other });
                        } else { value.push(ch); }
                    }
                    if !closed { return Err(format!("line {}: unterminated string", self.line)); }
                    TokenKind::String(value)
                }
                c if c.is_ascii_digit() => {
                    while self.peek().is_ascii_digit() { self.advance(); }
                    if self.peek() == '.' && self.peek_next().is_ascii_digit() { self.advance(); while self.peek().is_ascii_digit() { self.advance(); } }
                    let text: String = self.chars[start..self.current].iter().collect();
                    TokenKind::Number(text.parse().map_err(|_| format!("line {}: invalid number", self.line))?)
                }
                c if is_ident_start(c) => {
                    while is_ident_continue(self.peek()) { self.advance(); }
                    let text: String = self.chars[start..self.current].iter().collect();
                    match text.as_str() {
                        "let" => TokenKind::Let, "print" => TokenKind::Print, "if" => TokenKind::If,
                        "else" => TokenKind::Else, "while" => TokenKind::While, "for" => TokenKind::For,
                        "fn" => TokenKind::Fn, "return" => TokenKind::Return, "break" => TokenKind::Break,
                        "continue" => TokenKind::Continue, "true" => TokenKind::True, "false" => TokenKind::False,
                        "nil" => TokenKind::Nil, "and" => TokenKind::And, "or" => TokenKind::Or,
                        "not" => TokenKind::Bang, _ => TokenKind::Ident(text),
                    }
                }
                other => return Err(format!("line {}: unexpected character '{}'", self.line, other)),
            };
            tokens.push(Token { kind, line: self.line });
        }
        tokens.push(Token { kind: TokenKind::Eof, line: self.line });
        Ok(tokens)
    }
    fn done(&self) -> bool { self.current >= self.chars.len() }
    fn advance(&mut self) -> char { let c = self.chars[self.current]; self.current += 1; c }
    fn peek(&self) -> char { self.chars.get(self.current).copied().unwrap_or('\0') }
    fn peek_next(&self) -> char { self.chars.get(self.current + 1).copied().unwrap_or('\0') }
    fn take(&mut self, expected: char) -> bool { if self.peek() == expected { self.current += 1; true } else { false } }
}
fn is_ident_start(c: char) -> bool { c == '_' || c.is_alphabetic() }
fn is_ident_continue(c: char) -> bool { c == '_' || c.is_alphanumeric() }

#[derive(Clone, Debug)]
enum Expr {
    Literal(Value), Variable(String), Assign(String, Box<Expr>),
    Index(Box<Expr>, Box<Expr>), SetIndex(Box<Expr>, Box<Expr>, Box<Expr>),
    Unary(TokenKind, Box<Expr>), Binary(Box<Expr>, TokenKind, Box<Expr>),
    Call(String, Vec<Expr>), Array(Vec<Expr>),
}
#[derive(Clone, Debug)]
enum Stmt {
    Let(String, Expr), Print(Expr), Expression(Expr), Block(Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>), While(Expr, Box<Stmt>),
    For(Option<Box<Stmt>>, Option<Expr>, Option<Expr>, Box<Stmt>),
    Function(String, Vec<String>, Vec<Stmt>), Return(Option<Expr>), Break, Continue,
}
struct Parser { tokens: Vec<Token>, current: usize }
impl Parser {
    fn new(tokens: Vec<Token>) -> Self { Self { tokens, current: 0 } }
    fn parse(&mut self) -> Result<Vec<Stmt>, String> { let mut out = Vec::new(); while !self.end() { out.push(self.declaration()?); } Ok(out) }
    fn declaration(&mut self) -> Result<Stmt, String> {
        if self.matches(&TokenKind::Fn) { return self.function(); }
        if self.matches(&TokenKind::Let) { return self.let_statement(true); }
        self.statement()
    }
    fn let_statement(&mut self, semicolon: bool) -> Result<Stmt, String> {
        let name = self.ident("expected a variable name after 'let'")?;
        self.consume(&TokenKind::Equal, "expected '=' after variable name")?;
        let value = self.expression()?;
        if semicolon { self.consume(&TokenKind::Semicolon, "expected ';' after variable declaration")?; }
        Ok(Stmt::Let(name, value))
    }
    fn function(&mut self) -> Result<Stmt, String> {
        let name = self.ident("expected a function name")?;
        self.consume(&TokenKind::LeftParen, "expected '(' after function name")?;
        let mut params = Vec::new();
        if !self.check(&TokenKind::RightParen) { loop { if params.len() == 255 { return self.error("functions support at most 255 parameters"); } params.push(self.ident("expected parameter name")?); if !self.matches(&TokenKind::Comma) { break; } } }
        self.consume(&TokenKind::RightParen, "expected ')' after parameters")?;
        self.consume(&TokenKind::LeftBrace, "expected '{' before function body")?;
        Ok(Stmt::Function(name, params, self.block_contents()?))
    }
    fn statement(&mut self) -> Result<Stmt, String> {
        if self.matches(&TokenKind::Print) { let e = self.expression()?; self.consume(&TokenKind::Semicolon, "expected ';' after print")?; return Ok(Stmt::Print(e)); }
        if self.matches(&TokenKind::If) { let c = self.expression()?; let yes = Box::new(self.statement()?); let no = if self.matches(&TokenKind::Else) { Some(Box::new(self.statement()?)) } else { None }; return Ok(Stmt::If(c, yes, no)); }
        if self.matches(&TokenKind::While) { let c = self.expression()?; return Ok(Stmt::While(c, Box::new(self.statement()?))); }
        if self.matches(&TokenKind::For) { return self.for_statement(); }
        if self.matches(&TokenKind::Return) { let v = if self.check(&TokenKind::Semicolon) { None } else { Some(self.expression()?) }; self.consume(&TokenKind::Semicolon, "expected ';' after return")?; return Ok(Stmt::Return(v)); }
        if self.matches(&TokenKind::Break) { self.consume(&TokenKind::Semicolon, "expected ';' after break")?; return Ok(Stmt::Break); }
        if self.matches(&TokenKind::Continue) { self.consume(&TokenKind::Semicolon, "expected ';' after continue")?; return Ok(Stmt::Continue); }
        if self.matches(&TokenKind::LeftBrace) { return Ok(Stmt::Block(self.block_contents()?)); }
        let e = self.expression()?; self.consume(&TokenKind::Semicolon, "expected ';' after expression")?; Ok(Stmt::Expression(e))
    }
    fn for_statement(&mut self) -> Result<Stmt, String> {
        self.consume(&TokenKind::LeftParen, "expected '(' after 'for'")?;
        let init = if self.matches(&TokenKind::Semicolon) { None } else if self.matches(&TokenKind::Let) { Some(Box::new(self.let_statement(true)?)) } else { let e = self.expression()?; self.consume(&TokenKind::Semicolon, "expected ';' after loop initializer")?; Some(Box::new(Stmt::Expression(e))) };
        let condition = if self.check(&TokenKind::Semicolon) { None } else { Some(self.expression()?) };
        self.consume(&TokenKind::Semicolon, "expected ';' after loop condition")?;
        let increment = if self.check(&TokenKind::RightParen) { None } else { Some(self.expression()?) };
        self.consume(&TokenKind::RightParen, "expected ')' after for clauses")?;
        Ok(Stmt::For(init, condition, increment, Box::new(self.statement()?)))
    }
    fn block_contents(&mut self) -> Result<Vec<Stmt>, String> { let mut out = Vec::new(); while !self.check(&TokenKind::RightBrace) && !self.end() { out.push(self.declaration()?); } self.consume(&TokenKind::RightBrace, "expected '}' to close block")?; Ok(out) }
    fn expression(&mut self) -> Result<Expr, String> { self.assignment() }
    fn assignment(&mut self) -> Result<Expr, String> {
        let left = self.or()?;
        if self.matches(&TokenKind::Equal) { let value = self.assignment()?; return match left { Expr::Variable(n) => Ok(Expr::Assign(n, Box::new(value))), Expr::Index(a, i) => Ok(Expr::SetIndex(a, i, Box::new(value))), _ => self.error("invalid assignment target") }; }
        Ok(left)
    }
    fn or(&mut self) -> Result<Expr, String> { let mut e = self.and()?; while self.matches(&TokenKind::Or) { e = Expr::Binary(Box::new(e), TokenKind::Or, Box::new(self.and()?)); } Ok(e) }
    fn and(&mut self) -> Result<Expr, String> { let mut e = self.equality()?; while self.matches(&TokenKind::And) { e = Expr::Binary(Box::new(e), TokenKind::And, Box::new(self.equality()?)); } Ok(e) }
    fn equality(&mut self) -> Result<Expr, String> { let mut e = self.comparison()?; while self.matches_any(&[TokenKind::BangEqual, TokenKind::EqualEqual]) { let op = self.previous().kind.clone(); e = Expr::Binary(Box::new(e), op, Box::new(self.comparison()?)); } Ok(e) }
    fn comparison(&mut self) -> Result<Expr, String> { let mut e = self.term()?; while self.matches_any(&[TokenKind::Greater, TokenKind::GreaterEqual, TokenKind::Less, TokenKind::LessEqual]) { let op = self.previous().kind.clone(); e = Expr::Binary(Box::new(e), op, Box::new(self.term()?)); } Ok(e) }
    fn term(&mut self) -> Result<Expr, String> { let mut e = self.factor()?; while self.matches_any(&[TokenKind::Minus, TokenKind::Plus]) { let op = self.previous().kind.clone(); e = Expr::Binary(Box::new(e), op, Box::new(self.factor()?)); } Ok(e) }
    fn factor(&mut self) -> Result<Expr, String> { let mut e = self.unary()?; while self.matches_any(&[TokenKind::Slash, TokenKind::Star, TokenKind::Percent]) { let op = self.previous().kind.clone(); e = Expr::Binary(Box::new(e), op, Box::new(self.unary()?)); } Ok(e) }
    fn unary(&mut self) -> Result<Expr, String> { if self.matches_any(&[TokenKind::Bang, TokenKind::Minus]) { let op = self.previous().kind.clone(); return Ok(Expr::Unary(op, Box::new(self.unary()?))); } self.call() }
    fn call(&mut self) -> Result<Expr, String> {
        let mut e = self.primary()?;
        loop {
            if self.matches(&TokenKind::LeftParen) {
                let name = match e { Expr::Variable(n) => n, _ => return self.error("only named functions can be called") };
                let mut args = Vec::new(); if !self.check(&TokenKind::RightParen) { loop { if args.len() == 255 { return self.error("calls support at most 255 arguments"); } args.push(self.expression()?); if !self.matches(&TokenKind::Comma) { break; } } }
                self.consume(&TokenKind::RightParen, "expected ')' after arguments")?; e = Expr::Call(name, args);
            } else if self.matches(&TokenKind::LeftBracket) { let index = self.expression()?; self.consume(&TokenKind::RightBracket, "expected ']' after index")?; e = Expr::Index(Box::new(e), Box::new(index)); }
            else { break; }
        }
        Ok(e)
    }
    fn primary(&mut self) -> Result<Expr, String> {
        let t = self.advance().clone(); match t.kind {
            TokenKind::False => Ok(Expr::Literal(Value::Bool(false))), TokenKind::True => Ok(Expr::Literal(Value::Bool(true))), TokenKind::Nil => Ok(Expr::Literal(Value::Nil)),
            TokenKind::Number(n) => Ok(Expr::Literal(Value::Number(n))), TokenKind::String(s) => Ok(Expr::Literal(Value::String(s))), TokenKind::Ident(n) => Ok(Expr::Variable(n)),
            TokenKind::LeftParen => { let e = self.expression()?; self.consume(&TokenKind::RightParen, "expected ')' after expression")?; Ok(e) }
            TokenKind::LeftBracket => { let mut values = Vec::new(); if !self.check(&TokenKind::RightBracket) { loop { values.push(self.expression()?); if !self.matches(&TokenKind::Comma) { break; } if self.check(&TokenKind::RightBracket) { break; } } } self.consume(&TokenKind::RightBracket, "expected ']' after array")?; Ok(Expr::Array(values)) }
            _ => Err(format!("line {}: expected a value or expression", t.line)),
        }
    }
    fn matches(&mut self, k: &TokenKind) -> bool { if self.check(k) { self.advance(); true } else { false } }
    fn matches_any(&mut self, ks: &[TokenKind]) -> bool { for k in ks { if self.check(k) { self.advance(); return true; } } false }
    fn check(&self, k: &TokenKind) -> bool { if self.end() { *k == TokenKind::Eof } else { self.peek().kind == *k } }
    fn advance(&mut self) -> &Token { if !self.end() { self.current += 1; } self.previous() }
    fn end(&self) -> bool { self.peek().kind == TokenKind::Eof }
    fn peek(&self) -> &Token { &self.tokens[self.current] }
    fn previous(&self) -> &Token { &self.tokens[self.current - 1] }
    fn consume(&mut self, k: &TokenKind, msg: &str) -> Result<(), String> { if self.check(k) { self.advance(); Ok(()) } else { self.error(msg) } }
    fn ident(&mut self, msg: &str) -> Result<String, String> { if let TokenKind::Ident(n) = self.peek().kind.clone() { self.advance(); Ok(n) } else { self.error(msg) } }
    fn error<T>(&self, msg: &str) -> Result<T, String> { Err(format!("line {}: {}", self.peek().line, msg)) }
}

#[derive(Clone, Debug)]
enum Value { Number(f64), String(String), Bool(bool), Nil, Array(Rc<RefCell<Vec<Value>>>) }
impl Value {
    fn display(&self) -> String { match self { Value::Number(n) if n.fract() == 0.0 && n.is_finite() => format!("{:.0}", n), Value::Number(n) => n.to_string(), Value::String(s) => s.clone(), Value::Bool(b) => b.to_string(), Value::Nil => "nil".into(), Value::Array(a) => format!("[{}]", a.borrow().iter().map(Value::display).collect::<Vec<_>>().join(", ")) } }
    fn truthy(&self) -> bool { !matches!(self, Value::Nil | Value::Bool(false)) }
    fn type_name(&self) -> &'static str { match self { Value::Number(_) => "number", Value::String(_) => "string", Value::Bool(_) => "boolean", Value::Nil => "nil", Value::Array(_) => "array" } }
}
impl PartialEq for Value { fn eq(&self, other: &Self) -> bool { match (self, other) { (Value::Number(a), Value::Number(b)) => a == b, (Value::String(a), Value::String(b)) => a == b, (Value::Bool(a), Value::Bool(b)) => a == b, (Value::Nil, Value::Nil) => true, (Value::Array(a), Value::Array(b)) => *a.borrow() == *b.borrow(), _ => false } } }
#[derive(Clone)]
struct Function { params: Vec<String>, body: Vec<Stmt> }
enum Flow { Continue, Break, Return(Value) }
struct Interpreter { scopes: Vec<HashMap<String, Value>>, functions: HashMap<String, Function> }
impl Interpreter {
    fn new() -> Self { Self { scopes: vec![HashMap::new()], functions: HashMap::new() } }
    fn run(&mut self, statements: &[Stmt]) -> Result<(), String> { for s in statements { match self.execute(s)? { Flow::Return(_) => return Err("return used outside a function".into()), Flow::Break => return Err("break used outside a loop".into()), Flow::Continue => {} } } Ok(()) }
    fn execute(&mut self, s: &Stmt) -> Result<Flow, String> { match s {
        Stmt::Let(n, e) => { let v = self.eval(e)?; self.scopes.last_mut().unwrap().insert(n.clone(), v); }
        Stmt::Print(e) => println!("{}", self.eval(e)?.display()), Stmt::Expression(e) => { self.eval(e)?; }
        Stmt::Block(body) => { self.scopes.push(HashMap::new()); let result = self.execute_all(body); self.scopes.pop(); return result; }
        Stmt::If(c, yes, no) => { if self.eval(c)?.truthy() { return self.execute(yes); } else if let Some(n) = no { return self.execute(n); } }
        Stmt::While(c, body) => { while self.eval(c)?.truthy() { match self.execute(body)? { Flow::Return(v) => return Ok(Flow::Return(v)), Flow::Break => break, Flow::Continue => {} } } }
        Stmt::For(init, cond, inc, body) => { self.scopes.push(HashMap::new()); if let Some(i) = init { self.execute(i)?; } loop { if let Some(c) = cond { if !self.eval(c)?.truthy() { break; } } match self.execute(body)? { Flow::Return(v) => { self.scopes.pop(); return Ok(Flow::Return(v)); }, Flow::Break => break, Flow::Continue => {} } if let Some(i) = inc { self.eval(i)?; } } self.scopes.pop(); }
        Stmt::Function(n, p, b) => { self.functions.insert(n.clone(), Function { params: p.clone(), body: b.clone() }); }
        Stmt::Return(e) => return Ok(Flow::Return(if let Some(e) = e { self.eval(e)? } else { Value::Nil })),
        Stmt::Break => return Ok(Flow::Break), Stmt::Continue => return Ok(Flow::Continue),
    } Ok(Flow::Continue) }
    fn execute_all(&mut self, ss: &[Stmt]) -> Result<Flow, String> { for s in ss { match self.execute(s)? { Flow::Continue => {}, other => return Ok(other) } } Ok(Flow::Continue) }
    fn eval(&mut self, e: &Expr) -> Result<Value, String> { match e {
        Expr::Literal(v) => Ok(v.clone()), Expr::Variable(n) => self.lookup(n).ok_or_else(|| format!("undefined variable '{n}'")),
        Expr::Assign(n, e) => { let v = self.eval(e)?; for scope in self.scopes.iter_mut().rev() { if scope.contains_key(n) { scope.insert(n.clone(), v.clone()); return Ok(v); } } Err(format!("undefined variable '{n}'")) }
        Expr::Array(items) => Ok(Value::Array(Rc::new(RefCell::new(items.iter().map(|e| self.eval(e)).collect::<Result<Vec<_>, _>>()?)))),
        Expr::Index(a, i) => { let array = self.eval(a)?; let index = self.index_value(self.eval(i)?)?; match array { Value::Array(v) => v.borrow().get(index).cloned().ok_or_else(|| format!("array index {index} is out of bounds")), Value::String(s) => s.chars().nth(index).map(|c| Value::String(c.to_string())).ok_or_else(|| format!("string index {index} is out of bounds")), _ => Err("indexing requires an array or string".into()) } }
        Expr::SetIndex(a, i, value) => { let array = self.eval(a)?; let index = self.index_value(self.eval(i)?)?; let value = self.eval(value)?; if let Value::Array(items) = array { let mut items = items.borrow_mut(); if index >= items.len() { return Err(format!("array index {index} is out of bounds")); } items[index] = value.clone(); Ok(value) } else { Err("only array elements can be assigned".into()) } }
        Expr::Unary(op, right) => { let v = self.eval(right)?; match op { TokenKind::Bang => Ok(Value::Bool(!v.truthy())), TokenKind::Minus => if let Value::Number(n) = v { Ok(Value::Number(-n)) } else { Err("unary '-' requires a number".into()) }, _ => unreachable!() } }
        Expr::Binary(left, op, right) => { let a = self.eval(left)?; if *op == TokenKind::And && !a.truthy() { return Ok(Value::Bool(false)); } if *op == TokenKind::Or && a.truthy() { return Ok(Value::Bool(true)); } let b = self.eval(right)?; self.binary(a, op, b) }
        Expr::Call(name, args) => { let values = args.iter().map(|a| self.eval(a)).collect::<Result<Vec<_>, _>>()?; if is_builtin(name) { return self.builtin(name, values); } let f = self.functions.get(name).cloned().ok_or_else(|| format!("undefined function '{name}'"))?; if values.len() != f.params.len() { return Err(format!("'{name}' expects {} arguments, got {}", f.params.len(), values.len())); } self.scopes.push(HashMap::new()); for (p, v) in f.params.iter().zip(values) { self.scopes.last_mut().unwrap().insert(p.clone(), v); } let result = self.execute_all(&f.body); self.scopes.pop(); match result? { Flow::Continue => Ok(Value::Nil), Flow::Return(v) => Ok(v), Flow::Break => Err("break used outside a loop".into()) } }
    } }
    fn lookup(&self, n: &str) -> Option<Value> { self.scopes.iter().rev().find_map(|s| s.get(n).cloned()) }
    fn index_value(&self, v: Value) -> Result<usize, String> { if let Value::Number(n) = v { if n >= 0.0 && n.fract() == 0.0 && n <= usize::MAX as f64 { Ok(n as usize) } else { Err("array index must be a non-negative integer".into()) } } else { Err("array index must be a number".into()) } }
    fn builtin(&self, name: &str, a: Vec<Value>) -> Result<Value, String> {
        let arity = match name { "len" | "type" | "str" | "number" | "input" | "assert" => 1, "push" => 2, "pop" => 1, _ => 0 };
        if a.len() != arity { return Err(format!("'{name}' expects {arity} argument(s), got {}", a.len())); }
        match name {
            "len" => match &a[0] { Value::Array(v) => Ok(Value::Number(v.borrow().len() as f64)), Value::String(s) => Ok(Value::Number(s.chars().count() as f64)), _ => Err("len() expects an array or string".into()) },
            "type" => Ok(Value::String(a[0].type_name().into())), "str" => Ok(Value::String(a[0].display())),
            "number" => match &a[0] { Value::Number(n) => Ok(Value::Number(*n)), Value::Bool(b) => Ok(Value::Number(if *b { 1.0 } else { 0.0 })), Value::String(s) => s.trim().parse::<f64>().map(Value::Number).map_err(|_| "number() could not convert the string".into()), _ => Err("number() expects a number, boolean, or numeric string".into()) },
            "push" => if let Value::Array(v) = &a[0] { v.borrow_mut().push(a[1].clone()); Ok(Value::Array(v.clone())) } else { Err("push() expects an array as its first argument".into()) },
            "pop" => if let Value::Array(v) = &a[0] { v.borrow_mut().pop().ok_or_else(|| "pop() cannot remove an item from an empty array".into()) } else { Err("pop() expects an array".into()) },
            "input" => { print!("{}", a[0].display()); io::stdout().flush().map_err(|e| e.to_string())?; let mut line = String::new(); io::stdin().read_line(&mut line).map_err(|e| e.to_string())?; if line.ends_with('\n') { line.pop(); if line.ends_with('\r') { line.pop(); } } Ok(Value::String(line)) },
            "assert" => if a[0].truthy() { Ok(Value::Nil) } else { Err("assertion failed".into()) },
            _ => unreachable!(),
        }
    }
    fn binary(&self, a: Value, op: &TokenKind, b: Value) -> Result<Value, String> { use TokenKind::*; match op {
        Plus => match (a, b) { (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x + y)), (Value::String(x), Value::String(y)) => Ok(Value::String(x + &y)), (Value::Array(x), Value::Array(y)) => { let mut out = x.borrow().clone(); out.extend(y.borrow().iter().cloned()); Ok(Value::Array(Rc::new(RefCell::new(out)))) }, _ => Err("'+' requires two numbers, two strings, or two arrays".into()) },
        Minus | Star | Slash | Percent | Greater | GreaterEqual | Less | LessEqual => { let (x, y) = match (a, b) { (Value::Number(x), Value::Number(y)) => (x, y), _ => return Err("this operator requires two numbers".into()) }; match op { Minus => Ok(Value::Number(x-y)), Star => Ok(Value::Number(x*y)), Slash if y == 0.0 => Err("division by zero".into()), Slash => Ok(Value::Number(x/y)), Percent if y == 0.0 => Err("remainder by zero".into()), Percent => Ok(Value::Number(x%y)), Greater => Ok(Value::Bool(x>y)), GreaterEqual => Ok(Value::Bool(x>=y)), Less => Ok(Value::Bool(x<y)), LessEqual => Ok(Value::Bool(x<=y)), _ => unreachable!() } },
        EqualEqual => Ok(Value::Bool(a == b)), BangEqual => Ok(Value::Bool(a != b)), And => Ok(Value::Bool(a.truthy() && b.truthy())), Or => Ok(Value::Bool(a.truthy() || b.truthy())), _ => unreachable!(),
    } }
}
fn is_builtin(n: &str) -> bool { matches!(n, "len" | "type" | "str" | "number" | "input" | "push" | "pop" | "assert") }
fn execute_source(source: &str, vm: &mut Interpreter) -> Result<(), String> { let tokens = Lexer::new(source).scan()?; let statements = Parser::new(tokens).parse()?; vm.run(&statements) }
fn usage() { println!("RUST++ v1.0.0\nA small, friendly interpreted programming language.\n\nUsage: rustpp [--version | --help | FILE.rustqq]\nRun without a file to start the interactive shell."); }
fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.iter().any(|a| a == "--version" || a == "-V") { println!("RUST++ v1.0.0"); return; }
    if args.iter().any(|a| a == "--help" || a == "-h") { usage(); return; }
    let mut vm = Interpreter::new();
    if let Some(path) = args.first() { if args.len() > 1 { eprintln!("error: only one source file may be provided"); std::process::exit(2); } match fs::read_to_string(path) { Ok(src) => if let Err(e) = execute_source(&src, &mut vm) { eprintln!("error: {e}"); std::process::exit(1); }, Err(e) => { eprintln!("error: cannot read '{path}': {e}"); std::process::exit(1); } } return; }
    println!("RUST++ v1.0.0 — type 'exit' to quit"); let stdin = io::stdin();
    loop { print!("rustqq> "); if io::stdout().flush().is_err() { break; } let mut line = String::new(); match stdin.read_line(&mut line) { Ok(0) => break, Ok(_) => { if line.trim() == "exit" { break; } if let Err(e) = execute_source(&line, &mut vm) { eprintln!("error: {e}"); } }, Err(e) => { eprintln!("error: {e}"); break; } } }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn lexer_recognizes_v1_keywords_and_arrays() { let t = Lexer::new("for (let x = 0; x < 2; x = x + 1) { print [x, true]; }").scan().unwrap(); assert!(t.iter().any(|t| t.kind == TokenKind::For)); assert!(t.iter().any(|t| t.kind == TokenKind::LeftBracket)); }
    #[test] fn parser_accepts_functions_loops_and_arrays() { let tokens = Lexer::new("fn sum(a) { let x = [a]; for (let i = 0; i < 2; i = i + 1) { push(x, i); } return x[1]; }").scan().unwrap(); assert!(Parser::new(tokens).parse().is_ok()); }
    #[test] fn values_compare_arrays_by_content() { let a = Value::Array(Rc::new(RefCell::new(vec![Value::Number(1.0)]))); let b = Value::Array(Rc::new(RefCell::new(vec![Value::Number(1.0)]))); assert_eq!(a, b); }
}
