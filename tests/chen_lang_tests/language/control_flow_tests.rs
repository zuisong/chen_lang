use crate::common::run_chen_lang_code;

#[test]
fn test_simple_for_loop() {
    let code = r#"
let i = 0
for i <= 2 {
    print(i)
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
if a > b {
    print(1)
}
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "1");
}

#[test]
fn test_if_else_example() {
    let code = r#"
let i = 0
for i <= 99 {
    if i%2 == 0 {
        println(i + " 是偶数 ")
    } else {
        println(i + " 是奇数 ")
    }
    i = i + 1
}
"#;

    let output = run_chen_lang_code(code).unwrap();

    // 验证包含偶数和奇数的输出
    assert!(output.contains("0 是偶数"));
    assert!(output.contains("1 是奇数"));
    assert!(output.contains("98 是偶数"));
    assert!(output.contains("99 是奇数"));
}

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
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "3");
}

#[test]
fn test_infinite_for() {
    let code = r#"
let i = 0
for {
    i = i + 1
    if i == 3 {
        break
    }
}
print(i)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "3");
}

#[test]
fn test_for_in_array() {
    let code = r#"
let arr = [10, 20, 30]
for x in arr {
    print(x)
}
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("10"));
    assert!(output.contains("20"));
    assert!(output.contains("30"));
}

#[test]
fn test_for_in_object() {
    let code = r#"
let obj = ${ a: 1, b: 2 }
for v in obj {
    print(v)
}
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("1"));
    assert!(output.contains("2"));
}

#[test]
fn test_for_in_string() {
    let code = r#"
let s = "ABC"
for char in s {
    print(char)
}
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("A"));
    assert!(output.contains("B"));
    assert!(output.contains("C"));
}

#[test]
fn test_for_in_coroutine() {
    let code = r#"
let co = coroutine.create(def() {
    coroutine.yield(100)
    coroutine.yield(200)
})
for x in co {
    print(x)
}
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("100"));
    assert!(output.contains("200"));
}

#[test]
fn test_explicit_iter_call() {
    let code = r#"
let arr = [5, 6]
let it = arr:iter()
print(coroutine.resume(it))
print(coroutine.resume(it))
print(coroutine.resume(it))
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("5"));
    assert!(output.contains("6"));
    assert!(output.contains("null"));
}

#[test]
fn test_for_in_break() {
    let code = r#"
let arr = [1, 2, 3, 4]
for x in arr {
    if x == 3 {
        break
    }
    print(x)
}
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("1"));
    assert!(output.contains("2"));
    assert!(!output.contains("3"));
}

#[test]
fn test_for_in_continue() {
    let code = r#"
let arr = [1, 2, 3, 4]
let sum = 0
for x in arr {
    if x == 2 {
        continue
    }
    sum = sum + x
}
print(sum)
"#;
    // 1 + 3 + 4 = 8
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "8");
}

#[test]
fn test_for_in_array_entries() {
    let code = r#"
let arr = ["A", "B"]
for e in arr:entries() {
    print(e.key + ":" + e.value)
}
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("0:A"));
    assert!(output.contains("1:B"));
}

#[test]
fn test_for_in_object_entries() {
    let code = r#"
let obj = ${ x: 100 }
for e in obj:entries() {
    print(e.key + "=" + e.value)
}
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("x=100"));
}
