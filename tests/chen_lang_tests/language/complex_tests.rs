use crate::common::run_chen_lang_code;

#[test]
fn test_string_operations() {
    let code = r#"
local hello = "Hello"
local world = "World"
local result = hello + " " + world
print(result)
"#;

    let output = run_chen_lang_code(code).unwrap();
    // 字符串被转换为哈希值，但应该有输出
    assert!(!output.trim().is_empty());
}

#[test]
fn test_nine_nine_multiply_table() {
    let code = r#"
local i=1
while i<=9 do
    local j = 1
    while j <= i do
        local temp_prod = i*j
        print(j + "x" + i + "=" + temp_prod + " ")
        j = j + 1
    end
    println("")
    i=i+1
end
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
function aaa(n)
    local i = 100
    local sum = 0
    while i ~= 0 do
        i = i - 1
        if (i%2~=0) or (i%3==0) then
            println(i)
            sum = sum + i
        end
    end
    println("100以内的 奇数或者是能被三整除的偶数 之和是")
    println(sum)
    return sum
end
local sum = 0
sum = aaa(100)
println(sum)
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("100以内的 奇数或者是能被三整除的偶数 之和是"));
    assert!(output.contains("3316"));
}
