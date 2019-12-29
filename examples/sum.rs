#![feature(box_syntax, box_patterns)]

fn main() {
    simple_logger::init().unwrap();

    let code = r#"
# 这里是注释,
# 注释以# 开始, 直到行末
def aaa(n){

    let i = 0
    let sum = 0
    for (i <=n) {
       sum = sum + i
        i = i+1
    }
    println(sum)
    sum
}
let sum = 0
sum = aaa(100)
println(sum)
"#
    .to_string();

    chen_lang::run(code).unwrap();
}
