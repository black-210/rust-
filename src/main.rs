use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};

#[derive(Clone, Debug, PartialEq)]
enum TokenKind {
    Number(f64), String(String), Ident(String),
    Plus, Minus, Star, Slash, Percent,
    LeftParen, RightParen, LeftBrace, RightBrace, Comma, Semicolon,
    Equal, EqualEqual, Bang, BangEqual, Less, LessEqual, Greater, GreaterEqual,
    And, Or,
    Let, Print, If, Else, While, Fn, Return, True, False, Nil,
    Eof,
}

#[derive(Clone, Debug)]
struct Token { kind: TokenKind, line: usize }

struct Lexer { chars: Vec<char>, start: usize, current: usize, line: usize }
impl Lexer {
    fn new(source: &str) -> Self { Self { chars: source.chars().collect(), start: 0, current: 0, line: 1 } }
    fn scan(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        while !self.done() {
            self.start = self.current;
            if let Some(token) = self.next_token()? { tokens.push(token); }
        }
        tokens.push(Token { kind: TokenKind::Eof, line: self.line });
        Ok(tokens)
    }
    fn done(&self) -> bool { self.current >= self.chars.len() }
    fn advance(&mut self) -> char { let c = self.chars[self.current]; self.current += 1; c }
    fn peek(&self) -> char { self.chars.get(self.current).copied().unwrap_or('\0') }
    fn peek_next(&self) -> char { self.chars.get(self.current + 1).copied().unwrap_or('\0') }
    fn take(&mut self, expected: char) -> bool {
        if self.peek() == expected { self.current += 1; true } else { false }
    }
    fn token(&self, kind: TokenKind) -> Token { Token { kind, line: self.line } }
    fn next_token(&mut self) -> Result<Option<Token>, String> {
        let c = self.advance();
        let token = match c {
            ' ' | '\r' | '\t' => return Ok(None),
            '\n' => { self.line += 1; return Ok(None); }
            '(' => TokenKind::LeftParen, ')' => TokenKind::RightParen,
            '{' => TokenKind::LeftBrace, '}' => TokenKind::RightBrace,
            ',' => TokenKind::Comma, ';' => TokenKind::Semicolon,
            '+' => TokenKind::Plus, '-' => TokenKind::Minus, '*' => TokenKind::Star,
            '%' => TokenKind::Percent,
            '/' if self.take('/') => { while self.peek() != '\n' && !self.done() { self.advance(); } return Ok(None); }
            '/' => TokenKind::Slash,
            '!' => if self.take('=') { TokenKind::BangEqual } else { TokenKind::Bang },
            '=' => if self.take('=') { TokenKind::EqualEqual } else { TokenKind::Equal },
            '<' => if self.take('=') { TokenKind::LessEqual } else { TokenKind::Less },
            '>' => if self.take('=') { TokenKind::GreaterEqual } else { TokenKind::Greater },
            '&' if self.take('&') => TokenKind::And,
            '|' if self.take('|') => TokenKind::Or,
            '"' => return self.string_token(),
            c if c.is_ascii_digit() => return self.number_token(),
            c if is_ident_start(c) => return Ok(Some(self.identifier_token())),
            _ => return Err(format!("السطر {}: رمز غير معروف '{}'", self.line, c)),
        };
        Ok(Some(self.token(token)))
    }
    fn string_token(&mut self) -> Result<Option<Token>, String> {
        let mut value = String::new();
        while self.peek() != '"' && !self.done() {
            let c = self.advance();
            if c == '\n' { self.line += 1; }
            if c == '\\' {
                let escaped = self.advance();
                value.push(match escaped { 'n' => '\n', 't' => '\t', '"' => '"', '\\' => '\\', other => other });
            } else { value.push(c); }
        }
        if self.done() { return Err(format!("السطر {}: نص غير مغلق", self.line)); }
        self.advance();
        Ok(Some(self.token(TokenKind::String(value))))
    }
    fn number_token(&mut self) -> Result<Option<Token>, String> {
        while self.peek().is_ascii_digit() { self.advance(); }
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() { self.advance(); }
        }
        let text: String = self.chars[self.start..self.current].iter().collect();
        let n = text.parse::<f64>().map_err(|_| format!("السطر {}: رقم غير صالح", self.line))?;
        Ok(Some(self.token(TokenKind::Number(n))))
    }
    fn identifier_token(&mut self) -> Token {
        while is_ident_continue(self.peek()) { self.advance(); }
        let text: String = self.chars[self.start..self.current].iter().collect();
        let kind = match text.as_str() {
            "let" => TokenKind::Let, "print" => TokenKind::Print,
            "if" => TokenKind::If, "else" => TokenKind::Else,
            "while" => TokenKind::While, "fn" => TokenKind::Fn,
            "return" => TokenKind::Return, "true" => TokenKind::True,
            "false" => TokenKind::False, "nil" => TokenKind::Nil,
            _ => TokenKind::Ident(text),
        };
        self.token(kind)
    }
}
fn is_ident_start(c: char) -> bool { c == '_' || c.is_alphabetic() }
fn is_ident_continue(c: char) -> bool { c == '_' || c.is_alphanumeric() }

#[derive(Clone, Debug)]
enum Expr {
    Literal(Value), Variable(String), Assign(String, Box<Expr>),
    Unary(TokenKind, Box<Expr>), Binary(Box<Expr>, TokenKind, Box<Expr>),
    Call(String, Vec<Expr>),
}
#[derive(Clone, Debug)]
enum Stmt {
    Let(String, Expr), Print(Expr), Expression(Expr), Block(Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>), While(Expr, Box<Stmt>),
    Function(String, Vec<String>, Vec<Stmt>), Return(Option<Expr>),
}
struct Parser { tokens: Vec<Token>, current: usize }
impl Parser {
    fn new(tokens: Vec<Token>) -> Self { Self { tokens, current: 0 } }
    fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = Vec::new();
        while !self.is_at_end() { statements.push(self.declaration()?); }
        Ok(statements)
    }
    fn declaration(&mut self) -> Result<Stmt, String> {
        if self.matches(&TokenKind::Fn) { return self.function(); }
        if self.matches(&TokenKind::Let) {
            let name = self.consume_ident("توقع اسم المتغير بعد let")?;
            self.consume(&TokenKind::Equal, "توقع '=' بعد اسم المتغير")?;
            let value = self.expression()?;
            self.consume(&TokenKind::Semicolon, "أضف ; بعد تعريف المتغير")?;
            return Ok(Stmt::Let(name, value));
        }
        self.statement()
    }
    fn function(&mut self) -> Result<Stmt, String> {
        let name = self.consume_ident("توقع اسم الدالة بعد fn")?;
        self.consume(&TokenKind::LeftParen, "توقع ( بعد اسم الدالة")?;
        let mut params = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
                if params.len() >= 255 { return self.error("لا يمكن أن تتجاوز الدالة 255 وسيطاً"); }
                params.push(self.consume_ident("توقع اسم وسيط")?);
                if !self.matches(&TokenKind::Comma) { break; }
            }
        }
        self.consume(&TokenKind::RightParen, "توقع ) بعد الوسائط")?;
        self.consume(&TokenKind::LeftBrace, "توقع { قبل جسم الدالة")?;
        let body = self.block_contents()?;
        Ok(Stmt::Function(name, params, body))
    }
    fn statement(&mut self) -> Result<Stmt, String> {
        if self.matches(&TokenKind::Print) {
            let value = self.expression()?;
            self.consume(&TokenKind::Semicolon, "أضف ; بعد print")?;
            return Ok(Stmt::Print(value));
        }
        if self.matches(&TokenKind::If) {
            let condition = self.expression()?;
            let then_branch = Box::new(self.statement()?);
            let else_branch = if self.matches(&TokenKind::Else) { Some(Box::new(self.statement()?)) } else { None };
            return Ok(Stmt::If(condition, then_branch, else_branch));
        }
        if self.matches(&TokenKind::While) {
            let condition = self.expression()?;
            return Ok(Stmt::While(condition, Box::new(self.statement()?)));
        }
        if self.matches(&TokenKind::Return) {
            let value = if self.check(&TokenKind::Semicolon) { None } else { Some(self.expression()?) };
            self.consume(&TokenKind::Semicolon, "أضف ; بعد return")?;
            return Ok(Stmt::Return(value));
        }
        if self.matches(&TokenKind::LeftBrace) { return Ok(Stmt::Block(self.block_contents()?)); }
        let expr = self.expression()?;
        self.consume(&TokenKind::Semicolon, "أضف ; بعد التعليمة")?;
        Ok(Stmt::Expression(expr))
    }
    fn block_contents(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() { statements.push(self.declaration()?); }
        self.consume(&TokenKind::RightBrace, "توقع } لإغلاق الكتلة")?;
        Ok(statements)
    }
    fn expression(&mut self) -> Result<Expr, String> { self.assignment() }
    fn assignment(&mut self) -> Result<Expr, String> {
        let expr = self.or()?;
        if self.matches(&TokenKind::Equal) {
            let value = self.assignment()?;
            if let Expr::Variable(name) = expr { return Ok(Expr::Assign(name, Box::new(value))); }
            return self.error("يمكن إسناد قيمة إلى متغير فقط");
        }
        Ok(expr)
    }
    fn or(&mut self) -> Result<Expr, String> {
        let mut expr = self.and()?;
        while self.matches(&TokenKind::Or) { let right = self.and()?; expr = Expr::Binary(Box::new(expr), TokenKind::Or, Box::new(right)); }
        Ok(expr)
    }
    fn and(&mut self) -> Result<Expr, String> {
        let mut expr = self.equality()?;
        while self.matches(&TokenKind::And) { let right = self.equality()?; expr = Expr::Binary(Box::new(expr), TokenKind::And, Box::new(right)); }
        Ok(expr)
    }
    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.comparison()?;
        while self.matches_any(&[TokenKind::BangEqual, TokenKind::EqualEqual]) { let op = self.previous().kind.clone(); let right = self.comparison()?; expr = Expr::Binary(Box::new(expr), op, Box::new(right)); }
        Ok(expr)
    }
    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.term()?;
        while self.matches_any(&[TokenKind::Greater, TokenKind::GreaterEqual, TokenKind::Less, TokenKind::LessEqual]) { let op = self.previous().kind.clone(); let right = self.term()?; expr = Expr::Binary(Box::new(expr), op, Box::new(right)); }
        Ok(expr)
    }
    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;
        while self.matches_any(&[TokenKind::Minus, TokenKind::Plus]) { let op = self.previous().kind.clone(); let right = self.factor()?; expr = Expr::Binary(Box::new(expr), op, Box::new(right)); }
        Ok(expr)
    }
    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.unary()?;
        while self.matches_any(&[TokenKind::Slash, TokenKind::Star, TokenKind::Percent]) { let op = self.previous().kind.clone(); let right = self.unary()?; expr = Expr::Binary(Box::new(expr), op, Box::new(right)); }
        Ok(expr)
    }
    fn unary(&mut self) -> Result<Expr, String> {
        if self.matches_any(&[TokenKind::Bang, TokenKind::Minus]) { let op = self.previous().kind.clone(); return Ok(Expr::Unary(op, Box::new(self.unary()?))); }
        self.call()
    }
    fn call(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;
        loop {
            if self.matches(&TokenKind::LeftParen) {
                let name = if let Expr::Variable(name) = expr { name } else { return self.error("استدعاء الدوال يكون باسم الدالة مباشرة"); };
                let mut args = Vec::new();
                if !self.check(&TokenKind::RightParen) {
                    loop { if args.len() >= 255 { return self.error("لا يمكن أن يتجاوز الاستدعاء 255 وسيطاً"); } args.push(self.expression()?); if !self.matches(&TokenKind::Comma) { break; } }
                }
                self.consume(&TokenKind::RightParen, "توقع ) بعد الوسائط")?;
                expr = Expr::Call(name, args);
            } else { break; }
        }
        Ok(expr)
    }
    fn primary(&mut self) -> Result<Expr, String> {
        let kind = self.advance().kind.clone();
        match kind {
            TokenKind::False => Ok(Expr::Literal(Value::Bool(false))),
            TokenKind::True => Ok(Expr::Literal(Value::Bool(true))),
            TokenKind::Nil => Ok(Expr::Literal(Value::Nil)),
            TokenKind::Number(n) => Ok(Expr::Literal(Value::Number(n))),
            TokenKind::String(s) => Ok(Expr::Literal(Value::String(s))),
            TokenKind::Ident(name) => Ok(Expr::Variable(name)),
            TokenKind::LeftParen => { let expr = self.expression()?; self.consume(&TokenKind::RightParen, "توقع ) بعد التعبير")?; Ok(expr) }
            _ => self.error("توقع قيمة أو تعبيراً"),
        }
    }
    fn matches(&mut self, kind: &TokenKind) -> bool { if self.check(kind) { self.advance(); true } else { false } }
    fn matches_any(&mut self, kinds: &[TokenKind]) -> bool { for kind in kinds { if self.check(kind) { self.advance(); return true; } } false }
    fn check(&self, kind: &TokenKind) -> bool { if self.is_at_end() { *kind == TokenKind::Eof } else { self.peek().kind == *kind } }
    fn advance(&mut self) -> &Token { if !self.is_at_end() { self.current += 1; } self.previous() }
    fn is_at_end(&self) -> bool { self.peek().kind == TokenKind::Eof }
    fn peek(&self) -> &Token { &self.tokens[self.current] }
    fn previous(&self) -> &Token { &self.tokens[self.current - 1] }
    fn consume(&mut self, kind: &TokenKind, message: &str) -> Result<(), String> { if self.check(kind) { self.advance(); Ok(()) } else { self.error(message) } }
    fn consume_ident(&mut self, message: &str) -> Result<String, String> { if let TokenKind::Ident(name) = self.peek().kind.clone() { self.advance(); Ok(name) } else { self.error(message) } }
    fn error<T>(&self, message: &str) -> Result<T, String> { Err(format!("السطر {}: {}", self.peek().line, message)) }
}

#[derive(Clone, Debug, PartialEq)]
enum Value { Number(f64), String(String), Bool(bool), Nil }
impl Value {
    fn display(&self) -> String { match self { Value::Number(n) if n.fract() == 0.0 => format!("{}", *n as i64), Value::Number(n) => n.to_string(), Value::String(s) => s.clone(), Value::Bool(b) => b.to_string(), Value::Nil => "nil".into() } }
    fn truthy(&self) -> bool { !matches!(self, Value::Nil | Value::Bool(false)) }
}
#[derive(Clone)]
struct Function { params: Vec<String>, body: Vec<Stmt> }
enum Flow { Continue, Return(Value) }
struct Interpreter { scopes: Vec<HashMap<String, Value>>, functions: HashMap<String, Function> }
impl Interpreter {
    fn new() -> Self { Self { scopes: vec![HashMap::new()], functions: HashMap::new() } }
    fn run(&mut self, statements: &[Stmt]) -> Result<(), String> {
        for stmt in statements { if let Flow::Return(_) = self.execute(stmt)? { return Err("return لا يعمل خارج الدالة".into()); } }
        Ok(())
    }
    fn execute(&mut self, stmt: &Stmt) -> Result<Flow, String> {
        match stmt {
            Stmt::Let(name, expr) => { let value = self.evaluate(expr)?; self.scopes.last_mut().unwrap().insert(name.clone(), value); }
            Stmt::Print(expr) => println!("{}", self.evaluate(expr)?.display()),
            Stmt::Expression(expr) => { self.evaluate(expr)?; }
            Stmt::Block(statements) => { self.scopes.push(HashMap::new()); let result = self.execute_all(statements); self.scopes.pop(); return result; }
            Stmt::If(condition, yes, no) => { if self.evaluate(condition)?.truthy() { return self.execute(yes); } else if let Some(branch) = no { return self.execute(branch); } }
            Stmt::While(condition, body) => { while self.evaluate(condition)?.truthy() { if let Flow::Return(value) = self.execute(body)? { return Ok(Flow::Return(value)); } } }
            Stmt::Function(name, params, body) => { self.functions.insert(name.clone(), Function { params: params.clone(), body: body.clone() }); }
            Stmt::Return(expr) => { let value = if let Some(expr) = expr { self.evaluate(expr)? } else { Value::Nil }; return Ok(Flow::Return(value)); }
        }
        Ok(Flow::Continue)
    }
    fn execute_all(&mut self, statements: &[Stmt]) -> Result<Flow, String> { for stmt in statements { if let Flow::Return(value) = self.execute(stmt)? { return Ok(Flow::Return(value)); } } Ok(Flow::Continue) }
    fn evaluate(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Literal(value) => Ok(value.clone()),
            Expr::Variable(name) => self.lookup(name).ok_or_else(|| format!("متغير غير معرّف: {}", name)),
            Expr::Assign(name, value) => { let value = self.evaluate(value)?; for scope in self.scopes.iter_mut().rev() { if scope.contains_key(name) { scope.insert(name.clone(), value.clone()); return Ok(value); } } Err(format!("متغير غير معرّف: {}", name)) }
            Expr::Unary(op, right) => { let value = self.evaluate(right)?; match op { TokenKind::Bang => Ok(Value::Bool(!value.truthy())), TokenKind::Minus => match value { Value::Number(n) => Ok(Value::Number(-n)), _ => Err("العامل - يحتاج إلى رقم".into()) }, _ => unreachable!() } }
            Expr::Binary(left, op, right) => {
                let a = self.evaluate(left)?;
                if *op == TokenKind::And && !a.truthy() { return Ok(Value::Bool(false)); }
                if *op == TokenKind::Or && a.truthy() { return Ok(Value::Bool(true)); }
                let b = self.evaluate(right)?;
                self.binary(a, op, b)
            }
            Expr::Call(name, args) => {
                let function = self.functions.get(name).cloned().ok_or_else(|| format!("دالة غير معرّفة: {}", name))?;
                if args.len() != function.params.len() { return Err(format!("{} تستقبل {} وسيطاً، وليس {}", name, function.params.len(), args.len())); }
                let values = args.iter().map(|arg| self.evaluate(arg)).collect::<Result<Vec<_>, _>>()?;
                self.scopes.push(HashMap::new());
                for (param, value) in function.params.iter().zip(values) { self.scopes.last_mut().unwrap().insert(param.clone(), value); }
                let result = self.execute_all(&function.body);
                self.scopes.pop();
                match result? { Flow::Continue => Ok(Value::Nil), Flow::Return(value) => Ok(value) }
            }
        }
    }
    fn lookup(&self, name: &str) -> Option<Value> { self.scopes.iter().rev().find_map(|scope| scope.get(name).cloned()) }
    fn binary(&self, a: Value, op: &TokenKind, b: Value) -> Result<Value, String> {
        use TokenKind::*;
        match op {
            Plus => match (a, b) { (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x + y)), (Value::String(x), Value::String(y)) => Ok(Value::String(x + &y)), _ => Err("+ يعمل مع رقمين أو نصين".into()) },
            Minus | Star | Slash | Percent | Greater | GreaterEqual | Less | LessEqual => {
                let (x, y) = match (a, b) { (Value::Number(x), Value::Number(y)) => (x, y), _ => return Err("هذا العامل يحتاج إلى رقمين".into()) };
                match op { Minus => Ok(Value::Number(x-y)), Star => Ok(Value::Number(x*y)), Slash if y == 0.0 => Err("لا يمكن القسمة على صفر".into()), Slash => Ok(Value::Number(x/y)), Percent if y == 0.0 => Err("لا يمكن باقي القسمة على صفر".into()), Percent => Ok(Value::Number(x%y)), Greater => Ok(Value::Bool(x>y)), GreaterEqual => Ok(Value::Bool(x>=y)), Less => Ok(Value::Bool(x<y)), LessEqual => Ok(Value::Bool(x<=y)), _ => unreachable!() }
            }
            EqualEqual => Ok(Value::Bool(a == b)), BangEqual => Ok(Value::Bool(a != b)),
            And => Ok(Value::Bool(a.truthy() && b.truthy())), Or => Ok(Value::Bool(a.truthy() || b.truthy())),
            _ => unreachable!(),
        }
    }
}

fn execute_source(source: &str, interpreter: &mut Interpreter) -> Result<(), String> {
    let tokens = Lexer::new(source).scan()?;
    let statements = Parser::new(tokens).parse()?;
    interpreter.run(&statements)
}
fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--version" || arg == "-V") { println!("RUST++ v0.1.0"); return; }
    if let Some(path) = args.first() {
        if !path.ends_with(".rustqq") { eprintln!("تحذير: الامتداد المقترح لملفات RUST++ هو .rustqq"); }
        match fs::read_to_string(path) { Ok(source) => if let Err(error) = execute_source(&source, &mut Interpreter::new()) { eprintln!("خطأ: {}", error); std::process::exit(1); }, Err(error) => { eprintln!("تعذر فتح الملف '{}': {}", path, error); std::process::exit(1); } }
        return;
    }
    println!("RUST++ v0.1.0 — اكتب exit للخروج");
    let stdin = io::stdin();
    let mut interpreter = Interpreter::new();
    loop {
        print!("rustqq> ");
        if io::stdout().flush().is_err() { break; }
        let mut line = String::new();
        match stdin.read_line(&mut line) { Ok(0) => break, Ok(_) => { if line.trim() == "exit" { break; } if let Err(error) = execute_source(&line, &mut interpreter) { eprintln!("خطأ: {}", error); } }, Err(error) => { eprintln!("تعذر قراءة الإدخال: {}", error); break; } }
    }
}
