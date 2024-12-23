use std::{ cell::RefCell, path::{ self, PathBuf } };

use ast::{ AstArena, AstPrettifier };
use bumpalo::Bump;
use codegen::CodeGen;
use icfg::Icfg;
use icfg_builder::IcfgBuilder;
use ir::{ GlobalMem, ModId, ResolvedInformation };
use parser::Parser;
use resolver::Resolver;

pub struct Compiler {
    entry_dir: PathBuf,
    entry_file: PathBuf,
}

impl Compiler {
    pub fn new() -> Self {
        let mut args = std::env::args();
        args.next();

        let mut input_file = if let Some(input_file) = args.next() {
            path::Path::new(&input_file).to_path_buf()
        } else {
            println!("Missing file input");
            std::process::exit(1);
        };

        if let Some(ext) = input_file.extension() {
            if ext != "vs" {
                println!("File must end with .vs");
                std::process::exit(1);
            }
        } else {
            println!("Must be a .vs file");
            std::process::exit(1);
        }

        // Get last part of the path
        let entry_file = PathBuf::from(input_file.file_name().unwrap().to_str().unwrap());
        input_file.pop();
        let entry_dir = input_file;

        Self { entry_file, entry_dir }
    }

    pub fn compile_entry(&self) {
        let now = std::time::Instant::now();

        let arena = Bump::new();
        let global_mems = RefCell::new(Vec::new());
        let icfg = self.compile_icfg(&arena, &global_mems);

        // IcfgPrettifier::new(&icfg).print_icfg();

        println!("Viskum compilation took: {:?}", now.elapsed());

        let now = std::time::Instant::now();
        CodeGen::new(&icfg).gen_code(&self.entry_file.as_os_str().to_str().unwrap());
        println!("LLVM compilation took: {:?}", now.elapsed());
    }

    fn compile_icfg<'a>(
        &self,
        arena: &'a Bump,
        global_mems: &'a RefCell<Vec<GlobalMem>>
    ) -> Icfg<'a> {
        let file_content = match
            std::fs::read_to_string(&self.entry_dir.join(&self.entry_file).with_extension("vs"))
        {
            Ok(file_content) => file_content,
            Err(e) => {
                eprintln!("Error reading file: {}", e);
                std::process::exit(1);
            }
        };
        let ast_arena = AstArena::new();

        let (resolved, ast) = {
            let mut resolver = Resolver::new(arena, global_mems);

            resolver.resolve_all_modules(&self.entry_dir, &self.entry_file);

            let mut parser = Parser::new(&file_content, &ast_arena, ModId(0));

            let ast = parser.parse_ast();
            let (forward_declared_ast, lexical_relations) = resolver.forward_declare_ast(ast);
            let resolved_ast = resolver.resolve_ast(forward_declared_ast, &lexical_relations);
            let type_checked_ast = resolver.type_check_ast(resolved_ast, &lexical_relations);

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

    // fn get_file_content(&self) -> String {
    //     match std::fs::read_to_string(&format!("{}.vs", self.file)) {
    //         Ok(file_content) => file_content,
    //         Err(e) => {
    //             eprintln!("Error reading file: {}", e);
    //             std::process::exit(1);
    //         }
    //     }
    // }
}
