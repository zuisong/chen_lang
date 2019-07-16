fn main() {
    simple_logger::init().unwrap();
    let cpde: String = r#"

let r = 0
let d1 = 0
let d2 = 0
def fibo(n){
    println(n)
    n
}
r = fibo(6)
println(r)

"#
    .to_string();

    chen_lang::run(cpde).unwrap();
}
