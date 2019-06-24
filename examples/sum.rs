fn main() {
    simple_logger::init().unwrap();

    let code = r#"
sum = 0
# 这里是注释,
# 注释以# 开始, 直到行末
# if 和 for 里面的表达式运算结果都是int类型 0 为假  非0 为真
i = 100
sum = 0
for i {
    i = i - 1
    if i%2 {
        println(i)
# 打印出来的 i 都是奇数
        sum = sum + i
    }
}
# sum 为 100以为的奇数之和
println(sum)

"#
        .to_string();

    chen_lang::run(code).unwrap();
}
