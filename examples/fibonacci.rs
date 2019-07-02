fn main() {
    simple_logger::init().unwrap();
    let cpde: String = r#"

let r = 0
let d1 = 0
let d2 = 0
def fibo(n){
    let res = 0
    if n==1{
        res = 1
    }
    if n==2{
        res = 1
    }
    if n>2{
        d1 = fibo(n- 2)
        d2 = fibo(n- 1)
        res = d1 + d2
    }
    res
}
r = fibo(6)
println(r)

"#
    .to_string();

    chen_lang::run(cpde).unwrap();
}
