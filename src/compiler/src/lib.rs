use ast::{ AstArena, AstPrettifier };
use bumpalo::Bump;
use codegen::CodeGen;
use icfg::{ Icfg, IcfgPrettifier };
use icfg_builder::IcfgBuilder;
use ir::ResolvedInformation;
use parser::Parser;
use resolver::Resolver;

pub struct Compiler;

impl Compiler {
    pub fn new() -> Self {
        Self
    }

    pub fn compile_entry(&self) {
        let now = std::time::Instant::now();

        let arena = Bump::new();
        let (icfg, resolved_information) = self.build_to_icfg(&arena);

        // IcfgPrettifier::new(&icfg).print_icfg();

        println!("Viskum compilation took: {:?}", now.elapsed());

        let now = std::time::Instant::now();
        CodeGen::new(&icfg).gen_code("file", &resolved_information);
        println!("LLVM compilation took: {:?}", now.elapsed());
    }

    fn build_to_icfg<'a>(&self, arena: &'a Bump) -> (Icfg, ResolvedInformation<'a>) {
        let file_content = Self::get_file_content();
        let ast_arena = AstArena::new();

        let (resolved, ast) = {
            let parser = Parser::new(&file_content, &ast_arena);

            let (mut resolver, ast) = Resolver::from_ast(
                &file_content,
                parser.parse_into_ast(),
                &arena
            );

            let resolved_ast = resolver.resolve_ast(ast);
            let type_checked_ast = resolver.type_check_ast(resolved_ast);

            (resolver.take_resolved_information(), type_checked_ast)
        };

        // AstPrettifier::new(
        //     &ast,
        //     &file_content,
        //     Some(&resolved_information.node_id_to_ty)
        // ).print_ast();

        let icfg_builder = IcfgBuilder::new(ast, &resolved.1, &file_content);
        let icfg = icfg_builder.build(resolved.0);
        (icfg, resolved.1)
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
