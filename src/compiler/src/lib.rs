use ast::AstArena;
use parser::Parser;
use resolver::Resolver;

pub struct Compiler {}

impl Compiler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compile_entry(&self) {
        let file_content = Self::get_file_content();

        let ast_arena = AstArena::new();

        let ast = {
            let parser = Parser::new(file_content.as_str(), &ast_arena);
            let ast = parser.parse_into_ast();

            let (mut resolver, ast) = Resolver::from_ast(&file_content, ast);
            let resolved_ast = resolver.resolve_ast(ast);
            let type_checked_ast = resolver.type_check_ast(resolved_ast);

            type_checked_ast
        };
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
