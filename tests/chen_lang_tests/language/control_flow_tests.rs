use crate::common::run_chen_lang_code;

#[test]
fn test_simple_for_loop() {
    let code = r#"
let i = 0
while (i <= 2) {
    console.print(i)
    i = i + 1
}
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("0"));
    assert!(output.contains("1"));
    assert!(output.contains("2"));
}

#[test]
fn test_simple_if_statement() {
    let code = r#"
let a = 5
let b = 3
if (a > b) {
    console.print(1)
}
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "1");
}

#[test]
fn test_if_else_example() {
    let code = r#"
let i = 0
while (i <= 99) {
    if (i%2 == 0) {
        console.log(i + " 是偶数 ")
    } else {
        console.log(i + " 是奇数 ")
    }
    i = i + 1
}
"#;

    let output = run_chen_lang_code(code).unwrap();

    assert!(output.contains("0 是偶数"));
    assert!(output.contains("1 是奇数"));
    assert!(output.contains("98 是偶数"));
    assert!(output.contains("99 是奇数"));
}

#[test]
fn test_break() {
    let code = r#"
let i = 0
while (i < 10) {
    i = i + 1
    if (i == 5) {
        break
    }
}
console.print(i)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "5");
}

#[test]
fn test_continue() {
    let code = r#"
let i = 0
let sum = 0
while (i < 10) {
    i = i + 1
    if (i % 2 == 0) {
        continue
    }
    sum = sum + i
}
console.print(sum)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "25");
}

#[test]
fn test_nested_loops_break() {
    let code = r#"
let i = 0
let j = 0
let sum = 0
while (i < 3) {
    i = i + 1
    j = 0
    while (j < 3) {
        j = j + 1
        if (j == 2) {
            break
        }
        sum = sum + 1
    }
}
console.print(sum)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "3");
}

#[test]
fn test_infinite_for() {
    let code = r#"
let i = 0
while (true) {
    i = i + 1
    if (i == 3) {
        break
    }
}
console.print(i)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "3");
}

#[test]
fn test_for_of_array() {
    let code = r#"
let arr = [10, 20, 30]
for (let x of arr) {
    console.print(x)
}
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("10"));
    assert!(output.contains("20"));
    assert!(output.contains("30"));
}

#[test]
fn test_for_of_object() {
    let code = r#"
let obj = { a: 1, b: 2 }
for (let v of obj) {
    console.print(v)
}
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("1"));
    assert!(output.contains("2"));
}

#[test]
fn test_for_of_string() {
    let code = r#"
let s = "ABC"
for (let char of s) {
    console.print(char)
}
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("A"));
    assert!(output.contains("B"));
    assert!(output.contains("C"));
}

#[test]
fn test_for_of_break() {
    let code = r#"
let arr = [1, 2, 3, 4]
for (let x of arr) {
    if (x == 3) {
        break
    }
    console.print(x)
}
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("1"));
    assert!(output.contains("2"));
    assert!(!output.contains("3"));
}

#[test]
fn test_for_of_continue() {
    let code = r#"
let arr = [1, 2, 3, 4]
let sum = 0
for (let x of arr) {
    if (x == 2) {
        continue
    }
    sum = sum + x
}
console.print(sum)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "8");
}

#[test]
fn test_for_of_array_entries() {
    let code = r#"
let arr = ["A", "B"]
for (let e of arr.entries()) {
    console.print(e.key + ":" + e.value)
}
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("0:A"));
    assert!(output.contains("1:B"));
}

#[test]
fn test_for_of_object_entries() {
    let code = r#"
let obj = { x: 100 }
for (let e of obj.entries()) {
    console.print(e[0] + "=" + e[1])
}
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("x=100"));
}
