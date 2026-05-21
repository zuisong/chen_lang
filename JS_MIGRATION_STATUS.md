# JS-like Chen Migration Status

This document tracks the planned migration from the current Chen syntax/runtime surface to JS-like Chen.

## 1. Migration Target

Chen Lang is moving toward **JS-like Chen**:

- Supported Chen syntax should look like JavaScript.
- Chen is not an ECMAScript engine and does not promise arbitrary JavaScript compatibility.
- JavaScript-familiar runtime objects should keep familiar names.
- Chen-specific runtime capabilities should be explicit under the `Chen` namespace.

## 2. Confirmed Syntax Direction

| Current Chen | JS-like Chen |
| :--- | :--- |
| `${ name: "Alice" }` | `{ name: "Alice" }` |
| `def add(a, b) { ... }` | `function add(a, b) { ... }` |
| `let add = def(a, b) { ... }` | `let add = function(a, b) { ... }` |
| `obj:method(arg)` | `obj.method(arg)` |
| `self` parameter convention | `this` binding for method calls |
| `catch error { ... }` | `catch (error) { ... }` |
| `for i < n { ... }` | `while (i < n) { ... }` |
| `for x in iterable { ... }` | `for (let x of iterable) { ... }` |
| `obj:keys()` | `Object.keys(obj)` |
| `obj:entries()` | `Object.entries(obj)` |
| `arr:len()` / `str:len()` | `arr.length` / `str.length` |
| `str:upper()` / `str:lower()` | `str.toUpperCase()` / `str.toLowerCase()` |
| `# comment` | `// comment` |

## 3. Confirmed Runtime Surface

### JavaScript-Familiar Globals

- `console.log(...)` for line-based console output.
- `console.print(...)` for non-line-ending console output.
- `console.readLine()` for console input.
- `JSON.stringify(value)` and `JSON.parse(text)`.
- `Object.create(proto)`, `Object.keys(obj)`, and `Object.entries(obj)`.

### Chen Runtime Namespace

Built-in Chen modules and Chen-specific capabilities live under `Chen`:

- `Chen.fs`
- `Chen.http`
- `Chen.process`
- `Chen.timer`
- `Chen.date`
- `Chen.coroutine`
- `Chen.setMeta(obj, meta)`
- `Chen.getMeta(obj)`
- `Chen.load(path)` for user-defined modules

Built-in modules are available directly under `Chen`; they are not loaded through `Chen.import("stdlib/...")`.

## 4. Confirmed Semantics

- Chen uses `null` only; it does not introduce JavaScript `undefined`.
- Missing or empty values use `null`.
- Accessing `this` without a method-call binding is an error.
- Method calls bind `this` only when called as `obj.method(...)`.
- Truthiness:
  - falsey: `false`, `null`, zero, empty string.
  - truthy: other values.
- `&&` and `||` return operand values.
- `!` returns a boolean.
- String concatenation uses `+` and keeps Chen's convenient string conversion for mixed string additions.
- Semicolons are optional; newlines may separate statements.
- Comments use JavaScript-style `//` line comments.

## 5. Object Model Direction

The first stage uses a prototype-first JavaScript-style object model:

- Normal prototype inheritance uses `Object.create(proto)`.
- `class` syntax is out of scope for the first stage.
- `Chen.setMeta` and `Chen.getMeta` remain advanced Chen meta hook APIs.
- `__index` is retained only as an advanced Chen meta hook, not the recommended normal inheritance path.

## 6. First-Stage Control Flow Scope

Supported in the first stage:

```js
if (cond) {
}

while (cond) {
}

for (let x of iterable) {
}
```

Deferred:

```js
for (let i = 0; i < n; i = i + 1) {
}
```

## 7. Implementation Task Outline

1. Update tokenizer/parser for JS-like syntax:
   - `{ key: value }` object literals.
   - `function` declarations and expressions.
   - parenthesized `if`, `while`, and `catch`.
   - `for (let x of iterable)`.
   - optional semicolon/newline statement separators.
2. Update AST/compiler/VM call semantics:
   - `obj.method(...)` binds `this`.
   - ordinary function calls do not bind `this`.
   - unbound `this` is an error.
3. Update runtime globals:
   - add `Chen` namespace.
   - add `console`, `JSON`, and `Object` runtime objects.
   - expose built-in modules under `Chen`.
4. Update built-in APIs to JS/Deno-like names:
   - `Chen.fs.readTextFile`, `Chen.fs.writeTextFile`, `Chen.fs.readDir`.
   - collection APIs: `.length`, `push`, `pop`, `toUpperCase`, `toLowerCase`, `trim`.
5. Migrate tests and demo code to JS-like Chen.
6. Add IDE-facing TypeScript declarations for the runtime surface.

## 8. Open Questions

- Exact API names for `Chen.http`, `Chen.timer`, `Chen.date`, and `Chen.process`.

## 9. Compatibility Strategy

Old syntax is removed rather than kept as a migration compatibility layer. Tests and demo code should migrate to JS-like Chen syntax directly.

Examples of removed syntax:

- `${ key: value }`
- `def name(...) { ... }`
- `obj:method(...)`
- `catch error { ... }`
- `for i < n { ... }`
- `import("stdlib/...")`

## 10. IDE Declarations

JS-like Chen should provide TypeScript declaration files for the runtime surface, including:

- `Chen`
- `Chen.fs`
- `Chen.http`
- `Chen.process`
- `Chen.timer`
- `Chen.date`
- `Chen.coroutine`
- `console`
- `JSON`
- `Object`

## 11. Source File Extension

JS-like Chen source files use the `*.chen.js` extension.

Examples:

- `main.chen.js`
- `math_utils.chen.js`
- `demo.chen.js`

The project should avoid `.cjs` because it already means CommonJS JavaScript in the JavaScript ecosystem.
