use crate::ast::*;

pub struct CodeGen;

impl CodeGen {
    pub fn generate(stmts: &[Stmt]) -> String {
        let mut output = String::new();

        // ─── Runtime Header (Python-like builtins) ───
        output.push_str("const http = require(\"http\");\n");
        output.push_str("const https = require(\"https\");\n");
        output.push_str("const readline = require(\"readline\");\n");
        output.push_str("const __fs = require(\"fs/promises\");\n\n");

        // File system
        output.push_str("const fs = {\n");
        output.push_str("  read: (path) => __fs.readFile(path, 'utf-8'),\n");
        output.push_str("  write: (path, content) => __fs.writeFile(path, String(content))\n");
        output.push_str("};\n\n");

        // Python-like builtins
        output.push_str("const len = (obj) => {\n");
        output.push_str("  if (obj == null) return 0;\n");
        output.push_str("  if (typeof obj === 'string' || Array.isArray(obj)) return obj.length;\n");
        output.push_str("  if (typeof obj === 'object') return Object.keys(obj).length;\n");
        output.push_str("  return 0;\n");
        output.push_str("};\n");
        output.push_str("const str = (x) => String(x);\n");
        output.push_str("const int = (x) => parseInt(x, 10);\n");
        output.push_str("const float = (x) => parseFloat(x);\n");
        output.push_str("const bool = (x) => Boolean(x);\n");
        output.push_str("const type = (x) => typeof x;\n");
        output.push_str("const abs = (x) => Math.abs(x);\n");
        output.push_str("const round = (x) => Math.round(x);\n");
        output.push_str("const sorted = (arr) => [...arr].sort();\n");
        output.push_str("const reversed = (arr) => [...arr].reverse();\n");
        output.push_str("const sum = (arr) => arr.reduce((a, b) => a + b, 0);\n");
        output.push_str("const min = (...a) => a.length === 1 && Array.isArray(a[0]) ? Math.min(...a[0]) : Math.min(...a);\n");
        output.push_str("const max = (...a) => a.length === 1 && Array.isArray(a[0]) ? Math.max(...a[0]) : Math.max(...a);\n");
        output.push_str("const enumerate = (arr) => arr.map((v, i) => [i, v]);\n");
        output.push_str("const zip = (...arrays) => arrays[0].map((_, i) => arrays.map(a => a[i]));\n");
        output.push_str("const any = (arr) => arr.some(Boolean);\n");
        output.push_str("const all = (arr) => arr.every(Boolean);\n");
        output.push_str("const keys = (obj) => Object.keys(obj);\n");
        output.push_str("const values = (obj) => Object.values(obj);\n");
        output.push_str("const items = (obj) => Object.entries(obj);\n");
        output.push_str("const isinstance = (obj, cls) => obj instanceof cls;\n");
        output.push_str("const chr = (n) => String.fromCharCode(n);\n");
        output.push_str("const ord = (c) => c.charCodeAt(0);\n\n");

        // range() function
        output.push_str("const range = (...args) => {\n");
        output.push_str("  let start = 0, end, step = 1;\n");
        output.push_str("  if (args.length === 1) { end = args[0]; }\n");
        output.push_str("  else if (args.length === 2) { start = args[0]; end = args[1]; }\n");
        output.push_str("  else { start = args[0]; end = args[1]; step = args[2]; }\n");
        output.push_str("  const r = [];\n");
        output.push_str("  if (step > 0) { for (let i = start; i < end; i += step) r.push(i); }\n");
        output.push_str("  else { for (let i = start; i > end; i += step) r.push(i); }\n");
        output.push_str("  return r;\n");
        output.push_str("};\n\n");

        // input() function
        output.push_str("const input = (msg) => new Promise(resolve => {\n");
        output.push_str("  const rl = readline.createInterface({ input: process.stdin, output: process.stdout });\n");
        output.push_str("  rl.question(String(msg || ''), ans => {\n");
        output.push_str("    rl.close();\n");
        output.push_str("    resolve(ans);\n");
        output.push_str("  });\n");
        output.push_str("});\n\n");

        // Membership test helper for 'in' / 'not in'
        output.push_str("const __contains = (container, item) => {\n");
        output.push_str("  if (Array.isArray(container)) return container.includes(item);\n");
        output.push_str("  if (typeof container === 'string') return container.includes(item);\n");
        output.push_str("  if (typeof container === 'object' && container !== null) return item in container;\n");
        output.push_str("  return false;\n");
        output.push_str("};\n\n");

        // HTTP helpers
        output.push_str("const parseJsonBody = (req) => new Promise((resolve) => {\n");
        output.push_str("  let body = \"\";\n");
        output.push_str("  req.on(\"data\", (chunk) => body += chunk);\n");
        output.push_str("  req.on(\"end\", () => {\n");
        output.push_str("    try { resolve(JSON.parse(body)); } catch { resolve({}); }\n");
        output.push_str("  });\n");
        output.push_str("});\n\n");

        output.push_str("const fetchJson = (url) => new Promise((resolve) => {\n");
        output.push_str("  const lib = url.startsWith(\"https\") ? https : http;\n");
        output.push_str("  lib.get(url, { headers: { \"User-Agent\": \"Harbor/2.0\" } }, (res) => {\n");
        output.push_str("    let data = \"\";\n");
        output.push_str("    res.on(\"data\", (chunk) => data += chunk);\n");
        output.push_str("    res.on(\"end\", () => {\n");
        output.push_str("      try { res.body = JSON.parse(data); } catch { res.body = data; }\n");
        output.push_str("      resolve(res);\n");
        output.push_str("    });\n");
        output.push_str("  }).on(\"error\", (err) => {\n");
        output.push_str("    resolve({ statusCode: 500, body: { error: err.message } });\n");
        output.push_str("  });\n");
        output.push_str("});\n\n");

        // Wrap in async IIFE
        output.push_str("(async () => {\n");

        for stmt in stmts {
            output.push_str(&Self::gen_stmt(stmt, "null", "  "));
        }

        output.push_str("})();\n");

        output
    }

    // ─── Statement Code Generation ───

    fn gen_stmt(stmt: &Stmt, req_name: &str, indent: &str) -> String {
        let inner = format!("{}  ", indent);
        let mut code = String::new();

        match stmt {
            Stmt::Set { target, value } => {
                let val = Self::gen_val(value, req_name);
                match target {
                    Expr::Ident(name) => {
                        code.push_str(&format!("{}var {} = {};\n", indent, name, val));
                    }
                    Expr::Member(obj, field) => {
                        let obj_code = Self::gen_val(obj, req_name);
                        let final_obj = if obj_code == "self" { "this".to_string() } else { obj_code };
                        code.push_str(&format!("{}{}.{} = {};\n", indent, final_obj, field, val));
                    }
                    Expr::Index(obj, idx) => {
                        code.push_str(&format!("{}{}[{}] = {};\n", indent,
                            Self::gen_val(obj, req_name),
                            Self::gen_val(idx, req_name),
                            val));
                    }
                    _ => {
                        let target_code = Self::gen_val(target, req_name);
                        code.push_str(&format!("{}{} = {};\n", indent, target_code, val));
                    }
                }
            }

            Stmt::AugAssign { target, op, value } => {
                let target_code = Self::gen_val(target, req_name);
                let val = Self::gen_val(value, req_name);
                code.push_str(&format!("{}{} {}= {};\n", indent, target_code, op, val));
            }

            Stmt::Expression(expr) => {
                let val = Self::gen_val(expr, req_name);
                code.push_str(&format!("{}{};\n", indent, val));
            }

            Stmt::Print(exprs) => {
                let vals: Vec<String> = exprs.iter().map(|e| Self::gen_val(e, req_name)).collect();
                code.push_str(&format!("{}console.log({});\n", indent, vals.join(", ")));
            }

            Stmt::Pass => {
                code.push_str(&format!("{}/* pass */\n", indent));
            }

            Stmt::If { condition, then_body, elif_branches, else_body } => {
                let cond = Self::gen_val(condition, req_name);
                code.push_str(&format!("{}if ({}) {{\n", indent, cond));
                for s in then_body {
                    code.push_str(&Self::gen_stmt(s, req_name, &inner));
                }
                code.push_str(&format!("{}}}\n", indent));

                for (elif_cond, elif_body) in elif_branches {
                    let econd = Self::gen_val(elif_cond, req_name);
                    code.push_str(&format!("{}else if ({}) {{\n", indent, econd));
                    for s in elif_body {
                        code.push_str(&Self::gen_stmt(s, req_name, &inner));
                    }
                    code.push_str(&format!("{}}}\n", indent));
                }

                if let Some(else_stmts) = else_body {
                    code.push_str(&format!("{}else {{\n", indent));
                    for s in else_stmts {
                        code.push_str(&Self::gen_stmt(s, req_name, &inner));
                    }
                    code.push_str(&format!("{}}}\n", indent));
                }
            }

            Stmt::ForIn { var, iterable, body } => {
                let iter_val = Self::gen_val(iterable, req_name);
                code.push_str(&format!("{}for (const {} of {}) {{\n", indent, var, iter_val));
                for s in body {
                    code.push_str(&Self::gen_stmt(s, req_name, &inner));
                }
                code.push_str(&format!("{}}}\n", indent));
            }

            Stmt::While { condition, body } => {
                let cond = Self::gen_val(condition, req_name);
                code.push_str(&format!("{}while ({}) {{\n", indent, cond));
                for s in body {
                    code.push_str(&Self::gen_stmt(s, req_name, &inner));
                }
                code.push_str(&format!("{}}}\n", indent));
            }

            Stmt::Break => {
                code.push_str(&format!("{}break;\n", indent));
            }

            Stmt::Continue => {
                code.push_str(&format!("{}continue;\n", indent));
            }

            Stmt::Func { name, args, body } => {
                code.push_str(&format!("{}async function {}({}) {{\n", indent, name, args.join(", ")));
                for s in body {
                    code.push_str(&Self::gen_stmt(s, req_name, &inner));
                }
                code.push_str(&format!("{}}}\n", indent));
            }

            Stmt::Return(opt_expr) => {
                if let Some(expr) = opt_expr {
                    let val = Self::gen_val(expr, req_name);
                    code.push_str(&format!("{}return {};\n", indent, val));
                } else {
                    code.push_str(&format!("{}return;\n", indent));
                }
            }

            Stmt::Class { name, methods } => {
                code.push_str(&format!("{}class {} {{\n", indent, name));
                for method in methods {
                    if let Stmt::Func { name: m_name, args, body } = method {
                        let is_init = m_name == "init";
                        let js_name = if is_init { "constructor" } else { m_name.as_str() };
                        let async_kw = if is_init { "" } else { "async " };

                        code.push_str(&format!("{}  {}{}({}) {{\n", indent, async_kw, js_name, args.join(", ")));
                        for s in body {
                            code.push_str(&Self::gen_stmt(s, "this", &format!("{}    ", indent)));
                        }
                        code.push_str(&format!("{}  }}\n", indent));
                    }
                }
                code.push_str(&format!("{}}}\n", indent));
            }

            Stmt::Try { body, except_var, except_body } => {
                code.push_str(&format!("{}try {{\n", indent));
                for s in body {
                    code.push_str(&Self::gen_stmt(s, req_name, &inner));
                }
                let err_var = except_var.clone().unwrap_or_else(|| "_err".to_string());
                code.push_str(&format!("{}}} catch ({}) {{\n", indent, err_var));
                for s in except_body {
                    code.push_str(&Self::gen_stmt(s, req_name, &inner));
                }
                code.push_str(&format!("{}}}\n", indent));
            }

            Stmt::Import { path, alias } => {
                let import_path = if path.ends_with(".hb") {
                    path.replace(".hb", ".js")
                } else {
                    path.clone()
                };
                if let Some(name) = alias {
                    code.push_str(&format!("{}const {} = require(\"{}\");\n", indent, name, import_path));
                } else {
                    code.push_str(&format!("{}require(\"{}\");\n", indent, import_path));
                }
            }

            Stmt::FromImport { path, names } => {
                let import_path = if path.ends_with(".hb") {
                    path.replace(".hb", ".js")
                } else {
                    path.clone()
                };
                let names_str = names.join(", ");
                code.push_str(&format!("{}const {{ {} }} = require(\"{}\");\n", indent, names_str, import_path));
            }

            Stmt::Export(inner_stmt) => {
                code.push_str(&Self::gen_stmt(inner_stmt, req_name, indent));
                match &**inner_stmt {
                    Stmt::Func { name, .. } => {
                        code.push_str(&format!("{}module.exports.{} = {};\n", indent, name, name));
                    }
                    Stmt::Class { name, .. } => {
                        code.push_str(&format!("{}module.exports.{} = {};\n", indent, name, name));
                    }
                    Stmt::Set { target, .. } => {
                        if let Expr::Ident(name) = target {
                            code.push_str(&format!("{}module.exports.{} = {};\n", indent, name, name));
                        }
                    }
                    _ => {}
                }
            }

            // ─── Harbor-specific ───

            Stmt::Server { port, routes } => {
                code.push_str(&Self::gen_server(port, routes, indent));
            }

            Stmt::Respond { status, value } => {
                if let Some(status_code) = status {
                    code.push_str(&format!("{}__res.statusCode = {};\n", indent, status_code));
                }
                let val = Self::gen_val(value, req_name);
                code.push_str(&format!("{}const __val = {};\n", indent, val));
                code.push_str(&format!("{}if (typeof __val === 'object' && __val !== null) {{\n", indent));
                code.push_str(&format!("{}  __res.setHeader('Content-Type', 'application/json');\n", indent));
                code.push_str(&format!("{}  __res.end(JSON.stringify(__val));\n", indent));
                code.push_str(&format!("{}}} else {{\n", indent));
                code.push_str(&format!("{}  __res.end(String(__val));\n", indent));
                code.push_str(&format!("{}}}\n", indent));
                code.push_str(&format!("{}return;\n", indent));
            }

            Stmt::Fetch { url, body } => {
                let url_val = Self::gen_val(url, req_name);
                code.push_str(&format!("{}const fetch_res = await fetchJson({});\n", indent, url_val));
                code.push_str(&format!("{}{{\n", indent));
                code.push_str(&format!("{}  const res = fetch_res;\n", indent));
                for s in body {
                    code.push_str(&Self::gen_stmt(s, req_name, &inner));
                }
                code.push_str(&format!("{}}}\n", indent));
            }
        }

        code
    }

    // ─── Server & Route Generation ───

    fn gen_server(port: &Expr, routes: &[Route], indent: &str) -> String {
        let mut code = String::new();
        let port_val = Self::gen_val(port, "null");

        code.push_str(&format!("{}const server = http.createServer(async (req, __res) => {{\n", indent));

        for route in routes {
            code.push_str(&Self::gen_route(route, indent));
        }

        code.push_str(&format!("{}  __res.statusCode = 404;\n", indent));
        code.push_str(&format!("{}  __res.end(\"Not Found\");\n", indent));
        code.push_str(&format!("{}}});\n\n", indent));

        code.push_str(&format!("{}server.listen({}, () => {{\n", indent, port_val));
        code.push_str(&format!("{}  console.log(`Harbor server running on http://127.0.0.1:${{{}}}`); \n", indent, port_val));
        code.push_str(&format!("{}}});\n", indent));

        code
    }

    fn gen_route(route: &Route, base_indent: &str) -> String {
        let mut code = String::new();
        let indent = format!("{}  ", base_indent);
        let inner = format!("{}  ", indent);

        let has_params = route.path.contains(':');

        if has_params {
            let mut re_parts = Vec::new();
            for part in route.path.split('/') {
                if part.starts_with(':') {
                    re_parts.push("([^/]+)".to_string());
                } else if !part.is_empty() {
                    re_parts.push(part.replace(".", "\\."));
                }
            }
            let re_path = format!("^/{}$", re_parts.join("/"));
            let var_name = format!("match_{}_{}", route.method.to_lowercase(),
                route.path.replace("/", "_").replace(":", ""));

            code.push_str(&format!("{}const {} = req.url.match(/{}/);\n", indent, var_name,
                re_path.replace("/", "\\/")));
            code.push_str(&format!("{}if ({} && req.method === \"{}\") {{\n", indent, var_name, route.method));

            code.push_str(&format!("{}req.params = {{}};\n", inner));
            let mut param_idx = 1;
            for part in route.path.split('/') {
                if part.starts_with(':') {
                    let param_name = &part[1..];
                    code.push_str(&format!("{}req.params[\"{}\"] = {}[{}];\n", inner, param_name, var_name, param_idx));
                    param_idx += 1;
                }
            }
        } else {
            code.push_str(&format!("{}if (req.url === \"{}\" && req.method === \"{}\") {{\n",
                indent, route.path, route.method));
        }

        if route.method != "GET" {
            code.push_str(&format!("{}req.body = await parseJsonBody(req);\n", inner));
        }

        for stmt in &route.body {
            code.push_str(&Self::gen_stmt(stmt, "req", &inner));
        }

        code.push_str(&format!("{}}}\n\n", indent));
        code
    }

    // ─── Expression Code Generation ───

    fn gen_val(expr: &Expr, req_name: &str) -> String {
        match expr {
            Expr::String(s) => format!("\"{}\"", s),

            Expr::FString(parts) => {
                let mut s = String::from("`");
                for part in parts {
                    match part {
                        FStringExprPart::Literal(text) => s.push_str(text),
                        FStringExprPart::Expression(expr) => {
                            s.push_str("${");
                            s.push_str(&Self::gen_val(expr, req_name));
                            s.push('}');
                        }
                    }
                }
                s.push('`');
                s
            }

            Expr::Number(n) => {
                if *n == (*n as i64) as f64 && n.is_finite() {
                    format!("{}", *n as i64)
                } else {
                    n.to_string()
                }
            }

            Expr::Bool(b) => b.to_string(),

            Expr::None => "null".to_string(),

            Expr::Ident(name) => {
                if name == "req" && req_name != "null" {
                    req_name.to_string()
                } else if name == "res" {
                    "res".to_string()
                } else {
                    name.clone()
                }
            }

            Expr::Member(obj, field) => {
                let obj_code = Self::gen_val(obj, req_name);
                if obj_code == req_name && req_name != "null" {
                    match field.as_str() {
                        "path" => format!("{}.url", req_name),
                        "method" => format!("{}.method", req_name),
                        "params" => format!("{}.params", req_name),
                        "body" => format!("{}.body", req_name),
                        "header" | "headers" => format!("{}.headers", req_name),
                        _ => format!("{}.{}", req_name, field),
                    }
                } else if obj_code == "res" {
                    match field.as_str() {
                        "body" => "res.body".to_string(),
                        "status" => "res.statusCode".to_string(),
                        _ => format!("res.{}", field),
                    }
                } else if obj_code.ends_with(".headers") {
                    format!("{}['{}']", obj_code, field.to_lowercase())
                } else {
                    format!("{}.{}", obj_code, field)
                }
            }

            Expr::Object(fields) => {
                let mut obj_code = String::from("{");
                for (i, (key, value)) in fields.iter().enumerate() {
                    if i > 0 { obj_code.push_str(", "); }
                    obj_code.push_str(&format!("\"{}\": {}", key, Self::gen_val(value, req_name)));
                }
                obj_code.push('}');
                obj_code
            }

            Expr::Array(elements) => {
                let mut arr_code = String::from("[");
                for (i, el) in elements.iter().enumerate() {
                    if i > 0 { arr_code.push_str(", "); }
                    arr_code.push_str(&Self::gen_val(el, req_name));
                }
                arr_code.push(']');
                arr_code
            }

            Expr::Binary(left, op, right) => {
                let l = Self::gen_val(left, req_name);
                let r = Self::gen_val(right, req_name);
                match op.as_str() {
                    "and" => format!("({} && {})", l, r),
                    "or" => format!("({} || {})", l, r),
                    "in" => format!("__contains({}, {})", r, l),
                    "not in" => format!("!__contains({}, {})", r, l),
                    "**" => format!("Math.pow({}, {})", l, r),
                    "//" => format!("Math.floor({} / {})", l, r),
                    _ => format!("({} {} {})", l, op, r),
                }
            }

            Expr::Unary(op, right) => {
                let r = Self::gen_val(right, req_name);
                match op.as_str() {
                    "not" => format!("(!{})", r),
                    _ => format!("({}{})", op, r),
                }
            }

            Expr::Index(obj, idx) => {
                format!("{}[{}]",
                    Self::gen_val(obj, req_name),
                    Self::gen_val(idx, req_name))
            }

            Expr::Call(func, args) => {
                let func_code = Self::gen_val(func, req_name);
                let args_strs: Vec<String> = args.iter()
                    .map(|a| Self::gen_val(a, req_name))
                    .collect();
                let args_code = args_strs.join(", ");

                // PascalCase detection: class instantiation (no 'new' keyword needed)
                if let Expr::Ident(name) = &**func {
                    if name.chars().next().map_or(false, |c| c.is_uppercase()) {
                        return format!("new {}({})", func_code, args_code);
                    }
                }

                format!("(await {}({}))", func_code, args_code)
            }
        }
    }
}
