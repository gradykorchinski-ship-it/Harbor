mod lexer;
mod parser;
mod ast;
mod codegen;

use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.contains(&"--version".to_string()) {
        println!("Harbor v2.0.0");
        return;
    }

    if args.len() < 2 || args.contains(&"--help".to_string()) {
        println!("Harbor v2.0.0");
        println!("Usage: harbor <input.hb> [-o output.js]");
        println!("\nFlags:");
        println!("  --help      Show this help");
        println!("  --version   Show version information");
        println!("  -o <path>   Specify output file (default: output.js)");
        return;
    }

    if args[1] == "doc" {
        if args.len() < 3 {
             println!("Usage: harbor doc <file.hb>");
             return;
        }
        let input_path = &args[2];
        let src = match fs::read_to_string(input_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error: Could not read file '{}': {}", input_path, e);
                std::process::exit(1);
            }
        };
        
        let mut lexer = lexer::Lexer::new(&src);
        let tokens = lexer.tokenize();
        let mut parser = parser::Parser::new(tokens);
        let ast = parser.parse();
        
        println!("Documentation for {}:", input_path);
        println!("--------------------------------");
        for stmt in ast {
            match stmt {
                ast::Stmt::Func { name, args, .. } => {
                    println!("def {}({})", name, args.join(", "));
                }
                ast::Stmt::Class { name, methods } => {
                    println!("class {}:", name);
                    for method in methods {
                        if let ast::Stmt::Func { name: m_name, args: m_args, .. } = method {
                             println!("    def {}({})", m_name, m_args.join(", "));
                        }
                    }
                }
                ast::Stmt::Export(inner) => {
                     match *inner {
                        ast::Stmt::Func { name, args, .. } => {
                            println!("export def {}({})", name, args.join(", "));
                        }
                        ast::Stmt::Class { name, methods } => {
                            println!("export class {}:", name);
                            for method in methods {
                                if let ast::Stmt::Func { name: m_name, args: m_args, .. } = method {
                                     println!("    def {}({})", m_name, m_args.join(", "));
                                }
                            }
                        }
                        _ => {}
                     }
                }
                _ => {}
            }
        }
        println!("--------------------------------");
        return;
    }

    // Check for run mode (no -o flag)
    let is_run_mode = !args.iter().any(|a| a == "-o");
    
    let input_path = &args[1];
    let mut output_path = if is_run_mode {
        // If running, create adjacent .js file
        let path = std::path::Path::new(input_path);
        let file_stem = path.file_stem().unwrap().to_str().unwrap();
        // Use a hidden file to avoid clutter? or just .js?
        // Node requires .js extension usually.
        // Let's use same dir.
        let mut out = path.to_path_buf();
        out.set_file_name(format!("{}.js", file_stem));
        out.to_str().unwrap().to_string()
    } else {
        "output.js".to_string()
    };

    if let Some(pos) = args.iter().position(|r| r == "-o") {
        if pos + 1 < args.len() {
            output_path = args[pos + 1].clone();
        }
    }

    let src = match fs::read_to_string(input_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: Could not read file '{}': {}", input_path, e);
            std::process::exit(1);
        }
    };

    // Tokenize
    let mut lexer = lexer::Lexer::new(&src);
    let tokens = lexer.tokenize();

    // Parse
    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse();

    // Generate Code
    let js_code = codegen::CodeGen::generate(&ast);

    // Save Output
    match fs::write(&output_path, js_code) {
        Ok(_) => {
            if !is_run_mode {
                println!("─────────────────────────────────────────");
                println!("  � Harbor Compilation Successful!");
                println!("  Input:  {}", input_path);
                println!("  Output: {}", output_path);
                println!("─────────────────────────────────────────");
            } else {
                 // Run it!
                 let status = std::process::Command::new("node")
                    .arg(&output_path)
                    .status();
                 
                 // Cleanup
                 // let _ = fs::remove_file(&output_path); // Optional: Keep it for debugging or remove?
                 // The user might expect the .js file to be there if they imported it elsewhere.
                 // Actually, for imports to work, the imported files MUST exist as .js.
                 // So if I run main_import.hb, and it imports utils.js, utils.js must exist.
                 // So we should probably NOT delete it, or at least be careful.
                 // For now, let's keep it.
                 
                 match status {
                     Ok(s) => {
                         if !s.success() {
                             std::process::exit(s.code().unwrap_or(1));
                         }
                     }
                     Err(e) => {
                         eprintln!("Error: Could not run node: {}", e);
                         std::process::exit(1);
                     }
                 }
            }
        }
        Err(e) => {
            eprintln!("Error: Could not write to '{}': {}", output_path, e);
            std::process::exit(1);
        }
    }
}
