fn main() {
    tracing_subscriber::fmt::init();

    let code = r#"
def test(){
    println("hello")
    42
}
let x = test()
println("done")
"#
    .to_string();

    chen_lang::run(code).unwrap();
}