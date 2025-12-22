use crate::common::run_chen_lang_code;

#[test]
fn test_break() {
    let code = r#"
let i = 0
for i < 10 {
    i = i + 1
    if i == 5 {
        break
    }
}
print(i)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "5");
}

#[test]
fn test_continue() {
    let code = r#"
let i = 0
let sum = 0
for i < 10 {
    i = i + 1
    if i % 2 == 0 {
        continue
    }
    sum = sum + i
}
print(sum)
"#;
    // 1 + 3 + 5 + 7 + 9 = 25
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "25");
}

#[test]
fn test_nested_loops_break() {
    let code = r#"
let i = 0
let j = 0
let sum = 0
for i < 3 {
    i = i + 1
    j = 0
    for j < 3 {
        j = j + 1
        if j == 2 {
            break
        }
        sum = sum + 1
    }
}
print(sum)
"#;
    // Outer loop runs 3 times.
    // Inner loop runs: j=1 (sum++), j=2 (break). So sum increments by 1 each outer iteration.
    // Total sum = 3
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "3");
}
