## 1. Tokenizer and Parser Support

- [ ] 1.1 Add `yield` and `yield*` to tokenizer keywords in `src/tokenizer.rs`.
- [ ] 1.2 Support `*` suffix in `function*` in `src/chen.pest` and `src/tokenizer.rs`.
- [ ] 1.3 Add `Yield` and `YieldDelegate` variants to `Expression` enum in `src/expression.rs`.
- [ ] 1.4 Add `is_generator` flag to `FunctionDeclaration` in `src/expression.rs`.
- [ ] 1.5 Update `src/parser/handwritten.rs` to parse generator functions and yield expressions.
- [ ] 1.6 Update `src/parser/pest_impl.rs` (if applicable) to match generator syntax.

## 2. Compiler and VM Infrastructure

- [ ] 2.1 Add `Yield` instruction to `Instruction` enum in `src/vm/program.rs`.
- [ ] 2.2 Implement compiler logic for `Expression::Yield` in `src/compiler.rs`.
- [ ] 2.3 Implement compiler logic to handle `is_generator` functions (return Generator Object).
- [ ] 2.4 Implement `Instruction::Yield` in `src/vm/interpreter.rs` to suspend current Fiber.
- [ ] 2.5 Implement Generator Object logic in `src/vm/interpreter.rs` (wrapping Fiber).

## 3. Integration and Standard Library

- [ ] 3.1 Implement `[Symbol.iterator]` and `[Symbol.asyncIterator]` for Generator Objects.
- [ ] 3.2 Update `for...of` and `for await...of` loops to work with Generator Objects.
- [ ] 3.3 Implement `yield*` delegation logic in the compiler/VM.

## 4. Testing and Documentation

- [ ] 4.1 Write comprehensive unit tests for generator syntax and runtime behavior.
- [ ] 4.2 Add async generator tests.
- [ ] 4.3 Update `LANGUAGE_REFERENCE.md` and `docs/index.js` with generator examples.
- [ ] 4.4 Remove temporary debug artifacts (`tests/parser_debug.rs`, `test_parse.chen.js`).
