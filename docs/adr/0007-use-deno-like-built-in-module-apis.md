# Use Deno-like built-in module APIs

Built-in Chen modules under the `Chen` runtime namespace will use JavaScript-style camelCase APIs, with file-system operations named in a Deno-like style such as `Chen.fs.readTextFile`, `Chen.fs.writeTextFile`, and `Chen.fs.readDir`. This keeps the runtime namespace useful for IDEs and familiar to JavaScript users instead of preserving the old snake_case stdlib API shape.
