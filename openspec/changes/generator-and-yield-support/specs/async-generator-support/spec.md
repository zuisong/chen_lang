## ADDED Requirements

### Requirement: async function* syntax
The parser SHALL support `async function*` for defining asynchronous generators.

#### Scenario: Parse async generator
- **WHEN** parsing `async function* gen() { yield await fetchData() }`
- **THEN** it should be recognized as an async generator with `is_async: true` and `is_generator: true`

### Requirement: await inside generators
Asynchronous generators SHALL support `await` expressions.

#### Scenario: Await in generator
- **WHEN** calling `next()` on an async generator
- **THEN** it should return a `Promise` that resolves to `{ value, done }`
