## Why

Chen Lang users need a way to create custom iterators more elegantly and efficiently. Standard JavaScript uses Generator functions (`function*` and `yield`) to manage stateful iteration without manual state management in closures. Adding this syntax will further align Chen Lang with JavaScript surface syntax and provide a powerful tool for asynchronous sequences.

## What Changes

- **Tokenizer Update**: Add support for the `*` suffix in `function*`.
- **Parser Update**: Support `function*` and `async function*` declarations and expressions. Add the `yield` and `yield*` keyword support.
- **AST Update**: Add `Yield` expression variant. Add `is_generator` flag to `FunctionDeclaration`.
- **Compiler/VM Update**: Implement suspension and resumption of function execution. This involves transforming generator functions into a state-machine or utilizing existing Fiber capabilities to yield control back to the iterator caller.
- **Standard Library Update**: Ensure `Symbol.iterator` and `Symbol.asyncIterator` can be used with generator functions.

## Capabilities

### New Capabilities
- `generator-syntax`: Parsing and AST support for `function*` and `yield`.
- `generator-runtime`: VM support for suspending and resuming execution in generator functions, returning an iterator object.
- `async-generator-support`: Support for `async function*` and `await` inside generators.

### Modified Capabilities
- `iterator-protocol`: Update the protocol to allow generator objects as valid iterators.

## Impact

- `src/tokenizer.rs`: New token rules for `yield` and potentially `function*`.
- `src/parser/handwritten.rs`: Support for generator syntax.
- `src/compiler.rs`: Emitting bytecode for `yield`.
- `src/vm/interpreter.rs`: Handling generator suspension/resumption.
- `src/expression.rs`: AST changes.
