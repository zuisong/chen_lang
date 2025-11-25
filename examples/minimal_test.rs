fn main() {
    tracing_subscriber::fmt::init();

    let code = r#"
def func(){
    123
}
let x = func()
println(x)
"#
    .to_string();

    chen_lang::run(code).unwrap();
}