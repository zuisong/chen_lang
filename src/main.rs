fn main() {
    let code = r#"
sum = 0
i = 998
print(i)
for i {
    sum = sum + i
    i = i + -2
}
print(sum)
"#
        .to_string();

    chen_lang::run(code).unwrap();
}
