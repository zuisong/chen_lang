fn main() {
    simple_logger::init();

    let code = r#"
i = 9
for i {
    j = 10- i
    for j {
        m = 10 - i
        n = 10 - i + j - 1
        print(n)
        print(" x ")
        print(m)
        print(" = ")
        print(m*n)
        print("    ")
        j = j - 1
    }
    println("")
    i = i- 1
}
"#
        .to_string();

    chen_lang::run(code).unwrap();
}