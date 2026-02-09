#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FStringPart {
    Literal(String),
    Expression(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenData {
    // Python-like keywords
    Def,
    Return,
    If,
    Elif,
    Else,
    For,
    In,
    While,
    Break,
    Continue,
    Class,
    Self_,
    Pass,
    Try,
    Except,
    Import,
    From,
    As,
    Export,
    And,
    Or,
    Not,

    // Literals / values
    True,
    False,
    None_,

    // Harbor built-ins
    Print,
    Server,
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Respond,
    Fetch,

    // Indentation
    Indent,
    Dedent,
    Newline,

    // Identifiers and literals
    Ident(String),
    String(String),
    Number(f64),
    FStringToken(Vec<FStringPart>),

    // Punctuation
    Dot,
    Colon,
    Comma,
    Assign,     // =
    Eq,         // ==
    NotEq,      // !=
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    LParen,
    RParen,

    // Operators
    Plus,
    Dash,
    Star,
    Slash,
    Percent,      // %
    DoubleStar,   // **
    DoubleSlash,  // //
    Less,
    Greater,
    LessEq,
    GreaterEq,

    // Compound assignment
    PlusAssign,   // +=
    DashAssign,   // -=
    StarAssign,   // *=
    SlashAssign,  // /=

    EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub data: TokenData,
    pub span: Span,
}

pub struct Lexer {
    src: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    indent_stack: Vec<usize>,
    pending_tokens: std::collections::VecDeque<Token>,
    at_line_start: bool,
    brace_level: usize,
    bracket_level: usize,
    paren_level: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            src: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            indent_stack: vec![0],
            pending_tokens: std::collections::VecDeque::new(),
            at_line_start: true,
            brace_level: 0,
            bracket_level: 0,
            paren_level: 0,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            let is_eof = tok.data == TokenData::EOF;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        tokens
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos >= self.src.len() {
            return None;
        }
        let ch = self.src[self.pos];
        self.pos += 1;
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
            self.at_line_start = true;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    fn peek(&self) -> Option<char> {
        if self.pos >= self.src.len() {
            return None;
        }
        Some(self.src[self.pos])
    }

    #[allow(dead_code)]
    fn peek_ahead(&self, offset: usize) -> Option<char> {
        let idx = self.pos + offset;
        if idx >= self.src.len() {
            return None;
        }
        Some(self.src[idx])
    }

    fn inside_brackets(&self) -> bool {
        self.brace_level > 0 || self.bracket_level > 0 || self.paren_level > 0
    }

    fn scan_fstring(&mut self) -> TokenData {
        let quote = self.advance().unwrap(); // consume opening ' or "
        let mut parts = Vec::new();
        let mut current_lit = String::new();

        while let Some(c) = self.peek() {
            if c == '\\' {
                current_lit.push(self.advance().unwrap());
                if let Some(escaped) = self.advance() {
                    current_lit.push(escaped);
                }
            } else if c == '{' {
                self.advance(); // consume '{'
                // Check for {{ (escaped brace)
                if self.peek() == Some('{') {
                    self.advance();
                    current_lit.push('{');
                    continue;
                }
                // Save current literal
                if !current_lit.is_empty() {
                    parts.push(FStringPart::Literal(current_lit.clone()));
                    current_lit.clear();
                }
                // Collect expression text until '}'
                let mut expr_text = String::new();
                let mut brace_depth = 1;
                while let Some(ec) = self.peek() {
                    if ec == '}' {
                        brace_depth -= 1;
                        if brace_depth == 0 {
                            self.advance(); // consume '}'
                            break;
                        }
                    } else if ec == '{' {
                        brace_depth += 1;
                    }
                    expr_text.push(self.advance().unwrap());
                }
                parts.push(FStringPart::Expression(expr_text));
            } else if c == '}' {
                self.advance();
                // Check for }} (escaped brace)
                if self.peek() == Some('}') {
                    self.advance();
                    current_lit.push('}');
                } else {
                    current_lit.push('}');
                }
            } else if c == quote {
                self.advance(); // consume closing quote
                break;
            } else {
                current_lit.push(self.advance().unwrap());
            }
        }

        if !current_lit.is_empty() {
            parts.push(FStringPart::Literal(current_lit));
        }

        TokenData::FStringToken(parts)
    }

    fn next_token(&mut self) -> Token {
        if let Some(tok) = self.pending_tokens.pop_front() {
            return tok;
        }

        // Handle indentation at line starts (only outside brackets)
        if self.at_line_start && !self.inside_brackets() {
            self.at_line_start = false;
            let indent;

            loop {
                let mut current_line_spaces = 0;
                while let Some(ch) = self.peek() {
                    match ch {
                        ' ' => { current_line_spaces += 1; self.advance(); }
                        '\t' => { current_line_spaces += 4; self.advance(); }
                        _ => break,
                    }
                }

                match self.peek() {
                    Some('\n') => {
                        self.advance();
                        self.at_line_start = true;
                        continue;
                    }
                    Some('#') => {
                        // Skip comment lines entirely
                        while let Some(c) = self.peek() {
                            if c == '\n' { break; }
                            self.advance();
                        }
                        if self.peek() == Some('\n') {
                            self.advance();
                            self.at_line_start = true;
                            continue;
                        }
                        indent = current_line_spaces;
                        break;
                    }
                    Some(_) => {
                        indent = current_line_spaces;
                        self.at_line_start = false;
                        break;
                    }
                    None => {
                        indent = 0;
                        self.at_line_start = false;
                        break;
                    }
                }
            }

            let current_indent = *self.indent_stack.last().unwrap();
            if indent > current_indent {
                self.indent_stack.push(indent);
                return Token { data: TokenData::Indent, span: Span { line: self.line, col: self.col } };
            } else if indent < current_indent {
                while indent < *self.indent_stack.last().unwrap() {
                    self.indent_stack.pop();
                    self.pending_tokens.push_back(Token { data: TokenData::Dedent, span: Span { line: self.line, col: self.col } });
                }
                if let Some(tok) = self.pending_tokens.pop_front() {
                    return tok;
                }
            }
        }

        // Skip non-significant whitespace
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else if ch == '\n' && self.inside_brackets() {
                self.advance();
                self.at_line_start = true;
            } else {
                break;
            }
        }

        let start_line = self.line;
        let start_col = self.col;
        let span = Span { line: start_line, col: start_col };

        let ch = match self.peek() {
            Some(c) => c,
            None => {
                // EOF: emit remaining dedents
                while self.indent_stack.len() > 1 {
                    self.indent_stack.pop();
                    self.pending_tokens.push_back(Token { data: TokenData::Dedent, span });
                }
                if let Some(tok) = self.pending_tokens.pop_front() {
                    return tok;
                }
                return Token { data: TokenData::EOF, span };
            }
        };

        if ch == '\n' {
            self.advance();
            self.at_line_start = true;
            return Token { data: TokenData::Newline, span };
        }

        self.advance(); // advance past the peeked character

        let data = match ch {
            // Brackets
            '{' => { self.brace_level += 1; TokenData::LBrace }
            '}' => { self.brace_level = self.brace_level.saturating_sub(1); TokenData::RBrace }
            '[' => { self.bracket_level += 1; TokenData::LBracket }
            ']' => { self.bracket_level = self.bracket_level.saturating_sub(1); TokenData::RBracket }
            '(' => { self.paren_level += 1; TokenData::LParen }
            ')' => { self.paren_level = self.paren_level.saturating_sub(1); TokenData::RParen }

            '.' => TokenData::Dot,
            ':' => TokenData::Colon,
            ',' => TokenData::Comma,
            '%' => TokenData::Percent,

            // Comments (Python-style)
            '#' => {
                while let Some(c) = self.peek() {
                    if c == '\n' { break; }
                    self.advance();
                }
                return self.next_token();
            }

            // Operators with multi-char variants
            '=' if self.peek() == Some('=') => { self.advance(); TokenData::Eq }
            '=' => TokenData::Assign,

            '!' if self.peek() == Some('=') => { self.advance(); TokenData::NotEq }
            '!' => {
                eprintln!("Error: Use 'not' instead of '!' at line {}, col {}", start_line, start_col);
                std::process::exit(1);
            }

            '+' if self.peek() == Some('=') => { self.advance(); TokenData::PlusAssign }
            '+' => TokenData::Plus,

            '-' if self.peek() == Some('=') => { self.advance(); TokenData::DashAssign }
            '-' => TokenData::Dash,

            '*' if self.peek() == Some('*') => { self.advance(); TokenData::DoubleStar }
            '*' if self.peek() == Some('=') => { self.advance(); TokenData::StarAssign }
            '*' => TokenData::Star,

            '/' if self.peek() == Some('/') => { self.advance(); TokenData::DoubleSlash }
            '/' if self.peek() == Some('=') => { self.advance(); TokenData::SlashAssign }
            '/' => TokenData::Slash,

            '<' if self.peek() == Some('=') => { self.advance(); TokenData::LessEq }
            '<' => TokenData::Less,

            '>' if self.peek() == Some('=') => { self.advance(); TokenData::GreaterEq }
            '>' => TokenData::Greater,

            // Strings (single and double quotes)
            '"' | '\'' => {
                let quote = ch;
                let mut s = String::new();
                while let Some(c) = self.peek() {
                    if c == '\\' {
                        s.push(self.advance().unwrap()); // push '\'
                        if let Some(escaped) = self.advance() {
                            s.push(escaped);
                        }
                    } else if c == quote {
                        self.advance(); // consume closing quote
                        break;
                    } else if c == '\n' {
                        // Unterminated string
                        break;
                    } else {
                        s.push(self.advance().unwrap());
                    }
                }
                TokenData::String(s)
            }

            // Identifiers and keywords
            c if c.is_ascii_alphabetic() || c == '_' => {
                let mut ident = String::from(c);
                while let Some(n) = self.peek() {
                    if n.is_ascii_alphanumeric() || n == '_' {
                        ident.push(self.advance().unwrap());
                    } else {
                        break;
                    }
                }

                // Check for f-string: identifier "f" followed by quote
                if ident == "f" && matches!(self.peek(), Some('"') | Some('\'')) {
                    return Token { data: self.scan_fstring(), span };
                }

                match ident.as_str() {
                    // Python-like keywords
                    "def" => TokenData::Def,
                    "return" => TokenData::Return,
                    "if" => TokenData::If,
                    "elif" => TokenData::Elif,
                    "else" => TokenData::Else,
                    "for" => TokenData::For,
                    "in" => TokenData::In,
                    "while" => TokenData::While,
                    "break" => TokenData::Break,
                    "continue" => TokenData::Continue,
                    "class" => TokenData::Class,
                    "self" => TokenData::Self_,
                    "pass" => TokenData::Pass,
                    "try" => TokenData::Try,
                    "except" => TokenData::Except,
                    "import" => TokenData::Import,
                    "from" => TokenData::From,
                    "as" => TokenData::As,
                    "export" => TokenData::Export,
                    "and" => TokenData::And,
                    "or" => TokenData::Or,
                    "not" => TokenData::Not,

                    // Python-cased booleans & None
                    "True" => TokenData::True,
                    "False" => TokenData::False,
                    "true" => TokenData::True,   // alias
                    "false" => TokenData::False,  // alias
                    "None" => TokenData::None_,

                    // Harbor built-ins
                    "print" => TokenData::Print,
                    "server" => TokenData::Server,
                    "get" => TokenData::Get,
                    "post" => TokenData::Post,
                    "put" => TokenData::Put,
                    "delete" => TokenData::Delete,
                    "patch" => TokenData::Patch,
                    "respond" => TokenData::Respond,
                    "fetch" => TokenData::Fetch,

                    _ => TokenData::Ident(ident),
                }
            }

            // Numbers
            c if c.is_ascii_digit() => {
                let mut n = String::from(c);
                while let Some(next) = self.peek() {
                    if next.is_ascii_digit() || next == '.' {
                        n.push(self.advance().unwrap());
                    } else {
                        break;
                    }
                }
                TokenData::Number(n.parse().unwrap_or(0.0))
            }

            _ => {
                eprintln!("Error: Unexpected character '{}' at line {}, col {}",
                    ch, start_line, start_col);
                std::process::exit(1);
            }
        };

        Token { data, span }
    }
}
