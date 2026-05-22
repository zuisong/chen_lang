use chen_lang::tokenizer::tokenizer;

#[test]
fn debug_parser() {
    let code = "let custom = { [Symbol.iterator]: function() {} }";
    let tokens = tokenizer(code.to_string()).unwrap();
    println!("Tokens:");
    for t in &tokens {
        println!("{:?}", t);
    }
}
