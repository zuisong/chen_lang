fn main() {
    tracing_subscriber::fmt::init();

    let code = r#"
# 这里是注释,
# 注释以# 开始, 直到行末
def aaa(n){
    let i = 100
    let sum = 0
    for i!=0 {
        i = i - 1
        # 这里有相对复杂的逻辑运算
        if (i%2!=0) || (i%3==0)  {
            println(i)
            # 打印出来的 i 都是奇数 或者是能被三整除的偶数
            sum = sum + i
        }
    }
    # sum 为 100以为的奇数之和
    println("100以内的 奇数或者是能被三整除的偶数 之和是")
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
