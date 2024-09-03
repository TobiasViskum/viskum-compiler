use compiler::Compiler;

fn main() {
    let _result = {
        let compiler = Compiler::new();
        compiler.compile_entry()
    };
}
