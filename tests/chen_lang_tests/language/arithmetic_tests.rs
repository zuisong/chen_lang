use crate::common::run_chen_lang_code;

#[test]
fn test_simple_arithmetic() {
    let code = r#"
let io = import "stdlib/io"
let print = io.print
let i = 1
let j = 2
let k = i + j
print(k)
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "3");
}

#[test]
fn test_modulo_operation() {
    let code = r#"
let io = import "stdlib/io"
let print = io.print
let a = 10
let b = 3
let result = a % b
print(result)
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "1"); // 10 % 3 = 1
}

#[test]
fn test_complex_expression() {
    let code = r#"
let io = import "stdlib/io"
let print = io.print
let a = 2
let b = 3
let c = 4
let result = a + b * c
print(result)
let result2 = (a + b) * c
print(result2)
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("14")); // 2 + 3 * 4 = 14
    assert!(output.contains("20")); // (2 + 3) * 4 = 20
}

#[test]
fn test_metatable_add_operator() {
    let code = r#"
        let io = import "stdlib/io"
        let print = io.print
        let PointMeta = #{
            __add: def(a, b) {
                return #{ x: a.x + b.x, y: a.y + b.y }
            }
        }

        let p1 = #{ x: 10, y: 20 }
        set_meta(p1, PointMeta)

        let p2 = #{ x: 3, y: 5 }
        set_meta(p2, PointMeta)

        let p3 = p1 + p2
        print(p3.x)
        print(p3.y)
    "#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("13")); // 10 + 3 = 13
    assert!(output.contains("25")); // 20 + 5 = 25
}

#[test]
fn test_metatable_add_symmetric_lookup() {
    let code = r#"
        let io = import "stdlib/io"
        let print = io.print
        let VectorMeta = #{
            __add: def(a, b) {
                return #{ x: a.x + b.x, y: a.y + b.y }
            }
        }

        let point = #{ x: 1, y: 2 } # No metatable for point

        let vector = #{ x: 10, y: 20 }
        set_meta(vector, VectorMeta)

        let result = point + vector # point is left_val, vector is right_val. left_val has no metamethod.
        print(result.x)
        print(result.y)
    "#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("11")); // 1 + 10 = 11
    assert!(output.contains("22")); // 2 + 20 = 22
}

#[test]
fn test_metatable_subtract_operator() {
    let code = r#"
        let io = import "stdlib/io"
        let print = io.print
        let PointMeta = #{
            __sub: def(a, b) {
                return #{ x: a.x - b.x, y: a.y - b.y }
            }
        }

        let p1 = #{ x: 30, y: 25 }
        set_meta(p1, PointMeta)

        let p2 = #{ x: 10, y: 5 }
        set_meta(p2, PointMeta)

        let p3 = p1 - p2
        print(p3.x)
        print(p3.y)
    "#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("20")); // 30 - 10 = 20
    assert!(output.contains("20")); // 25 - 5 = 20
}

#[test]
fn test_metatable_multiply_operator() {
    let code = r#"
        let io = import "stdlib/io"
        let print = io.print
        let PointMeta = #{
            __mul: def(a, b) {
                return #{ x: a.x * b.x, y: a.y * b.y }
            }
        }

        let p1 = #{ x: 5, y: 10 }
        set_meta(p1, PointMeta)

        let p2 = #{ x: 2, y: 3 }
        set_meta(p2, PointMeta)

        let p3 = p1 * p2
        print(p3.x)
        print(p3.y)
    "#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("10")); // 5 * 2 = 10
    assert!(output.contains("30")); // 10 * 3 = 30
}

#[test]
fn test_metatable_subtract_symmetric_lookup() {
    let code = r#"
        let io = import "stdlib/io"
        let print = io.print
        let VectorMeta = #{
            __sub: def(a, b) {
                return #{ x: a.x - b.x, y: a.y - b.y }
            }
        }

        let point = #{ x: 10, y: 20 } # No metatable for point

        let vector = #{ x: 100, y: 50 }
        set_meta(vector, VectorMeta)

        let result = vector - point # vector is left_val, point is right_val. point has no metamethod.
        print(result.x)
        print(result.y)
    "#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("90")); // 100 - 10 = 90
    assert!(output.contains("30")); // 50 - 20 = 30
}

#[test]
fn test_metatable_multiply_symmetric_lookup() {
    let code = r#"
        let io = import "stdlib/io"
        let print = io.print
        let VectorMeta = #{
            __mul: def(a, b) {
                return #{ x: a.x * b.x, y: a.y * b.y }
            }
        }

        let point = #{ x: 3, y: 5 } # No metatable for point

        let vector = #{ x: 10, y: 20 }
        set_meta(vector, VectorMeta)

        let result = vector * point # vector is left_val, point is right_val. point has no metamethod.
        print(result.x)
        print(result.y)
    "#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("30")); // 10 * 3 = 30
    assert!(output.contains("100")); // 20 * 5 = 100
}
