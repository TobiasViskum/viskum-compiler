use std::cell::RefCell;

use ast::{ AstArena, AstPrettifier };
use bumpalo::Bump;
use codegen::CodeGen;
use icfg::Icfg;
use icfg_builder::IcfgBuilder;
use ir::{ GlobalMem, ResolvedInformation };
use parser::Parser;
use resolver::Resolver;

pub struct Compiler {
    file: String,
}

impl Compiler {
    pub fn new() -> Self {
        let mut args = std::env::args();
        args.next();

        let input_file = if let Some(input_file) = args.next() {
            input_file
        } else {
            println!("Missing file input");
            std::process::exit(1);
        };

        // Ends with .vs

        let input_file = if !input_file.ends_with(".vs") {
            println!("File must end with .vs");
            std::process::exit(1);
        } else {
            input_file.replace(".vs", "")
        };

        Self { file: input_file }
    }

    pub fn compile_entry(&self) {
        let now = std::time::Instant::now();

        let arena = Bump::new();
        let global_mems = RefCell::new(Vec::new());
        let icfg = self.build_to_icfg(&arena, &global_mems);

        // IcfgPrettifier::new(&icfg).print_icfg();

        println!("Viskum compilation took: {:?}", now.elapsed());

        let now = std::time::Instant::now();
        CodeGen::new(&icfg).gen_code(&self.file);
        println!("LLVM compilation took: {:?}", now.elapsed());
    }

    fn build_to_icfg<'a>(
        &self,
        arena: &'a Bump,
        global_mems: &'a RefCell<Vec<GlobalMem>>
    ) -> Icfg<'a> {
        let file_content = self.get_file_content();
        let ast_arena = AstArena::new();

        let (resolved, ast) = {
            let parser = Parser::new(&file_content, &ast_arena);

            let (mut resolver, ast) = Resolver::from_ast(
                &file_content,
                parser.parse_into_ast(),
                &arena,
                global_mems
            );

            // AstPrettifier::new(&ast, &file_content, None).print_ast();

            let resolved_ast = resolver.resolve_ast(ast);
            let type_checked_ast = resolver.type_check_ast(resolved_ast);

            (resolver.take_resolved_information(), type_checked_ast)
        };

        // AstPrettifier::new(
        //     &ast,
        //     &file_content,
        //     Some(&resolved_information.node_id_to_ty)
        // ).print_ast();

        let icfg_builder = IcfgBuilder::new(ast, resolved.1, global_mems, &file_content, arena);
        let icfg = icfg_builder.build(resolved.0);
        icfg
    }

    fn get_file_content(&self) -> String {
        match std::fs::read_to_string(&format!("{}.vs", self.file)) {
            Ok(file_content) => file_content,
            Err(e) => {
                eprintln!("Error reading file: {}", e);
                std::process::exit(1);
            }
        }
    }
}
