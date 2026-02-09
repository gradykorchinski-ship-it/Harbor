use crate::ast::*;
use crate::lexer::{Token, TokenData, FStringPart};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn peek_next(&self) -> Option<&Token> {
        if self.pos + 1 < self.tokens.len() {
            Some(&self.tokens[self.pos + 1])
        } else {
            None
        }
    }

    fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos];
        if tok.data != TokenData::EOF {
            self.pos += 1;
        }
        tok
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while self.peek().data != TokenData::EOF {
            match self.peek().data {
                TokenData::Indent | TokenData::Dedent => {
                    self.advance();
                    continue;
                }
                _ => {}
            }
            stmts.push(self.parse_stmt());
        }
        stmts
    }

    fn parse_block(&mut self) -> Vec<Stmt> {
        let mut body = Vec::new();

        match self.peek().data {
            TokenData::LBrace => {
                self.advance(); // consume '{'
                while !matches!(self.peek().data, TokenData::RBrace | TokenData::EOF) {
                    body.push(self.parse_stmt());
                }
                self.expect(TokenData::RBrace);
            }
            TokenData::Newline => {
                self.advance(); // consume newline after colon
                self.expect(TokenData::Indent);
                loop {
                    while matches!(self.peek().data, TokenData::Newline) {
                        self.advance();
                    }
                    if matches!(self.peek().data, TokenData::Dedent | TokenData::EOF) {
                        break;
                    }
                    body.push(self.parse_stmt());
                }
                self.expect(TokenData::Dedent);
            }
            TokenData::Indent => {
                self.advance();
                loop {
                    while matches!(self.peek().data, TokenData::Newline) {
                        self.advance();
                    }
                    if matches!(self.peek().data, TokenData::Dedent | TokenData::EOF) {
                        break;
                    }
                    body.push(self.parse_stmt());
                }
                self.expect(TokenData::Dedent);
            }
            _ => {
                // Single-line block
                body.push(self.parse_stmt());
            }
        }
        body
    }

    fn parse_stmt(&mut self) -> Stmt {
        // Skip leading newlines
        while matches!(self.peek().data, TokenData::Newline) {
            self.advance();
        }

        let stmt = match self.peek().data {
            // Control flow
            TokenData::If => self.parse_if(),
            TokenData::For => self.parse_for(),
            TokenData::While => self.parse_while(),
            TokenData::Break => { self.advance(); Stmt::Break }
            TokenData::Continue => { self.advance(); Stmt::Continue }
            TokenData::Pass => { self.advance(); Stmt::Pass }

            // Functions & classes
            TokenData::Def => self.parse_func(),
            TokenData::Return => self.parse_return(),
            TokenData::Class => self.parse_class(),

            // Error handling
            TokenData::Try => self.parse_try(),

            // Modules
            TokenData::Import => self.parse_import(),
            TokenData::From => self.parse_from_import(),
            TokenData::Export => self.parse_export(),

            // Print
            TokenData::Print => self.parse_print(),

            // Harbor-specific
            TokenData::Server => self.parse_server(),
            TokenData::Respond => self.parse_respond(),
            TokenData::Fetch => self.parse_fetch(),

            TokenData::EOF => Stmt::Pass,

            // Expression or assignment
            _ => self.parse_expr_or_assign(),
        };

        // Skip trailing newlines
        while matches!(self.peek().data, TokenData::Newline) {
            self.advance();
        }

        stmt
    }

    fn parse_expr_or_assign(&mut self) -> Stmt {
        let expr = self.parse_expr();

        if matches!(self.peek().data, TokenData::Assign) {
            self.advance(); // consume '='
            let value = self.parse_expr();
            match &expr {
                Expr::Ident(_) | Expr::Member(_, _) | Expr::Index(_, _) => {
                    Stmt::Set { target: expr, value }
                }
                _ => {
                    let tok = self.peek();
                    eprintln!("Error: Invalid assignment target at line {}, col {}",
                        tok.span.line, tok.span.col);
                    std::process::exit(1);
                }
            }
        } else if matches!(self.peek().data,
            TokenData::PlusAssign | TokenData::DashAssign |
            TokenData::StarAssign | TokenData::SlashAssign)
        {
            let op = match self.advance().data {
                TokenData::PlusAssign => "+".to_string(),
                TokenData::DashAssign => "-".to_string(),
                TokenData::StarAssign => "*".to_string(),
                TokenData::SlashAssign => "/".to_string(),
                _ => unreachable!(),
            };
            let value = self.parse_expr();
            Stmt::AugAssign { target: expr, op, value }
        } else {
            Stmt::Expression(expr)
        }
    }

    // ─── Control Flow ───

    fn parse_if(&mut self) -> Stmt {
        self.advance(); // consume 'if'
        let condition = self.parse_expr();

        if matches!(self.peek().data, TokenData::Colon) {
            self.advance();
        }

        let then_body = self.parse_block();

        let mut elif_branches = Vec::new();
        let mut else_body = None;

        loop {
            if matches!(self.peek().data, TokenData::Elif) {
                self.advance(); // consume 'elif'
                let elif_cond = self.parse_expr();
                if matches!(self.peek().data, TokenData::Colon) {
                    self.advance();
                }
                let elif_body = self.parse_block();
                elif_branches.push((elif_cond, elif_body));
            } else if matches!(self.peek().data, TokenData::Else) {
                self.advance(); // consume 'else'
                if matches!(self.peek().data, TokenData::Colon) {
                    self.advance();
                }
                else_body = Some(self.parse_block());
                break;
            } else {
                break;
            }
        }

        Stmt::If { condition, then_body, elif_branches, else_body }
    }

    fn parse_for(&mut self) -> Stmt {
        self.advance(); // consume 'for'

        let var_tok = self.advance();
        let var = match &var_tok.data {
            TokenData::Ident(n) => n.clone(),
            _ => {
                eprintln!("Error: Expected variable name after 'for' at line {}, col {}",
                    var_tok.span.line, var_tok.span.col);
                std::process::exit(1);
            }
        };

        self.expect(TokenData::In);

        let iterable = self.parse_expr();

        if matches!(self.peek().data, TokenData::Colon) {
            self.advance();
        }

        let body = self.parse_block();

        Stmt::ForIn { var, iterable, body }
    }

    fn parse_while(&mut self) -> Stmt {
        self.advance(); // consume 'while'
        let condition = self.parse_expr();
        if matches!(self.peek().data, TokenData::Colon) {
            self.advance();
        }
        let body = self.parse_block();
        Stmt::While { condition, body }
    }

    // ─── Functions & Classes ───

    fn parse_func(&mut self) -> Stmt {
        self.advance(); // consume 'def'

        let name = match &self.advance().data {
            TokenData::Ident(n) => n.clone(),
            _ => {
                eprintln!("Error: Expected function name after 'def'");
                std::process::exit(1);
            }
        };

        self.expect(TokenData::LParen);
        let mut args = Vec::new();
        if !matches!(self.peek().data, TokenData::RParen) {
            loop {
                let arg = match &self.advance().data {
                    TokenData::Ident(n) => n.clone(),
                    _ => {
                        eprintln!("Error: Expected argument name");
                        std::process::exit(1);
                    }
                };
                args.push(arg);
                if matches!(self.peek().data, TokenData::RParen) {
                    break;
                }
                self.expect(TokenData::Comma);
            }
        }
        self.expect(TokenData::RParen);

        if matches!(self.peek().data, TokenData::Colon) {
            self.advance();
        }

        let body = self.parse_block();
        Stmt::Func { name, args, body }
    }

    fn parse_return(&mut self) -> Stmt {
        self.advance(); // consume 'return'
        if matches!(self.peek().data, TokenData::Newline | TokenData::EOF | TokenData::Dedent) {
            Stmt::Return(None)
        } else {
            Stmt::Return(Some(self.parse_expr()))
        }
    }

    fn parse_class(&mut self) -> Stmt {
        self.advance(); // consume 'class'
        let name = match &self.advance().data {
            TokenData::Ident(n) => n.clone(),
            _ => {
                eprintln!("Error: Expected class name after 'class'");
                std::process::exit(1);
            }
        };

        if matches!(self.peek().data, TokenData::Colon) {
            self.advance();
        }

        let methods = self.parse_block();
        Stmt::Class { name, methods }
    }

    // ─── Error Handling ───

    fn parse_try(&mut self) -> Stmt {
        self.advance(); // consume 'try'

        if matches!(self.peek().data, TokenData::Colon) {
            self.advance();
        }

        let body = self.parse_block();

        self.expect(TokenData::Except);

        let mut except_var = None;
        if let TokenData::Ident(name) = &self.peek().data {
            except_var = Some(name.clone());
            self.advance();
        }

        if matches!(self.peek().data, TokenData::Colon) {
            self.advance();
        }

        let except_body = self.parse_block();

        Stmt::Try { body, except_var, except_body }
    }

    // ─── Modules ───

    fn parse_import(&mut self) -> Stmt {
        self.advance(); // consume 'import'

        let path = match &self.advance().data {
            TokenData::String(s) => s.clone(),
            _ => {
                eprintln!("Error: Expected string path after 'import'");
                std::process::exit(1);
            }
        };

        let mut alias = None;
        if matches!(self.peek().data, TokenData::As) {
            self.advance(); // consume 'as'
            match &self.advance().data {
                TokenData::Ident(name) => {
                    alias = Some(name.clone());
                }
                _ => {
                    eprintln!("Error: Expected identifier after 'as'");
                    std::process::exit(1);
                }
            }
        }

        Stmt::Import { path, alias }
    }

    fn parse_from_import(&mut self) -> Stmt {
        self.advance(); // consume 'from'

        let path = match &self.advance().data {
            TokenData::String(s) => s.clone(),
            _ => {
                eprintln!("Error: Expected string path after 'from'");
                std::process::exit(1);
            }
        };

        self.expect(TokenData::Import);

        let mut names = Vec::new();
        loop {
            match &self.advance().data {
                TokenData::Ident(n) => names.push(n.clone()),
                _ => {
                    eprintln!("Error: Expected identifier in import list");
                    std::process::exit(1);
                }
            }
            if !matches!(self.peek().data, TokenData::Comma) {
                break;
            }
            self.advance(); // consume comma
        }

        Stmt::FromImport { path, names }
    }

    fn parse_export(&mut self) -> Stmt {
        self.advance(); // consume 'export'
        let stmt = self.parse_stmt();
        Stmt::Export(Box::new(stmt))
    }

    // ─── Print ───

    fn parse_print(&mut self) -> Stmt {
        self.advance(); // consume 'print'

        let mut exprs = Vec::new();

        // Check if we've hit end of statement
        if matches!(self.peek().data, TokenData::Newline | TokenData::EOF | TokenData::Dedent) {
            // print with no arguments → print empty line
            exprs.push(Expr::String("".to_string()));
            return Stmt::Print(exprs);
        }

        loop {
            exprs.push(self.parse_expr());
            if !matches!(self.peek().data, TokenData::Comma) {
                break;
            }
            // Only consume comma if it's not inside parens (argument separator)
            // Check that the next token after comma isn't end-of-statement,
            // which would imply it's a print separator
            self.advance(); // consume comma
            if matches!(self.peek().data, TokenData::Newline | TokenData::EOF | TokenData::Dedent) {
                break;
            }
        }

        Stmt::Print(exprs)
    }

    // ─── Harbor-specific ───

    fn parse_server(&mut self) -> Stmt {
        self.advance(); // consume 'server'

        while matches!(self.peek().data, TokenData::Indent) {
            self.advance();
        }

        let port = match self.peek().data {
            TokenData::LBrace | TokenData::Colon | TokenData::Newline | TokenData::Indent => Expr::Number(8080.0),
            _ => self.parse_expr(),
        };

        if matches!(self.peek().data, TokenData::Colon) {
            self.advance();
        }

        let mut routes = Vec::new();
        match self.peek().data {
            TokenData::LBrace => {
                self.advance();
                while !matches!(self.peek().data, TokenData::RBrace | TokenData::EOF) {
                    routes.push(self.parse_route());
                }
                self.expect(TokenData::RBrace);
            }
            TokenData::Indent | TokenData::Newline => {
                routes = self.parse_routes_block();
            }
            _ => {
                let tok = self.peek();
                eprintln!("Error: Expected block after server at line {}, col {}, found {:?}",
                    tok.span.line, tok.span.col, tok.data);
                std::process::exit(1);
            }
        }

        Stmt::Server { port, routes }
    }

    fn parse_routes_block(&mut self) -> Vec<Route> {
        while matches!(self.peek().data, TokenData::Newline) {
            self.advance();
        }
        self.expect(TokenData::Indent);
        let mut routes = Vec::new();
        while !matches!(self.peek().data, TokenData::Dedent | TokenData::EOF) {
            routes.push(self.parse_route());
            while matches!(self.peek().data, TokenData::Newline) {
                self.advance();
            }
        }
        self.expect(TokenData::Dedent);
        routes
    }

    fn parse_route(&mut self) -> Route {
        while matches!(self.peek().data, TokenData::Newline) {
            self.advance();
        }
        let method_tok = self.advance();
        let method = match &method_tok.data {
            TokenData::Get => "GET".to_string(),
            TokenData::Post => "POST".to_string(),
            TokenData::Put => "PUT".to_string(),
            TokenData::Delete => "DELETE".to_string(),
            TokenData::Patch => "PATCH".to_string(),
            _ => {
                eprintln!("Error: Expected HTTP method (get, post, put, delete, patch) at line {}, col {}, found {:?}",
                    method_tok.span.line, method_tok.span.col, method_tok.data);
                std::process::exit(1);
            }
        };

        let path_tok = self.advance();
        let path = match &path_tok.data {
            TokenData::String(s) => s.clone(),
            _ => {
                eprintln!("Error: Expected string path in route at line {}, col {}, found {:?}",
                    path_tok.span.line, path_tok.span.col, path_tok.data);
                std::process::exit(1);
            }
        };

        if matches!(self.peek().data, TokenData::Colon) {
            self.advance();
        }

        let body = self.parse_block();

        Route { method, path, body }
    }

    fn parse_respond(&mut self) -> Stmt {
        self.advance(); // consume 'respond'

        let status = if let TokenData::Number(n) = self.peek().data {
            self.advance();
            Some(n as u16)
        } else {
            None
        };

        let value = self.parse_expr();
        Stmt::Respond { status, value }
    }

    fn parse_fetch(&mut self) -> Stmt {
        self.advance(); // consume 'fetch'
        let url = self.parse_expr();

        if matches!(self.peek().data, TokenData::Colon) {
            self.advance();
        }

        let body = self.parse_block();
        Stmt::Fetch { url, body }
    }

    // ─── Expression Parsing (Precedence Climbing) ───

    pub fn parse_expr(&mut self) -> Expr {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Expr {
        let mut expr = self.parse_and();
        while matches!(self.peek().data, TokenData::Or) {
            self.advance();
            let right = self.parse_and();
            expr = Expr::Binary(Box::new(expr), "or".to_string(), Box::new(right));
        }
        expr
    }

    fn parse_and(&mut self) -> Expr {
        let mut expr = self.parse_not();
        while matches!(self.peek().data, TokenData::And) {
            self.advance();
            let right = self.parse_not();
            expr = Expr::Binary(Box::new(expr), "and".to_string(), Box::new(right));
        }
        expr
    }

    fn parse_not(&mut self) -> Expr {
        if matches!(self.peek().data, TokenData::Not) {
            // Check for "not in" (two-token operator)
            if self.peek_next().map(|t| &t.data) == Some(&TokenData::In) {
                // Not "not" as unary prefix; let comparison handle "not in"
                return self.parse_comparison();
            }
            self.advance(); // consume 'not'
            let right = self.parse_not();
            return Expr::Unary("not".to_string(), Box::new(right));
        }
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Expr {
        let mut expr = self.parse_term();

        while matches!(self.peek().data,
            TokenData::Eq | TokenData::NotEq |
            TokenData::Less | TokenData::Greater |
            TokenData::LessEq | TokenData::GreaterEq |
            TokenData::In | TokenData::Not)
        {
            // Handle "not in"
            if matches!(self.peek().data, TokenData::Not) {
                if self.peek_next().map(|t| &t.data) == Some(&TokenData::In) {
                    self.advance(); // consume 'not'
                    self.advance(); // consume 'in'
                    let right = self.parse_term();
                    expr = Expr::Binary(Box::new(expr), "not in".to_string(), Box::new(right));
                    continue;
                } else {
                    break; // just 'not' without 'in' at comparison level
                }
            }

            // Handle "in"
            if matches!(self.peek().data, TokenData::In) {
                self.advance();
                let right = self.parse_term();
                expr = Expr::Binary(Box::new(expr), "in".to_string(), Box::new(right));
                continue;
            }

            let op = match self.advance().data {
                TokenData::Eq => "===".to_string(),
                TokenData::NotEq => "!==".to_string(),
                TokenData::Less => "<".to_string(),
                TokenData::Greater => ">".to_string(),
                TokenData::LessEq => "<=".to_string(),
                TokenData::GreaterEq => ">=".to_string(),
                _ => unreachable!(),
            };
            let right = self.parse_term();
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }

        expr
    }

    fn parse_term(&mut self) -> Expr {
        let mut expr = self.parse_factor();
        while matches!(self.peek().data, TokenData::Plus | TokenData::Dash) {
            let op = match self.advance().data {
                TokenData::Plus => "+".to_string(),
                TokenData::Dash => "-".to_string(),
                _ => unreachable!(),
            };
            let right = self.parse_factor();
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        expr
    }

    fn parse_factor(&mut self) -> Expr {
        let mut expr = self.parse_power();
        while matches!(self.peek().data,
            TokenData::Star | TokenData::Slash |
            TokenData::Percent | TokenData::DoubleSlash)
        {
            let op = match self.advance().data {
                TokenData::Star => "*".to_string(),
                TokenData::Slash => "/".to_string(),
                TokenData::Percent => "%".to_string(),
                TokenData::DoubleSlash => "//".to_string(),
                _ => unreachable!(),
            };
            let right = self.parse_power();
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        expr
    }

    fn parse_power(&mut self) -> Expr {
        let base = self.parse_unary();
        if matches!(self.peek().data, TokenData::DoubleStar) {
            self.advance();
            let exp = self.parse_power(); // right-associative
            Expr::Binary(Box::new(base), "**".to_string(), Box::new(exp))
        } else {
            base
        }
    }

    fn parse_unary(&mut self) -> Expr {
        if matches!(self.peek().data, TokenData::Dash) {
            self.advance();
            let right = self.parse_unary();
            return Expr::Unary("-".to_string(), Box::new(right));
        }
        self.parse_member()
    }

    fn parse_member(&mut self) -> Expr {
        let mut expr = self.parse_primary();

        while matches!(self.peek().data, TokenData::Dot | TokenData::LBracket | TokenData::LParen) {
            if matches!(self.peek().data, TokenData::Dot) {
                self.advance();
                let field_tok = self.advance();
                let field = match &field_tok.data {
                    TokenData::Ident(s) => s.clone(),
                    TokenData::String(s) => s.clone(),
                    _ => {
                        eprintln!("Error: Expected field name after '.' at line {}, col {}, found {:?}",
                            field_tok.span.line, field_tok.span.col, field_tok.data);
                        std::process::exit(1);
                    }
                };
                expr = Expr::Member(Box::new(expr), field);
            } else if matches!(self.peek().data, TokenData::LBracket) {
                self.advance();
                let index = self.parse_expr();
                self.expect(TokenData::RBracket);
                expr = Expr::Index(Box::new(expr), Box::new(index));
            } else if matches!(self.peek().data, TokenData::LParen) {
                self.advance();
                let args = self.parse_arguments();
                self.expect(TokenData::RParen);
                expr = Expr::Call(Box::new(expr), args);
            }
        }

        expr
    }

    fn parse_arguments(&mut self) -> Vec<Expr> {
        let mut args = Vec::new();
        if !matches!(self.peek().data, TokenData::RParen) {
            loop {
                args.push(self.parse_expr());
                if matches!(self.peek().data, TokenData::RParen) {
                    break;
                }
                self.expect(TokenData::Comma);
            }
        }
        args
    }

    fn parse_primary(&mut self) -> Expr {
        // Skip stray Indents/Dedents
        while matches!(self.peek().data, TokenData::Indent | TokenData::Dedent) {
            self.advance();
        }

        let tok = self.advance();
        match &tok.data {
            TokenData::String(s) => Expr::String(s.clone()),
            TokenData::Number(n) => Expr::Number(*n),
            TokenData::True => Expr::Bool(true),
            TokenData::False => Expr::Bool(false),
            TokenData::None_ => Expr::None,
            TokenData::Ident(name) => Expr::Ident(name.clone()),
            TokenData::Self_ => Expr::Ident("this".to_string()),

            TokenData::LBrace => self.parse_object(),
            TokenData::LBracket => self.parse_array(),

            TokenData::LParen => {
                let expr = self.parse_expr();
                self.expect(TokenData::RParen);
                expr
            }

            TokenData::FStringToken(parts) => {
                let parts = parts.clone();
                let mut expr_parts = Vec::new();
                for part in &parts {
                    match part {
                        FStringPart::Literal(s) => {
                            expr_parts.push(FStringExprPart::Literal(s.clone()));
                        }
                        FStringPart::Expression(text) => {
                            let mut sub_lexer = crate::lexer::Lexer::new(text);
                            let sub_tokens = sub_lexer.tokenize();
                            let mut sub_parser = Parser::new(sub_tokens);
                            let expr = sub_parser.parse_expr();
                            expr_parts.push(FStringExprPart::Expression(expr));
                        }
                    }
                }
                Expr::FString(expr_parts)
            }

            _ => {
                eprintln!("Error: Unexpected token {:?} in expression at line {}, col {}",
                    tok.data, tok.span.line, tok.span.col);
                std::process::exit(1);
            }
        }
    }

    fn parse_object(&mut self) -> Expr {
        let mut fields = Vec::new();
        if !matches!(self.peek().data, TokenData::RBrace) {
            loop {
                let key_tok = self.advance();
                let key = match &key_tok.data {
                    TokenData::String(s) => s.clone(),
                    TokenData::Ident(s) => s.clone(),
                    _ => {
                        eprintln!("Error: Expected key in object at line {}, col {}, found {:?}",
                            key_tok.span.line, key_tok.span.col, key_tok.data);
                        std::process::exit(1);
                    }
                };

                self.expect(TokenData::Colon);
                let value = self.parse_expr();
                fields.push((key, value));

                if matches!(self.peek().data, TokenData::RBrace) {
                    break;
                }
                self.expect(TokenData::Comma);
            }
        }
        self.expect(TokenData::RBrace);
        Expr::Object(fields)
    }

    fn parse_array(&mut self) -> Expr {
        let mut elements = Vec::new();
        if !matches!(self.peek().data, TokenData::RBracket) {
            loop {
                elements.push(self.parse_expr());
                if matches!(self.peek().data, TokenData::RBracket) {
                    break;
                }
                self.expect(TokenData::Comma);
            }
        }
        self.expect(TokenData::RBracket);
        Expr::Array(elements)
    }

    fn expect(&mut self, expected: TokenData) {
        let tok = self.advance();
        if tok.data != expected {
            eprintln!("Error: Expected {:?} at line {}, col {}, found {:?}",
                expected, tok.span.line, tok.span.col, tok.data);
            std::process::exit(1);
        }
    }
}
