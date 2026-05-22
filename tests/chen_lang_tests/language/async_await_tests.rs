use crate::common::run_chen_lang_code;

#[test]
fn test_print_debug() {
    let code = r#"
        console.print("Hello Debug")
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output, "Hello Debug");
}

#[test]
fn test_async_await_basic() {
    let code = r#"
        async function sayHello() {
            return "Hello"
        }
        
        async function main() {
            let msg = await sayHello()
            console.print(msg)
        }
        
        main()
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output, "Hello");
}

#[test]
fn test_async_await_sleep() {
    let code = r#"
        async function main() {
            console.print("Start")
            await Chen.timer.sleep(10)
            console.print("End")
        }
        
        main()
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output, "StartEnd");
}

#[test]
fn test_promise_resolve_static() {
    let code = r#"
        async function main() {
            let p = Promise.resolve("Resolved")
            let val = await p
            console.print(val)
        }
        main()
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output, "Resolved");
}

#[test]
fn test_promise_then_catch_finally() {
    let code = r#"
        async function main() {
            let p1 = Promise.resolve(42)
            p1.then(def(val) {
                console.print("then:" + val)
            })

            let p2 = Promise.reject("err")
            p2.catch(def(reason) {
                console.print("catch:" + reason)
            })

            let p3 = Promise.resolve("ok")
            p3.finally(def() {
                console.print("finally")
            })
        }
        main()
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output, "then:42catch:errfinally");
}

#[test]
fn test_promise_chaining() {
    let code = r#"
        async function main() {
            let p = Promise.resolve(10)
            p.then(def(v) {
                return v + 5
            }).then(def(v) {
                console.print(v)
            })
        }
        main()
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output, "15");
}

#[test]
fn test_promise_new_resolve() {
    let code = r#"
        async function main() {
            let p = Promise.new(def(resolve, reject) {
                resolve("resolved_value")
            })
            let val = await p
            console.print(val)
        }
        main()
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output, "resolved_value");
}

#[test]
fn test_promise_new_reject() {
    let code = r#"
        async function main() {
            let p = Promise.new(def(resolve, reject) {
                reject("rejected_reason")
            })
            try {
                await p
            } catch(e) {
                console.print(e)
            }
        }
        main()
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output, "rejected_reason");
}

#[test]
fn test_promise_all_success() {
    let code = r#"
        async function main() {
            let p1 = Promise.resolve(1)
            let p2 = Promise.resolve(2)
            let p3 = 3
            let all = await Promise.all([p1, p2, p3])
            console.print(all[0] + all[1] + all[2])
        }
        main()
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output, "6");
}

#[test]
fn test_promise_all_fail() {
    let code = r#"
        async function main() {
            let p1 = Promise.resolve(1)
            let p2 = Promise.reject("oops")
            try {
                await Promise.all([p1, p2])
            } catch(e) {
                console.print(e)
            }
        }
        main()
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output, "oops");
}

#[test]
fn test_promise_race() {
    let code = r#"
        async function main() {
            let p1 = Promise.new(def(resolve, reject) {
                async function f() {
                    await Chen.timer.sleep(50)
                    resolve("slow")
                }
                f()
            })
            let p2 = Promise.new(def(resolve, reject) {
                async function f() {
                    await Chen.timer.sleep(5)
                    resolve("fast")
                }
                f()
            })
            let winner = await Promise.race([p1, p2])
            console.print(winner)
        }
        main()
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output, "fast");
}

#[test]
fn test_promise_all_settled() {
    let code = r#"
        async function main() {
            let p1 = Promise.resolve("win")
            let p2 = Promise.reject("lose")
            let results = await Promise.allSettled([p1, p2])
            
            console.print(results[0].status + ":" + results[0].value + ",")
            console.print(results[1].status + ":" + results[1].reason)
        }
        main()
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output, "fulfilled:win,rejected:lose");
}
