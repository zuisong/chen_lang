fn main() {
    tracing_subscriber::fmt::init();

    let code = r#"
def func(){
    return 123
}
let x = 1
x =  func()
println(x)
"#
    .to_string();

    chen_lang::run(code).unwrap();
}