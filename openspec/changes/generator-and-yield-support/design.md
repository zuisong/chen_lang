## Context

Chen Lang has transitioned to a Promise-based asynchronous model using Fibers for execution. Currently, custom iterators require manual state management within closures. This design outlines how to implement `function*` and `yield` to allow for elegant iterator creation, leveraging the existing Fiber system or a state-machine transformation.

## Goals / Non-Goals

**Goals:**
- Support `function*` syntax for defining generator functions.
- Support `yield` and `yield*` expressions inside generator functions.
- Support `async function*` for asynchronous generators.
- Integration with `for...of` and `for await...of` loops via the `Symbol.iterator` / `Symbol.asyncIterator` protocols.
- Generator objects must have a `next()` method returning `{ value, done }`.

**Non-Goals:**
- Full ES6 generator compatibility (e.g., `return()` and `throw()` methods on generator objects are secondary goals).
- Performance optimizations beyond basic functionality.

## Decisions

1. **Fiber-based Implementation**: Instead of a complex state-machine rewrite in the compiler (CPS transformation), we will use the existing Fiber system. A generator function call will create a new Fiber. `yield` will suspend the Fiber and return control to the caller with the yielded value. `next()` will resume the Fiber.
   - *Rationale*: Reuses existing robust infrastructure for suspension and resumption. Much simpler to implement than a full compiler transformation.
2. **Generator Object Representation**: A generator function call returns a special "Generator Object". This object wraps the Fiber and provides a `next()` method.
   - *Rationale*: Aligns with standard JS behavior where calling a generator function doesn't execute the body immediately but returns an iterator.
3. **Symbol Integration**: Generator objects will automatically have `[Symbol.iterator]` and `[Symbol.asyncIterator]` (returning themselves) to work with `for...of`.
4. **Bytecode Addition**: A new `Yield` instruction will be added to the VM.

## Risks / Trade-offs

- [Memory overhead] → Each active generator will have its own Fiber (stack and state). If thousands of generators are active, memory usage might increase. *Mitigation*: Ensure Fibers are lightweight and properly garbage-collected when the generator is done.
- [Recursion with yield*] → `yield*` needs careful handling to delegate to another iterator. *Mitigation*: Implement `yield*` by looping over the delegate's iterator and yielding each value.

## Open Questions
- Should `yield` be allowed in top-level code? (Probably not, following JS standards).
- How to handle `next(value)` (passing a value back into the generator)? (Will implement in a later stage if needed).
