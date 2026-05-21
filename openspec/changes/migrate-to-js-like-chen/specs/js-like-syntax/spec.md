## ADDED Requirements

### Requirement: JS-like source files
The system SHALL treat `*.chen.js` files as Chen source files.

#### Scenario: Run chen.js source file
- **WHEN** the CLI runs a file named `main.chen.js`
- **THEN** the file is parsed and executed as Chen source

### Requirement: JavaScript-style comments
The tokenizer SHALL support `//` line comments in Chen source files.

#### Scenario: Parse line comment
- **WHEN** source contains `let x = 1 // note`
- **THEN** the comment is ignored and the statement parses successfully

### Requirement: Old hash comments are rejected
The tokenizer MUST reject `#` comments in JS-like Chen source.

#### Scenario: Reject hash comment
- **WHEN** source contains `let x = 1 # note`
- **THEN** parsing fails with a syntax error

### Requirement: JavaScript object literals
The parser SHALL support JavaScript-style object literals in expression position using `{ key: value }`.

#### Scenario: Parse object literal in declaration
- **WHEN** source contains `let user = { name: "Alice", age: 30 }`
- **THEN** the parser produces an object literal with `name` and `age` fields

### Requirement: Old dollar object syntax is rejected
The parser MUST reject old `${ key: value }` object syntax.

#### Scenario: Reject dollar object literal
- **WHEN** source contains `let user = ${ name: "Alice" }`
- **THEN** parsing fails with a syntax error

### Requirement: JavaScript function syntax
The parser SHALL support `function` declarations and `function` expressions.

#### Scenario: Parse function declaration
- **WHEN** source contains `function add(a, b) { return a + b }`
- **THEN** the parser produces a named function declaration

#### Scenario: Parse function expression
- **WHEN** source contains `let add = function(a, b) { return a + b }`
- **THEN** the parser produces a function expression assigned to `add`

### Requirement: Old def syntax is rejected
The parser MUST reject old `def` function syntax.

#### Scenario: Reject def declaration
- **WHEN** source contains `def add(a, b) { return a + b }`
- **THEN** parsing fails with a syntax error

### Requirement: First-stage JavaScript control flow
The parser SHALL support parenthesized `if`, `while`, and `for (let x of iterable)` control flow.

#### Scenario: Parse if statement
- **WHEN** source contains `if (ok) { console.log("yes") } else { console.log("no") }`
- **THEN** the parser produces an if statement with then and else bodies

#### Scenario: Parse while statement
- **WHEN** source contains `while (i < 3) { i = i + 1 }`
- **THEN** the parser produces a while loop

#### Scenario: Parse for-of statement
- **WHEN** source contains `for (let item of items) { console.log(item) }`
- **THEN** the parser produces an iterator loop binding `item`

### Requirement: Deferred JavaScript for syntax is rejected
The parser MUST reject full JavaScript `for (init; condition; step)` syntax in the first stage.

#### Scenario: Reject three-part for
- **WHEN** source contains `for (let i = 0; i < 3; i = i + 1) { console.log(i) }`
- **THEN** parsing fails with a syntax error

### Requirement: JavaScript-style exception syntax
The parser SHALL support `try`, `catch (error)`, `finally`, and `throw`.

#### Scenario: Parse catch binding with parentheses
- **WHEN** source contains `try { throw "x" } catch (error) { console.log(error) } finally { console.log("done") }`
- **THEN** the parser produces a try-catch-finally statement with `error` as the catch binding

### Requirement: Old catch binding syntax is rejected
The parser MUST reject old `catch error` binding syntax.

#### Scenario: Reject catch without parentheses
- **WHEN** source contains `try { throw "x" } catch error { console.log(error) }`
- **THEN** parsing fails with a syntax error

### Requirement: Optional semicolons
The parser SHALL allow statements to be separated by semicolons or newlines.

#### Scenario: Parse semicolon-separated statements
- **WHEN** source contains `let x = 1; let y = 2; console.log(x + y)`
- **THEN** parsing succeeds with three statements

#### Scenario: Parse newline-separated statements
- **WHEN** source contains three equivalent statements separated by newlines
- **THEN** parsing succeeds with three statements
