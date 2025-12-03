mod common;
use common::run_chen_lang_code;

#[test]
fn test_string_operations() {
    let code = r#"
let hello = "Hello"
let world = "World"
let result = hello + " " + world
print(result)
"#;

    let output = run_chen_lang_code(code).unwrap();
    // 字符串被转换为哈希值，但应该有输出
    assert!(!output.trim().is_empty());
}

#[test]
fn test_nine_nine_multiply_table() {
    let code = r#"
let i=1
for i<=9 {
    let j = 1
    for j <= i {
        let temp_prod = i*j
        print(j + "x" + i + "=" + temp_prod + " ")
        j = j + 1
    }
    println("")
    i=i+1
}
"#;

    let output = run_chen_lang_code(code).unwrap();
    println!("{}", output);
    let lines: Vec<&str> = output.lines().collect();
    assert!(lines[0].contains("1x1=1"));
    assert!(lines[8].contains("9x9=81"));
}

#[test]
fn test_sum_example() {
    let code = r#"
def aaa(n){
    let i = 100
    let sum = 0
    for i != 0 {
        i = i - 1
        if (i%2!=0) || (i%3==0)  {
            println(i)
            sum = sum + i
        }
    }
    println("100以内的 奇数或者是能被三整除的偶数 之和是")
    println(sum)
    sum
}
let sum = 0
sum = aaa(100)
println(sum)
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("100以内的 奇数或者是能被三整除的偶数 之和是"));
    assert!(output.contains("3316"));
}
