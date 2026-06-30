use crate::common::run_chen_lang_code;

#[test]
fn test_simple_for_loop() {
    let code = r#"
local i = 0
while i <= 2 do
    print(i)
    i = i + 1
end
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("0"));
    assert!(output.contains("1"));
    assert!(output.contains("2"));
}

#[test]
fn test_simple_if_statement() {
    let code = r#"
local a = 5
local b = 3
if a > b then
    print(1)
end
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "1");
}

#[test]
fn test_if_else_example() {
    let code = r#"
local i = 0
while i <= 99 do
    if i%2 == 0 then
        print(i + " 是偶数 ")
    else
        print(i + " 是奇数 ")
    end
    i = i + 1
end
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
local i = 0
while i < 10 do
    i = i + 1
    if i == 5 then
        break
    end
end
print(i)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "5");
}

#[test]
fn test_continue() {
    let code = r#"
local i = 0
local sum = 0
while i < 10 do
    i = i + 1
    if i % 2 == 0 then
        continue
    end
    sum = sum + i
end
print(sum)
"#;
    // 1 + 3 + 5 + 7 + 9 = 25
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "25");
}

#[test]
fn test_nested_loops_break() {
    let code = r#"
local i = 0
local j = 0
local sum = 0
while i < 3 do
    i = i + 1
    j = 0
    while j < 3 do
        j = j + 1
        if j == 2 then
            break
        end
        sum = sum + 1
    end
end
print(sum)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "3");
}

#[test]
fn test_infinite_for() {
    let code = r#"
local i = 0
while true do
    i = i + 1
    if i == 3 then
        break
    end
end
print(i)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "3");
}

#[test]
fn test_for_in_array() {
    let code = r#"
local arr = [10, 20, 30]
for x in arr do
    print(x)
end
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("10"));
    assert!(output.contains("20"));
    assert!(output.contains("30"));
}

#[test]
fn test_for_in_object() {
    let code = r#"
local obj = { a = 1, b = 2 }
for v in obj do
    print(v)
end
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("1"));
    assert!(output.contains("2"));
}

#[test]
fn test_for_in_string() {
    let code = r#"
local s = "ABC"
for char in s do
    print(char)
end
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("A"));
    assert!(output.contains("B"));
    assert!(output.contains("C"));
}

#[test]
fn test_for_in_coroutine() {
    let code = r#"
local co = coroutine.create(function()
    coroutine.yield(100)
    coroutine.yield(200)
end)
for x in co do
    print(x)
end
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("100"));
    assert!(output.contains("200"));
}

#[test]
fn test_explicit_iter_call() {
    let code = r#"
local arr = [5, 6]
local it = arr:iter()
print(coroutine.resume(it))
print(coroutine.resume(it))
print(coroutine.resume(it))
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("5"));
    assert!(output.contains("6"));
    assert!(output.contains("nil"));
}

#[test]
fn test_for_in_break() {
    let code = r#"
local arr = [1, 2, 3, 4]
for x in arr do
    if x == 3 then
        break
    end
    print(x)
end
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("1"));
    assert!(output.contains("2"));
    assert!(!output.contains("3"));
}

#[test]
fn test_for_in_continue() {
    let code = r#"
local arr = [1, 2, 3, 4]
local sum = 0
for x in arr do
    if x == 2 then
        continue
    end
    sum = sum + x
end
print(sum)
"#;
    // 1 + 3 + 4 = 8
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "8");
}

#[test]
fn test_for_in_array_entries() {
    let code = r#"
local arr = ["A", "B"]
for e in arr:entries() do
    print(e.key + ":" + e.value)
end
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("0:A"));
    assert!(output.contains("1:B"));
}

#[test]
fn test_for_in_object_entries() {
    let code = r#"
local obj = { x = 100 }
for e in obj:entries() do
    print(e.key + "=" + e.value)
end
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("x=100"));
}
