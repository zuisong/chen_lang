mod coroutine_test;
mod object_test;
mod parse_test;
mod parser_comprehensive_test;
mod sync_io_test;
mod token_test;
mod value_tests;

#[cfg(test)]
mod tests_1 {
    use pretty_assertions::assert_eq;

    #[test]
    #[should_panic]
    fn test_diff() {
        #[derive(Debug, PartialEq)]
        struct Foo {
            lorem: &'static str,
            ipsum: u32,
            dolor: Result<String, String>,
        }

        let x = Some(Foo {
            lorem: "Hello World!",
            ipsum: 42,
            dolor: Ok("hey".to_string()),
        });
        let y = Some(Foo {
            lorem: "Hello Wrold!",
            ipsum: 42,
            dolor: Ok("hey ho!".to_string()),
        });

        assert_eq!(x, y);
    }
}
