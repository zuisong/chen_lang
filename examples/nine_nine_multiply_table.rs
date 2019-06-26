fn main() {
    simple_logger::init().unwrap();

    let code = r#"
i=1
for i<=9 {
    j = 1
    for j<=i {
        print(j + "x" + i + "=" + i*j + " ")
        j = j + 1
    }
    println("")
    i=i+1
}

"#
    .to_string();

    chen_lang::run(code).unwrap();
}
