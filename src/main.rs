use compiler::Compiler;

fn main() {
    let result = {
        let compiler = Compiler::new();
        compiler.compile_entry()
    };
}
