mod compiler;

use crate::parser;
use crate::value::Value;
use crate::vm::{Program, VM};

pub use compiler::{compile_ast, TinyCompileError};

pub fn compile(source: &str) -> Result<Program, TinyCompileError> {
    let ast = parser::parse_from_source(source)?;
    compile_ast(&ast)
}

pub fn compile_with_ast(source: &str) -> Result<(crate::expression::Ast, Program), TinyCompileError> {
    let ast = parser::parse_from_source(source)?;
    let program = compile_ast(&ast)?;
    Ok((ast, program))
}

pub fn run(source: &str) -> Result<Value, TinyCompileError> {
    let program = compile(source)?;
    let mut vm = VM::new();
    let result = vm.execute(&program)?;
    Ok(result)
}


#[cfg(test)]
mod tests {
    use crate::tiny_compiler::{compile, compile_with_ast};

    #[test]
    fn test1(){
        compile_with_ast("1 + 2").unwrap();
    }

    #[test]
    fn test2(){
        compile(r#"
        if 1 < 2 {
            return 4
         }"
        "#).unwrap();
    }

}
