use crate::compiler::compile;
use crate::parser::parse_from_source;
use crate::vm::VM;

fn run_code(code: &str) -> String {
    let ast = parse_from_source(code).unwrap();
    let program = compile(&code.chars().collect::<Vec<_>>(), ast);
    let mut vm = VM::new();
    let result = vm.execute(&program);
    match result {
        Ok(val) => val.to_string(),
        Err(e) => format!("Error: {}", e),
    }
}

#[test]
fn test_simple_generator() {
    let code = r#"
    function* count() {
        yield 1
        yield 2
        yield 3
    }
    let g = count()
    let res = []
    res.push(g.next().value)
    res.push(g.next().value)
    res.push(g.next().value)
    res.push(g.next().done)
    return JSON.stringify(res)
    "#;
    assert_eq!(run_code(code), "[1,2,3,true]");
}

#[test]
fn test_generator_for_of() {
    let code = r#"
    function* fib(n) {
        let a = 0
        let b = 1
        let i = 0
        while (i < n) {
            yield a
            let temp = a
            a = b
            b = temp + b
            i = i + 1
        }
    }
    let res = []
    for (let x of fib(5)) {
        res.push(x)
    }
    return JSON.stringify(res)
    "#;
    assert_eq!(run_code(code), "[0,1,1,2,3]");
}
