# Chen Lang

Chen Lang is a small dynamic language moving toward JavaScript-compatible surface syntax while keeping Chen-specific runtime capabilities explicit.

## Language

**Chen Lang**:
The programming language defined by this repository.
_Avoid_: JavaScript runtime, Lua clone

**JavaScript-compatible surface syntax**:
The syntax direction where supported Chen Lang constructs should look like JavaScript source code.
_Avoid_: full ECMAScript compatibility, JavaScript engine compatibility

**JS-like Chen**:
The migration target where Chen Lang keeps its own language identity while making supported syntax look like JavaScript.
_Avoid_: ECMAScript subset, JavaScript engine

**JavaScript method semantics**:
The method-call direction where `obj.method(...)` is the normal method syntax and the receiver is available through a JavaScript-like receiver binding.
_Avoid_: colon method call, implicit self call

**this binding**:
The receiver binding available inside a method called with **JavaScript method semantics**.
_Avoid_: self parameter

**Chen extension namespace**:
The global `Chen` namespace for Chen-specific capabilities that are not familiar JavaScript runtime objects.
_Avoid_: scattered builtins, magic globals

**Chen-specific capability**:
A Chen capability, such as metatable access, module loading, or operator hooks, exposed through the **Chen extension namespace**.
_Avoid_: JavaScript object feature

**Chen meta hook**:
A Chen-specific object hook used for advanced behavior such as operator hooks or property interception.
_Avoid_: normal prototype inheritance

**Chen runtime namespace**:
The `Chen` runtime surface that contains built-in Chen modules and Chen-specific capabilities.
_Avoid_: stdlib import namespace

**Built-in Chen module**:
A built-in runtime module exposed directly under the **Chen runtime namespace**, such as `Chen.fs`, `Chen.http`, or `Chen.process`.
_Avoid_: imported stdlib module

**Deno-like Chen module API**:
The JavaScript-style API naming convention for built-in Chen modules, using camelCase names familiar from Deno or browser JavaScript.
_Avoid_: snake_case stdlib API

**Chen module loading**:
The explicit loading of user-defined Chen modules through `Chen.load(path)`.
_Avoid_: JavaScript import, stdlib import

**JavaScript-familiar runtime object**:
A runtime object whose name and shape are familiar to JavaScript users, such as `console` or `JSON`.
_Avoid_: Chen-only global

**console runtime object**:
The JavaScript-familiar runtime object for console input and output in **JS-like Chen**.
_Avoid_: stdlib/io

**JSON runtime object**:
The JavaScript-familiar runtime object for JSON serialization and parsing in **JS-like Chen**.
_Avoid_: stdlib/json, Chen.json

**null-only empty value**:
The empty-value model where Chen Lang uses `null` and does not introduce JavaScript `undefined`.
_Avoid_: undefined

**Chen-JS truthiness**:
The truth-value rule where `false`, `null`, zero, and empty strings are falsey, while other values are truthy.
_Avoid_: Luau truthiness

**JavaScript-style logical operators**:
The logical operator behavior where `&&` and `||` return one operand using **Chen-JS truthiness**, while `!` returns a bool.
_Avoid_: boolean-only logic

**JavaScript-style string addition**:
The string concatenation behavior where `+` joins strings and string-plus-other values by converting the other value to a string.
_Avoid_: Luau concatenation

**JavaScript function syntax**:
The function syntax using `function` declarations and `function` expressions in **JS-like Chen**.
_Avoid_: def syntax, arrow function

**First-stage JavaScript control flow**:
The initial control-flow syntax scope for **JS-like Chen**, covering parenthesized `if`, `while`, and `for...of`.
_Avoid_: condition-style for, full JavaScript for

**JavaScript-style exception syntax**:
The exception syntax using `try`, `catch (error)`, `finally`, and `throw`.
_Avoid_: catch-without-parentheses

**Optional semicolon statements**:
The statement separation style where semicolons are optional and newlines may separate statements.
_Avoid_: mandatory semicolons

**Direct syntax replacement**:
The migration strategy where old Chen syntax is removed instead of supported alongside JS-like Chen syntax.
_Avoid_: dual syntax compatibility

**Runtime declaration file**:
A TypeScript `.d.ts` file that describes JS-like Chen runtime globals for IDE support.
_Avoid_: informal runtime docs

**Chen JavaScript source file**:
A JS-like Chen source file using the `*.chen.js` extension.
_Avoid_: .cjs, plain .js

**JavaScript-style object model**:
The object model direction where JS-like Chen should make object behavior feel like JavaScript rather than Lua-style metatables.
_Avoid_: Lua-style object model

**Prototype-first object model**:
The first-stage JavaScript-style object model focused on prototype lookup and `this`, without class syntax.
_Avoid_: class-first object model

**Object runtime object**:
The JavaScript-familiar runtime object that provides object-model operations such as `Object.create`.
_Avoid_: Chen.create

**JavaScript collection API**:
The JavaScript-familiar API shape for arrays, strings, and object enumeration.
_Avoid_: colon collection methods

**JavaScript object literal**:
The object literal syntax `{ key: value }` used by **JS-like Chen**.
_Avoid_: dollar object literal

**JavaScript-style comment**:
The line comment syntax `// ...` used by **JS-like Chen**.
_Avoid_: hash comment

## Relationships

- **JS-like Chen** is the migration target for **Chen Lang**.
- **JavaScript-compatible surface syntax** defines how **JS-like Chen** code should look for supported constructs.
- **JavaScript method semantics** defines the preferred method-call model for **Chen Lang**.
- **this binding** is provided by **JavaScript method semantics**.
- **Chen extension namespace** keeps **Chen-specific capability** features visible and explicit.
- **Chen-specific capability** features may preserve Chen Lang behavior without pretending to be JavaScript behavior.
- **Chen meta hook** is exposed through **Chen extension namespace** APIs such as `Chen.setMeta` and `Chen.getMeta`.
- **Chen runtime namespace** contains **Built-in Chen module** objects and **Chen-specific capability** functions.
- **Chen module loading** is for user-defined modules, not built-in Chen modules.
- **Deno-like Chen module API** governs built-in module method names under the **Chen runtime namespace**.
- **JavaScript-familiar runtime object** names should be used when they make IDE support and user expectations better.
- **console runtime object** replaces the old `stdlib/io` module for console input and output.
- **JSON runtime object** replaces the old `stdlib/json` module for JSON serialization and parsing.
- **null-only empty value** defines missing or empty values in **JS-like Chen**.
- **Chen-JS truthiness** governs conditions and logical operators in **JS-like Chen**.
- **JavaScript-style logical operators** use **Chen-JS truthiness**.
- **JavaScript-style string addition** is the normal string concatenation behavior in **JS-like Chen**.
- **JavaScript function syntax** replaces old `def` function syntax.
- **First-stage JavaScript control flow** replaces old condition-style `for` loops.
- **JavaScript-style exception syntax** replaces old catch binding syntax.
- **Optional semicolon statements** define statement separation in **JS-like Chen**.
- **Direct syntax replacement** governs migration compatibility.
- **Runtime declaration file** supports IDE understanding of JS-like Chen runtime globals.
- **Chen JavaScript source file** is the source file format for **JS-like Chen**.
- **JavaScript-style object model** is the preferred object-model direction for **JS-like Chen**.
- **Prototype-first object model** is the initial scope of **JavaScript-style object model**.
- **Object runtime object** exposes JavaScript-familiar object operations for **Prototype-first object model**.
- **JavaScript collection API** replaces colon-style collection methods for common array, string, and object operations.
- **JavaScript object literal** replaces the old `${ key: value }` object syntax.
- **JavaScript-style comment** replaces the old `#` comment syntax.

## Example dialogue

> **Dev:** "If metatable operations are not JavaScript, where should they live?"
> **Domain expert:** "Put them under the **Chen extension namespace**, for example `Chen.setMeta(...)`, so JavaScript-like syntax stays clean."

> **Dev:** "Should printing be `Chen.println`?"
> **Domain expert:** "Prefer a **JavaScript-familiar runtime object** such as `console.log` when that matches JavaScript user expectations."

> **Dev:** "Should file-system APIs be imported from `stdlib/fs`?"
> **Domain expert:** "No. A **Built-in Chen module** such as `Chen.fs` should be available directly; use **Chen module loading** for user modules."

> **Dev:** "Does **JavaScript-compatible surface syntax** mean Chen Lang must run arbitrary npm-style JavaScript?"
> **Domain expert:** "No. It means supported Chen Lang constructs should look like JavaScript, not that Chen Lang is a JavaScript engine."

## Flagged ambiguities

- "Compatible with JavaScript" can mean visual syntax compatibility, ECMAScript language compatibility, or runtime ecosystem compatibility; resolved for now: this project means **JavaScript-compatible surface syntax**.
- **JS-like Chen** does not promise that arbitrary JavaScript code can run unchanged.
- **Chen-specific capability** features should be exposed through the **Chen extension namespace** unless they are promoted into the core JavaScript-like language.
- Familiar JavaScript runtime surfaces such as `console.log` and `JSON.stringify` should not be forced under `Chen` just for namespace uniformity.
- Built-in modules should be exposed as direct properties of the **Chen runtime namespace**, not loaded with `Chen.import("stdlib/...")`.
- Console input and output should use the **console runtime object**, not `stdlib/io`.
- JSON serialization and parsing should use the **JSON runtime object**, not `stdlib/json` or `Chen.json`.
- Built-in Chen modules should use **Deno-like Chen module API** naming, such as `Chen.fs.readTextFile` instead of `fs.read_file`.
- Built-in modules map to **Chen runtime namespace** properties: `stdlib/date` to `Chen.date`, `stdlib/timer` to `Chen.timer`, `stdlib/http` to `Chen.http`, `stdlib/process` to `Chen.process`, and `coroutine` to `Chen.coroutine`.
- User-defined modules load through `Chen.load(path)`.
- Chen Lang should not introduce JavaScript `undefined`; use **null-only empty value** instead.
- **Chen-JS truthiness** does not include JavaScript `undefined` or `NaN`.
- `&&` and `||` return operand values; `!` returns a bool.
- String concatenation uses `+`, not `..`, and keeps Chen's convenient string conversion for mixed string additions.
- First-stage function syntax supports `function` declarations and expressions, not arrow functions.
- First-stage control flow supports `if (...)`, `while (...)`, and `for (let x of iterable)`.
- First-stage control flow does not include full JavaScript `for (init; condition; step)`.
- Exception handling uses `catch (error)`, not `catch error`.
- Semicolons are optional; newlines may separate statements.
- Old Chen syntax should be removed rather than kept as a compatibility layer.
- JS-like Chen should provide TypeScript `.d.ts` declarations for runtime globals.
- JS-like Chen source files use `*.chen.js`, not `.cjs` or plain `.js`.
- Chen Lang should move object behavior toward **JavaScript-style object model**.
- Class syntax is out of scope for the first stage of **Prototype-first object model**.
- Prototype-based object creation uses `Object.create(proto)`, not `Chen.create(proto)`.
- `Chen.setMeta` and `Chen.getMeta` are advanced **Chen meta hook** APIs, not the normal object inheritance path.
- Ordinary inherited property lookup uses **Prototype-first object model** behavior; `__index` is retained only as an advanced **Chen meta hook**.
- Arrays and strings expose `.length` instead of `:len()`.
- Object enumeration uses `Object.keys(obj)` and `Object.entries(obj)` instead of `obj:keys()` and `obj:entries()`.
- Array methods use JavaScript names such as `push` and `pop`.
- String methods use JavaScript names such as `trim`, `toUpperCase`, and `toLowerCase`.
- `{}` is an object literal in expression position and a block in statement-block position.
- Colon method calls are retired in favor of **JavaScript method semantics**.
- **this binding** exists only for method calls; ordinary function calls do not receive a default receiver.
- Accessing `this` without a **this binding** is an error.
- **Chen extension namespace** uses JavaScript-style camelCase names such as `Chen.setMeta`, `Chen.getMeta`, and `Chen.load`.
