#[cfg(test)]
mod tests {
    use crate::expression::{Expression, Statement, TypeAnnotation};
    use crate::parser::handwritten::parse;
    use crate::tokenizer::tokenizer;

    #[test]
    fn test_parse_async_await() {
        let code = r#"
            async function fetchData(url: string) -> Promise<string> {
                let response = await http.get(url)
                return response.data
            }

            let main = async function() {
                let data = await fetchData("https://example.com")
                let p = Promise.new(function(resolve) {
                    resolve(data)
                })
            }
        "#;

        let tokens = tokenizer(code.to_string()).unwrap();
        let ast = parse(tokens).unwrap();

        assert_eq!(ast.len(), 2);

        // Check async function declaration
        if let Statement::AsyncFunctionDeclaration(decl) = &ast[0] {
            assert_eq!(decl.name.as_deref(), Some("fetchData"));
            assert_eq!(decl.parameters.len(), 1);
            assert_eq!(decl.parameters[0].name, "url");

            if let Some(TypeAnnotation::Promise(inner)) = &decl.return_type {
                if let TypeAnnotation::String = **inner {
                    // OK
                } else {
                    panic!("Expected Promise<string>");
                }
            } else {
                panic!("Expected Promise return type, got {:?}", decl.return_type);
            }

            // Check await expression in body
            if let Statement::Local(local) = &decl.body[0] {
                assert_eq!(local.name, "response");
                if let Expression::Await { expression, .. } = &local.expression {
                    if let Expression::FunctionCall(_call) = &**expression {
                        // OK
                    } else {
                        panic!("Expected function call inside await");
                    }
                } else {
                    panic!("Expected await expression");
                }
            }
        } else {
            panic!("Expected AsyncFunctionDeclaration");
        }

        // Check async function expression
        if let Statement::Local(local) = &ast[1] {
            assert_eq!(local.name, "main");
            if let Expression::AsyncFunction(decl) = &local.expression {
                assert_eq!(decl.name, None);

                // Check Promise.new static constructor call
                if let Statement::Local(p_local) = &decl.body[1] {
                    assert_eq!(p_local.name, "p");
                    if let Expression::FunctionCall(call) = &p_local.expression {
                        if let Expression::GetField { object, field, .. } = &*call.callee {
                            assert_eq!(field, "new");
                            if let Expression::Identifier(name, _) = &**object {
                                assert_eq!(name, "Promise");
                            } else {
                                panic!("Expected Promise identifier");
                            }
                        } else {
                            panic!("Expected Promise.new callee");
                        }
                        assert_eq!(call.arguments.len(), 1);
                    } else {
                        panic!("Expected FunctionCall expression");
                    }
                }
            } else {
                panic!("Expected AsyncFunction expression");
            }
        }
    }
}
