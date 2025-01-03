use std::{ path::{ self, PathBuf }, sync::Mutex };

use ast::{ Ast, AstArena, AstArenaObject, AstUnvalidated, VisitAst };
use bumpalo::Bump;
use codegen::CodeGen;

use diagnostics::set_mode_id_to_file_path;
use icfg::Icfg;
use icfg_builder::IcfgBuilder;
use ir::ModId;
use parser::Parser;
use resolver::Resolver;
use threadpool::ThreadPool;
use threadpool_scope::scope_with;

pub struct Compiler {
    entry_dir: PathBuf,
    entry_file: PathBuf,
    threadpool: ThreadPool,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        let mut args = std::env::args();
        args.next();

        let workers_amount = std::thread
            ::available_parallelism()
            .map(|x| x.get())
            .unwrap_or(1);

        let threadpool = ThreadPool::new(workers_amount);

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
        let entry_file = input_file.clone();
        input_file.pop();
        let entry_dir = input_file;

        Self { entry_file, entry_dir, threadpool }
    }

    pub fn compile_entry(&self) {
        let now = std::time::Instant::now();

        let arena = Bump::new();
        let icfg = self.compile_icfg(&arena);

        // IcfgPrettifier::new(&icfg).print_icfg();

        println!("Viskum compilation took: {:?}", now.elapsed());

        let now = std::time::Instant::now();
        CodeGen::new(&icfg, &self.threadpool).gen_code(
            self.entry_file.as_os_str().to_str().unwrap()
        );
        println!("LLVM compilation took: {:?}", now.elapsed());
    }

    pub fn parse_all_files_in_package<'ast>(
        &self,
        ast_arena: &'ast AstArena
    ) -> (Vec<(Ast<'ast, AstUnvalidated>, String, ModId)>, ModId) {
        let files = match std::fs::read_dir(&self.entry_dir) {
            Ok(files) => {
                files
                    .filter_map(|file| {
                        let file = file.unwrap();
                        let path = file.path();
                        if let Some(ext) = path.extension() {
                            if ext == "vs" { Some(path) } else { None }
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            }
            Err(e) => {
                eprintln!("Error reading directory: {}", e);
                std::process::exit(1);
            }
        };

        let entry_file_id = files
            .iter()
            .enumerate()
            .find_map(|(i, file)| {
                if file == &self.entry_file { Some(ModId(i as u32)) } else { None }
            })
            .expect("Cannot find entry file in directory");

        let asts = Mutex::new(Vec::with_capacity(files.len()));

        scope_with(&self.threadpool, |s| {
            let mut next_mod_id: u32 = 0;

            for file in files {
                let mod_id = ModId(next_mod_id);
                let asts_ref = &asts;

                set_mode_id_to_file_path(mod_id, file.clone());

                s.execute(move || {
                    let bump = ast_arena.get();
                    let (ast, file_content) = self.parse_file(file, bump, mod_id);

                    asts_ref.lock().unwrap().push((ast, file_content, mod_id));
                });

                next_mod_id += 1;
            }
        });

        (asts.into_inner().unwrap(), entry_file_id)
    }

    fn parse_file<'ast>(
        &self,
        path: PathBuf,
        ast_arena: AstArenaObject<'ast>,
        mod_id: ModId
    ) -> (Ast<'ast, AstUnvalidated>, String) {
        let file_content = match std::fs::read_to_string(&path) {
            Ok(file_content) => file_content,
            Err(e) => {
                eprintln!("Error reading file: {}", e);
                std::process::exit(1);
            }
        };

        let parser = Parser::new(&file_content, &ast_arena, mod_id);

        let (parsed_ast, diagnostics) = parser.parse_ast();

        if !diagnostics.is_empty() {
            diagnostics::report_diagnostics(diagnostics);
        }

        (parsed_ast, file_content)
    }

    fn compile_icfg<'a>(&self, arena: &'a Bump) -> Icfg<'a> {
        let ast_arena = AstArena::new();

        let (resolved_functions, resolved_information) = {
            let now = std::time::Instant::now();
            let (asts, _) = self.parse_all_files_in_package(&ast_arena);
            println!("Parsing took: {:?}", now.elapsed());

            let now = std::time::Instant::now();

            let total_nodes = asts
                .iter()
                .map(|(ast, _, _)| ast.metadata.node_count)
                .sum::<usize>();

            let total_def_count = asts
                .iter()
                .map(|(ast, _, _)| ast.metadata.def_count)
                .sum::<usize>();

            let mut resolver = Resolver::new(
                arena,
                total_nodes,
                total_def_count /*, global_mems */
            );

            println!("Setting up resolver took: {:?}", now.elapsed());
            let now = std::time::Instant::now();

            let asts_count = asts.len();

            let ast_pre_resolve_visit_results = {
                let ast_visit_results = Mutex::new(Vec::with_capacity(asts_count));
                let ast_visit_results_ref = &ast_visit_results;

                let global_visit_results = Mutex::new(Vec::with_capacity(asts_count));
                let global_visit_results_ref = &global_visit_results;

                scope_with(&self.threadpool, |s| {
                    let resolver_handle = &resolver;
                    for (ast, _, _) in asts {
                        s.execute(move || {
                            let (ast, global_visit_result, local_visit_result) = ast
                                .into_visitor(resolver_handle)
                                .visit();

                            global_visit_results_ref.lock().unwrap().push(global_visit_result);
                            ast_visit_results_ref.lock().unwrap().push((ast, local_visit_result));
                        });
                    }
                });

                // let merged_results = global_visit_results
                //     .into_inner()
                //     .unwrap()
                //     .into_iter()
                //     .fold(MergedResults::new(), |mut acc, result| {
                //         acc.merge(result);
                //         acc
                //     });

                for global_visit_result in global_visit_results.into_inner().unwrap() {
                    resolver.use_visit_result_from_pre_resolve(global_visit_result);
                }

                ast_visit_results.into_inner().unwrap()
            };

            println!("Pre-resolving took: {:?}", now.elapsed());
            let now = std::time::Instant::now();

            let ast_resolve_visit_results = {
                let ast_visit_results = Mutex::new(Vec::with_capacity(asts_count));
                let ast_visit_results_ref = &ast_visit_results;

                let global_visit_results = Mutex::new(Vec::with_capacity(asts_count));
                let global_visit_results_ref = &global_visit_results;

                scope_with(&self.threadpool, |s| {
                    let resolver_handle = &resolver;
                    for (ast, local_visit_result) in ast_pre_resolve_visit_results {
                        s.execute(move || {
                            let (ast, global_visit_result, local_visit_result) = ast
                                .into_visitor(resolver_handle, local_visit_result)
                                .visit();

                            global_visit_results_ref.lock().unwrap().push(global_visit_result);
                            ast_visit_results_ref.lock().unwrap().push((ast, local_visit_result));
                        });
                    }
                });

                for global_visit_result in global_visit_results.into_inner().unwrap() {
                    resolver.use_visit_result_from_resolve(global_visit_result);
                }

                ast_visit_results.into_inner().unwrap()
            };

            println!("Resolving took: {:?}", now.elapsed());
            let now = std::time::Instant::now();

            let _ = {
                let ast_visit_results = Mutex::new(Vec::with_capacity(asts_count));
                let ast_visit_results_ref = &ast_visit_results;

                let global_visit_results = Mutex::new(Vec::with_capacity(asts_count));
                let global_visit_results_ref = &global_visit_results;

                scope_with(&self.threadpool, |s| {
                    let resolver_handle = &resolver;
                    for (ast, local_visit_result) in ast_resolve_visit_results {
                        s.execute(move || {
                            let (ast, global_visit_result, local_visit_result) = ast
                                .into_visitor(resolver_handle, local_visit_result)
                                .visit();

                            global_visit_results_ref.lock().unwrap().push(global_visit_result);
                            ast_visit_results_ref.lock().unwrap().push((ast, local_visit_result));
                        });
                    }
                });

                for global_visit_result in global_visit_results.into_inner().unwrap() {
                    resolver.use_visit_result_from_type_check(global_visit_result);
                }

                ast_visit_results.into_inner().unwrap()
            };

            println!("Type checking took: {:?}", now.elapsed());

            if diagnostics::has_error() {
                diagnostics::print_diagnostics();
                std::process::exit(1);
            }

            resolver.take_resolved_information()
        };

        let now = std::time::Instant::now();

        let icfg_builder = IcfgBuilder::new(resolved_information, &self.threadpool);
        let icfg = icfg_builder.build(resolved_functions);

        println!("Building ICFG took: {:?}", now.elapsed());
        icfg
    }
}
