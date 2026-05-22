## ADDED Requirements

### Requirement: Generator execution
Calling a generator function SHALL NOT execute the body immediately, but instead return a Generator Object.

#### Scenario: Call generator function
- **WHEN** executing `let g = gen()` where `gen` is a `function*`
- **THEN** `g` should be an object with a `next` method

### Requirement: yield suspension
Executing a `yield` instruction SHALL suspend the generator's Fiber and return the yielded value to the caller.

#### Scenario: Yield value
- **WHEN** calling `g.next()`
- **THEN** it returns `{ value: 1, done: false }` if `yield 1` is executed

### Requirement: Iterator protocol integration
Generator Objects SHALL have `[Symbol.iterator]` and `[Symbol.asyncIterator]` methods that return the generator object itself.

#### Scenario: Iterating over generator
- **WHEN** using `for (let x of gen())`
- **THEN** it should successfully iterate over all yielded values
