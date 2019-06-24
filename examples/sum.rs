fn main() {
    let code = r#"
sum = 0
i = 100
print(i)
for i {
    sum = sum + i
    i = i + -1
}
print(sum)
"#
        .to_string();

    chen_lang::run(code).unwrap();
}