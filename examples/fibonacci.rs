fn main() {
    tracing_subscriber::fmt::init();
    let code: String = r#"

# 用 chen_lang 打印斐波那契数列前三十个数
let n = 1
let i = 1
let j = 2
let tmp = 0
for n <= 30 {
   println(i)
   tmp = i
   i = j
   j = tmp + j
   n = n + 1
}

"#
    .to_string();

    chen_lang::run(code).unwrap();
}
