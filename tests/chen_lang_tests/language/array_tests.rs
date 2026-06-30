use crate::common::run_chen_lang_code;

fn run_code_and_check(code: &str, expected_contains: &[&str]) {
    let output = run_chen_lang_code(code).expect("Execution failed");
    for s in expected_contains {
        assert!(output.contains(s), "Output missing: {:?} in {:?}", s, output);
    }
}

#[test]
fn test_array_creation() {
    run_code_and_check(
        r#"
        local arr = [10, 20, 30]
        print(arr[0])
        print(arr[1])
        print(arr[2])
    "#,
        &["10", "20", "30"],
    );
}

#[test]
fn test_array_indexing() {
    run_code_and_check(
        r#"
        local arr = [10, 20]
        print(arr[0])
        arr[1] = 50
        print(arr[1])
    "#,
        &["10", "50"],
    );
}

#[test]
fn test_sparse_array() {
    run_code_and_check(
        r#"
        local arr = [1]
        arr[10] = "sparse"
        print(arr[10]) 
    "#,
        &["sparse"],
    );
}

#[test]
fn test_mixed_array() {
    run_code_and_check(
        r#"
        local arr = [1, "two", true]
        print(arr[0])
        print(arr[1])
        print(arr[2])
    "#,
        &["1", "two", "true"],
    );
}

#[test]
fn test_array_push() {
    run_code_and_check(
        r#"
        local arr = [10, 20]
        local new_len = arr:push(30)
        print(new_len)
        print(arr[2])
    "#,
        &["3", "30"],
    );
}

#[test]
fn test_array_pop() {
    run_code_and_check(
        r#"
        local arr = [10, 20]
        local val = arr:pop()
        print(val)
        local removed = arr[1] 
        if removed == nil then
            print("Removed")
        end
    "#,
        &["20", "Removed"],
    );
}

#[test]
fn test_array_len() {
    run_code_and_check(
        r#"
        local arr = [1, 2, 300]
        print(arr:len())
    "#,
        &["3"],
    );
}

#[test]
fn test_array_type_tag() {
    run_code_and_check(
        r#"
        local arr = []
        print(arr.__type)
    "#,
        &["Array"],
    );
}

#[test]
fn test_array_like_object_creation() {
    run_code_and_check(
        r#"
        local arr = {}
        arr[0] = "first"
        arr[1] = "second"
        arr[2] = "third"
        print(arr[0])
        print(arr[1])
        print(arr[2])
    "#,
        &["first", "second", "third"],
    );
}

#[test]
fn test_array_like_index_access() {
    run_code_and_check(
        r#"
        local arr = {}
        arr[0] = 10
        arr[1] = 20
        arr[2] = 30
        local sum = arr[0] + arr[1] + arr[2]
        print(sum)
    "#,
        &["60"],
    );
}

#[test]
fn test_array_like_index_assignment() {
    run_code_and_check(
        r#"
        local arr = {}
        arr[0] = 1
        arr[1] = 2
        arr[2] = 3
        arr[0] = 100
        arr[1] = 200
        print(arr[0])
        print(arr[1])
        print(arr[2])
    "#,
        &["100", "200", "3"],
    );
}

#[test]
fn test_array_like_dynamic_indexing() {
    run_code_and_check(
        r#"
        local arr = {}
        arr[0] = "a"
        arr[1] = "b"
        arr[2] = "c"
        local i = 0
        while i < 3 do
            print(arr[i])
            i = i + 1
        end
    "#,
        &["a", "b", "c"],
    );
}

#[test]
fn test_array_like_sparse_array() {
    run_code_and_check(
        r#"
        local sparse = {}
        sparse[0] = "start"
        sparse[100] = "end"
        print(sparse[0])
        print(sparse[100])
    "#,
        &["start", "end"],
    );
}

#[test]
fn test_array_like_mixed_keys() {
    run_code_and_check(
        r#"
        local mixed = {}
        mixed[0] = "first element"
        mixed[1] = "second element"
        mixed.name = "my array"
        mixed.length = 2
        print(mixed[0])
        print(mixed[1])
        print(mixed.name)
        print(mixed.length)
    "#,
        &["first element", "second element", "my array", "2"],
    );
}

#[test]
fn test_array_like_nested() {
    run_code_and_check(
        r#"
        local row0 = {}
        row0[0] = 1
        row0[1] = 2
        local row1 = {}
        row1[0] = 3
        row1[1] = 4
        local matrix = {}
        matrix[0] = row0
        matrix[1] = row1
        print(matrix[0][0])
        print(matrix[0][1])
        print(matrix[1][0])
        print(matrix[1][1])
    "#,
        &["1", "2", "3", "4"],
    );
}

#[test]
fn test_array_like_iteration() {
    run_code_and_check(
        r#"
        local arr = {}
        arr[0] = 10
        arr[1] = 20
        arr[2] = 30
        arr[3] = 40
        local sum = 0
        local i = 0
        while i < 4 do
            sum = sum + arr[i]
            i = i + 1
        end
        print(sum)
    "#,
        &["100"],
    );
}

#[test]
fn test_array_like_with_strings() {
    run_code_and_check(
        r#"
        local names = {}
        names[0] = "Alice"
        names[1] = "Bob"
        names[2] = "Charlie"
        local greeting = "Hello, " + names[0] + "!"
        print(greeting)
    "#,
        &["Hello, Alice!"],
    );
}
