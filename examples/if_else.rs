fn main() {
    simple_logger::init().unwrap();
    let cpde: String = r#"
let i = 0
for i<=100{
    if i%2 == 0{
        println(i + " 是偶数")
    }else{
        println(i + " 是奇数")
    }
    i = i+1
}
"#
    .to_string();

    chen_lang::run(cpde).unwrap();
}
