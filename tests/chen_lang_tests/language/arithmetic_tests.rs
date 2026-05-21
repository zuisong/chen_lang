use crate::common::run_chen_lang_code;

#[test]
fn test_simple_arithmetic() {
    let code = r#"
let i = 1
let j = 2
let k = i + j
console.print(k)
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "3");
}

#[test]
fn test_modulo_operation() {
    let code = r#"
let a = 10
let b = 3
let result = a % b
console.print(result)
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "1");
}

#[test]
fn test_complex_expression() {
    let code = r#"
let a = 2
let b = 3
let c = 4
let result = a + b * c
console.print(result)
let result2 = (a + b) * c
console.print(result2)
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("14"));
    assert!(output.contains("20"));
}

#[test]
fn test_metatable_add_operator() {
    let code = r#"
        let PointMeta = {
            __add: function(a, b) {
                return { x: a.x + b.x, y: a.y + b.y }
            }
        }

        let p1 = { x: 10, y: 20 }
        Chen.setMeta(p1, PointMeta)

        let p2 = { x: 3, y: 5 }
        Chen.setMeta(p2, PointMeta)

        let p3 = p1 + p2
        console.print(p3.x)
        console.print(p3.y)
    "#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("13"));
    assert!(output.contains("25"));
}

#[test]
fn test_metatable_add_symmetric_lookup() {
    let code = r#"
        let VectorMeta = {
            __add: function(a, b) {
                return { x: a.x + b.x, y: a.y + b.y }
            }
        }

        let point = { x: 1, y: 2 }

        let vector = { x: 10, y: 20 }
        Chen.setMeta(vector, VectorMeta)

        let result = point + vector
        console.print(result.x)
        console.print(result.y)
    "#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("11"));
    assert!(output.contains("22"));
}

#[test]
fn test_metatable_subtract_operator() {
    let code = r#"
        let PointMeta = {
            __sub: function(a, b) {
                return { x: a.x - b.x, y: a.y - b.y }
            }
        }

        let p1 = { x: 30, y: 25 }
        Chen.setMeta(p1, PointMeta)

        let p2 = { x: 10, y: 5 }
        Chen.setMeta(p2, PointMeta)

        let p3 = p1 - p2
        console.print(p3.x)
        console.print(p3.y)
    "#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("20"));
    assert!(output.contains("20"));
}

#[test]
fn test_metatable_multiply_operator() {
    let code = r#"
        let PointMeta = {
            __mul: function(a, b) {
                return { x: a.x * b.x, y: a.y * b.y }
            }
        }

        let p1 = { x: 5, y: 10 }
        Chen.setMeta(p1, PointMeta)

        let p2 = { x: 2, y: 3 }
        Chen.setMeta(p2, PointMeta)

        let p3 = p1 * p2
        console.print(p3.x)
        console.print(p3.y)
    "#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("10"));
    assert!(output.contains("30"));
}

#[test]
fn test_metatable_subtract_symmetric_lookup() {
    let code = r#"
        let VectorMeta = {
            __sub: function(a, b) {
                return { x: a.x - b.x, y: a.y - b.y }
            }
        }

        let point = { x: 10, y: 20 }

        let vector = { x: 100, y: 50 }
        Chen.setMeta(vector, VectorMeta)

        let result = vector - point
        console.print(result.x)
        console.print(result.y)
    "#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("90"));
    assert!(output.contains("30"));
}

#[test]
fn test_metatable_multiply_symmetric_lookup() {
    let code = r#"
        let VectorMeta = {
            __mul: function(a, b) {
                return { x: a.x * b.x, y: a.y * b.y }
            }
        }

        let point = { x: 3, y: 5 }

        let vector = { x: 10, y: 20 }
        Chen.setMeta(vector, VectorMeta)

        let result = vector * point
        console.print(result.x)
        console.print(result.y)
    "#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("30"));
    assert!(output.contains("100"));
}
