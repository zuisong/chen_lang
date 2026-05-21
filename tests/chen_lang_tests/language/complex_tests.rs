use crate::common::run_chen_lang_code;

#[test]
fn test_string_operations() {
    let code = r#"
let hello = "Hello"
let world = "World"
let result = hello + " " + world
console.print(result)
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(!output.trim().is_empty());
}

#[test]
fn test_nine_nine_multiply_table() {
    let code = r#"
let i = 1
while (i <= 9) {
    let j = 1
    while (j <= i) {
        let temp_prod = i * j
        console.print(j + "x" + i + "=" + temp_prod + " ")
        j = j + 1
    }
    console.log("")
    i = i + 1
}
"#;

    let output = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = output.lines().collect();
    assert!(lines[0].contains("1x1=1"));
    assert!(lines[8].contains("9x9=81"));
}

#[test]
fn test_sum_example() {
    let code = r#"
function aaa(n){
    let i = 100
    let sum = 0
    while (i != 0) {
        i = i - 1
        if (i % 2 != 0 || i % 3 == 0) {
            console.log(i)
            sum = sum + i
        }
    }
    console.log("100以内的 奇数或者是能被三整除的偶数 之和是")
    console.log(sum)
    return sum
}
let sum = 0
sum = aaa(100)
console.log(sum)
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("100以内的 奇数或者是能被三整除的偶数 之和是"));
    assert!(output.contains("3316"));
}
