#![feature(box_syntax, box_patterns)]

fn main() {
    simple_logger::init().unwrap();

    let code = r#"
# 这里是注释,
# 注释以# 开始, 直到行末
i = 0
sum = 0
for !(i >=1000){
    if (i%2!=0) && (i%3==0){
       sum = sum + i
    }
    i = i+1
}
print(sum)

"#
    .to_string();

    chen_lang::run(code).unwrap();
}
