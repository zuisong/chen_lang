## 1. Baseline And Test Harness

- [x] 1.1 Run the current full test suite and capture the failing areas affected by JS-like Chen migration.
- [x] 1.2 Add parser tests for `*.chen.js` source extension handling.
- [x] 1.3 Add negative parser tests for removed syntax: `${...}`, `def`, colon method calls, `catch error`, condition-style `for`, and `import("stdlib/...")`.

## 2. Tokenizer And Parser

- [x] 2.1 Update object literal parsing so `{ key: value }` is accepted in expression position and `{}` remains a block in statement-block position.
- [x] 2.2 Replace `def` parsing with `function` declarations and `function` expressions.
- [x] 2.3 Parse parenthesized `if (...) {}` and `while (...) {}` control flow.
- [x] 2.4 Parse `for (let x of iterable) {}` and reject full `for (init; condition; step)` in the first stage.
- [x] 2.5 Parse JavaScript-style `try {}` / `catch (error) {}` / `finally {}` exception syntax.
- [x] 2.6 Preserve optional semicolon and newline statement separation.
- [x] 2.7 Parse `this` as a dedicated expression or reserved identifier with source locations.
- [x] 2.8 Update tokenizer/parser to accept `//` comments and reject `#` comments in JS-like Chen source.

## 3. Compiler And VM Call Semantics

- [x] 3.1 Lower `obj.method(...)` calls so the callee receives a method-call `this` binding.
- [x] 3.2 Ensure ordinary function calls do not bind `this`.
- [x] 3.3 Raise a runtime error when `this` is accessed without a method-call binding.
- [x] 3.4 Add regression tests for method `this`, nested calls, extracted methods, and unbound `this`.

## 4. Object Model

- [x] 4.1 Implement `Object.create(proto)` for normal prototype inheritance.
- [x] 4.2 Implement prototype lookup for fields missing on the receiver object.
- [x] 4.3 Implement `Object.keys(obj)` and `Object.entries(obj)`.
- [x] 4.4 Keep `Chen.setMeta(obj, meta)` and `Chen.getMeta(obj)` as advanced meta hook APIs.
- [x] 4.5 Ensure missing fields with no prototype or meta hook result return `null`.

## 5. Runtime Globals And Built-In Modules

- [x] 5.1 Create the global `Chen` runtime namespace.
- [x] 5.2 Expose built-in modules under `Chen`: `fs`, `http`, `process`, `timer`, `date`, and `coroutine`.
- [x] 5.3 Implement `Chen.load(path)` for user-defined modules with caching.
- [x] 5.4 Add the global `console` object with `log`, `print`, and `readLine`.
- [x] 5.5 Add the global `JSON` object with `stringify` and `parse`.
- [x] 5.6 Add the global `Object` object with `create`, `keys`, and `entries`.

## 6. JavaScript-Like Runtime APIs

- [x] 6.1 Rename file-system APIs to Deno-like camelCase names: `readTextFile`, `writeTextFile`, `readDir`, `exists`, and `remove`.
- [x] 6.2 Define and implement camelCase API names for `Chen.http`.
- [x] 6.3 Define and implement camelCase API names for `Chen.timer`.
- [x] 6.4 Define and implement camelCase API names for `Chen.date`.
- [x] 6.5 Define and implement camelCase API names for `Chen.process`.
- [x] 6.6 Implement `.length` for arrays and strings.
- [x] 6.7 Implement JavaScript-style array methods `push` and `pop` through dot calls.
- [x] 6.8 Implement JavaScript-style string methods `trim`, `toUpperCase`, and `toLowerCase`.

## 7. Expression Semantics

- [x] 7.1 Keep `null` as the only empty-value concept and avoid adding `undefined`.
- [x] 7.2 Implement Chen-JS truthiness for `false`, `null`, zero, and empty strings.
- [x] 7.3 Make `&&` and `||` return operand values.
- [x] 7.4 Keep `!` returning a boolean.
- [x] 7.5 Preserve `+` string concatenation with mixed string conversion.

## 8. Declarations And Tooling

- [x] 8.1 Add TypeScript declaration file for `Chen`.
- [x] 8.2 Add TypeScript declarations for `Chen.fs`, `Chen.http`, `Chen.process`, `Chen.timer`, `Chen.date`, and `Chen.coroutine`.
- [x] 8.3 Add TypeScript declarations for `console`, `JSON`, and `Object`.
- [x] 8.4 Document that `*.chen.js` is Chen source, not ordinary JavaScript.

## 9. Migration Of Tests And Demos

- [x] 9.1 Rename demo source files from `.ch` to `.chen.js`.
- [x] 9.2 Migrate demo code to JS-like Chen syntax and runtime globals.
- [x] 9.3 Migrate parser/tokenizer unit tests to JS-like Chen syntax.
- [x] 9.4 Migrate language integration tests to JS-like Chen syntax.
- [x] 9.5 Migrate stdlib/runtime integration tests to JS-like Chen runtime globals.
- [x] 9.6 Migrate import tests from `import("...")` to `Chen.load(...)` or direct `Chen.*` runtime access.

## 10. Verification

- [x] 10.1 Run `cargo fmt`.
- [x] 10.2 Run `cargo test`.
- [x] 10.3 Run representative `*.chen.js` demos through the CLI.
- [x] 10.4 Run OpenSpec validation or status checks for `migrate-to-js-like-chen`.
