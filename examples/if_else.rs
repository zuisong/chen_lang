fn main() {
    env_logger::init();
    let code: String = r#"
let i = 0
for i<100{

    if i % (4 - 1 * 2) == 0{
        println(i + " 是偶数")
    }else{
        println(i + " 是奇数")
    }
    i = i+1
}
"#
    .to_string();

    chen_lang::run(code).unwrap();
}
