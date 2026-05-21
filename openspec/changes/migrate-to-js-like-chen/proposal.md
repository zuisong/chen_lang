## Why

Chen Lang's current surface mixes Chen-specific syntax, Lua-like method calls, and JavaScript-like constructs, which makes the language harder to learn and harder for IDEs to recognize. Migrating to JS-like Chen creates a coherent source shape that remains Chen-specific while improving editor friendliness and user expectations.

## What Changes

- **BREAKING**: Remove old Chen syntax instead of maintaining a dual-syntax compatibility layer.
- **BREAKING**: Replace `${ key: value }` object literals with JavaScript-style `{ key: value }`.
- **BREAKING**: Replace `def` declarations and expressions with JavaScript-style `function`.
- **BREAKING**: Replace colon method calls with `obj.method(...)` and method-call `this` binding.
- **BREAKING**: Replace old condition-style `for` loops with `while (...)` and `for (let x of iterable)`.
- **BREAKING**: Replace `#` comments with JavaScript-style `//` comments.
- **BREAKING**: Replace old stdlib imports with runtime globals: `Chen.*`, `console`, `JSON`, and `Object`.
- Add JS-like runtime namespaces for built-in modules, including `Chen.fs`, `Chen.http`, `Chen.process`, `Chen.timer`, `Chen.date`, and `Chen.coroutine`.
- Add JS-familiar runtime objects: `console`, `JSON`, and `Object`.
- Add `*.chen.js` as the JS-like Chen source file extension.
- Add TypeScript `.d.ts` declarations for IDE support.

## Capabilities

### New Capabilities

- `js-like-syntax`: JS-like source syntax, direct old-syntax replacement, file extension, statement separation, control flow, functions, exceptions, and object literals.
- `js-like-runtime`: Runtime globals and built-in module surface, including `Chen`, `console`, `JSON`, `Object`, module loading, and TypeScript declarations.
- `js-like-object-model`: Prototype-first object behavior, method `this` binding, collection APIs, truthiness, logical operators, and string addition.

### Modified Capabilities

- None.

## Impact

- Parser/tokenizer: source grammar changes for functions, object literals, control flow, method calls, exceptions, semicolons, and `this`.
- AST/compiler/VM: method calls must bind `this`; ordinary functions must reject unbound `this`; prototype lookup and runtime globals must be supported.
- Runtime libraries: stdlib modules move to `Chen.*`; console/JSON/Object objects become runtime globals; APIs move to camelCase JS/Deno-like names.
- Tests and demos: all old syntax and `.ch` examples must migrate to `*.chen.js`.
- Tooling: add `.d.ts` declarations for IDE recognition of runtime globals.
