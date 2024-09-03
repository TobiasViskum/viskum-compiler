use std::time::Instant;

use ast_arena::AstArena;
use parser::Parser;

pub struct Compiler {}

impl Compiler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compile_entry(&self) {
        let file_content = Self::get_file_content();

        {
            let ast_arena = AstArena::new();
            let ast = {
                let mut parser = Parser::new(file_content.as_str(), &ast_arena);

                parser.parse();
            };
        }
    }

    fn get_file_content() -> String {
        let mut args = std::env::args();
        args.next();

        if let Some(input_file) = args.next() {
            match std::fs::read_to_string(input_file) {
                Ok(file_content) => file_content,
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                    std::process::exit(1);
                }
            }
        } else {
            println!("Missing file input");
            std::process::exit(1);
        }
    }
}
