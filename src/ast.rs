
#[derive(Debug, Clone)]
pub enum FStringExprPart {
    Literal(String),
    Expression(Expr),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Set {
        target: Expr,
        value: Expr,
    },
    AugAssign {
        target: Expr,
        op: String,
        value: Expr,
    },
    Expression(Expr),
    Print(Vec<Expr>),
    Pass,

    If {
        condition: Expr,
        then_body: Vec<Stmt>,
        elif_branches: Vec<(Expr, Vec<Stmt>)>,
        else_body: Option<Vec<Stmt>>,
    },
    ForIn {
        var: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    Break,
    Continue,

    Func {
        name: String,
        args: Vec<String>,
        body: Vec<Stmt>,
    },
    Return(Option<Expr>),

    Class {
        name: String,
        methods: Vec<Stmt>,
    },

    Try {
        body: Vec<Stmt>,
        except_var: Option<String>,
        except_body: Vec<Stmt>,
    },

    Import {
        path: String,
        alias: Option<String>,
    },
    FromImport {
        path: String,
        names: Vec<String>,
    },
    Export(Box<Stmt>),

    // Harbor-specific
    Server {
        port: Expr,
        routes: Vec<Route>,
    },
    Respond {
        status: Option<u16>,
        value: Expr,
    },
    Fetch {
        url: Expr,
        body: Vec<Stmt>,
    },
}

#[derive(Debug, Clone)]
pub enum Expr {
    String(String),
    FString(Vec<FStringExprPart>),
    Number(f64),
    Bool(bool),
    None,
    Ident(String),
    Member(Box<Expr>, String),
    Object(Vec<(String, Expr)>),
    Array(Vec<Expr>),
    Binary(Box<Expr>, String, Box<Expr>),
    Unary(String, Box<Expr>),
    Index(Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
}

#[derive(Debug, Clone)]
pub struct Route {
    pub method: String,
    pub path: String,
    pub body: Vec<Stmt>,
}
