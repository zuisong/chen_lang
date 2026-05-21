## Context

Chen Lang currently has a mixed source and runtime surface: JavaScript-like `let`, old Chen `${...}` object literals, `def` functions, Lua-like colon method calls, explicit stdlib imports, and metatable-oriented object behavior. The migration target is JS-like Chen: supported Chen constructs look like JavaScript, but Chen remains its own language rather than an ECMAScript engine.

Existing project notes in `CONTEXT.md` and `JS_MIGRATION_STATUS.md` define the target language direction. This design implements that direction as a breaking migration with no old-syntax compatibility layer.

## Goals / Non-Goals

**Goals:**

- Replace old syntax with JS-like Chen syntax.
- Add runtime globals that make Chen code IDE-friendly: `Chen`, `console`, `JSON`, and `Object`.
- Move built-in modules from `import("stdlib/...")` to stable runtime namespace properties.
- Implement method-call `this` binding and prototype-first object behavior.
- Provide TypeScript declarations for runtime globals.
- Migrate tests and demo files to `*.chen.js`.

**Non-Goals:**

- Full ECMAScript compatibility.
- Running arbitrary JavaScript packages or npm code.
- Arrow functions, class syntax, destructuring, hoisting, or full JavaScript `for (init; condition; step)`.
- A dual parser accepting both old Chen syntax and JS-like Chen syntax.
- Introducing JavaScript `undefined`.

## Decisions

1. Use direct old-syntax removal.
   - Decision: remove old constructs such as `${...}`, `def`, `obj:method(...)`, `catch error`, condition-style `for`, `#` comments, and `import("stdlib/...")`.
   - Rationale: dual syntax would increase parser ambiguity and test burden.
   - Alternative considered: keep old syntax during migration. Rejected because the project goal is a clean language surface.

2. Parse JS-like constructs as Chen constructs, not ECMAScript.
   - Decision: implement only the agreed first-stage syntax: object literals, `function`, parenthesized `if`/`while`, `for (let x of iterable)`, JS-style `catch`, and optional semicolons.
   - Rationale: this delivers the JS-like surface without committing to a JS engine.
   - Alternative considered: implement an ECMAScript subset. Rejected because it would drag in hoisting, `undefined`, lexical `this`, destructuring, classes, and more grammar complexity.

3. Treat `obj.method(...)` as method call only when the callee is a field access.
   - Decision: compiler lowers method calls so the receiver is available as `this` inside the callee.
   - Rationale: this preserves JS method-call intuition while keeping ordinary function calls receiver-free.
   - Alternative considered: keep hidden self arguments. Rejected because it would leak Lua/Chen method semantics into a JS-looking surface.

4. Keep `null` as the only empty value.
   - Decision: missing values and explicit empty values use `null`; `undefined` is not introduced.
   - Rationale: this avoids JS's two-empty-values complexity while preserving JS-like syntax.

5. Use runtime globals for IDE-friendly built-ins.
   - Decision: add `Chen`, `console`, `JSON`, and `Object` globals, with built-in modules directly under `Chen`.
   - Rationale: `console`, `JSON`, and `Object` are familiar to JS tools and users; Chen-specific capabilities remain explicit under `Chen`.
   - Alternative considered: put everything under `Chen`. Rejected because it reduces IDE familiarity for `console` and `JSON`.

6. Keep meta hooks as advanced Chen extensions.
   - Decision: normal inheritance uses `Object.create(proto)`, while `Chen.setMeta` and `Chen.getMeta` remain advanced APIs for Chen meta hooks.
   - Rationale: this preserves Chen's advanced behavior without making metatables the primary object model.

7. Provide `.d.ts` declarations and use `*.chen.js`.
   - Decision: JS-like Chen source files use `*.chen.js`; runtime globals are documented through TypeScript declarations.
   - Rationale: the extension signals Chen source while still giving IDEs a JavaScript-like hook; declarations make runtime globals discoverable.

## Risks / Trade-offs

- Parser ambiguity around `{}` → Resolve by using expression/block context: expression position is object literal, statement-block position is block.
- Method-call `this` changes call-frame behavior → Implement and test separately from parser migration.
- Removing old syntax breaks all existing examples and tests → Migrate tests and demo files in planned batches.
- Comment syntax also migrates to JavaScript-style `//` line comments so syntax highlighting and docs match the JS-like surface.
- `*.chen.js` may still be mistaken for JavaScript by tools → Provide declarations and documentation that Chen is JS-like, not ECMAScript.
- Built-in API renames are broad → Keep a mapping table in migration docs and test each runtime object.
- Prototype lookup and Chen meta hooks can overlap → Define `Object.create` as normal inheritance and `Chen.setMeta` as advanced extension only.
