## ADDED Requirements

### Requirement: function* syntax
The parser SHALL support `function*` as a valid function definition prefix for both declarations and expressions.

#### Scenario: Parse generator declaration
- **WHEN** parsing `function* gen() { yield 1 }`
- **THEN** it should be recognized as a generator function with `is_generator: true` in AST

### Requirement: yield keyword
The parser SHALL support `yield` expressions inside generator functions.

#### Scenario: Parse yield expression
- **WHEN** parsing `yield 10` inside a generator
- **THEN** it should create a `Yield` expression node in the AST

### Requirement: yield* delegation
The parser SHALL support `yield*` to delegate to another iterator.

#### Scenario: Parse yield* expression
- **WHEN** parsing `yield* other_iterable`
- **THEN** it should create a `Yield` node with `is_delegate: true` in the AST
