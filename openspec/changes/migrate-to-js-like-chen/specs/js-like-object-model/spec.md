## ADDED Requirements

### Requirement: Method calls bind this
The runtime SHALL bind `this` to the receiver when a function is called as `obj.method(...)`.

#### Scenario: Method reads receiver field
- **WHEN** an object method returns `this.name` and is called as `user.greet()`
- **THEN** `this` resolves to `user` during that call

### Requirement: Ordinary function calls do not bind this
The runtime MUST NOT bind a default receiver for ordinary function calls.

#### Scenario: Unbound this errors
- **WHEN** a function called as `fn()` reads `this`
- **THEN** execution fails with an unbound `this` error

### Requirement: Prototype-first inheritance
The runtime SHALL use `Object.create(proto)` as the normal object inheritance path.

#### Scenario: Inherited field lookup
- **WHEN** an object created with `Object.create(proto)` reads a missing field present on `proto`
- **THEN** the inherited field value is returned

### Requirement: Chen meta hooks remain advanced extensions
The runtime SHALL expose `Chen.setMeta` and `Chen.getMeta` for advanced Chen meta hooks.

#### Scenario: Set and get meta hook object
- **WHEN** source calls `Chen.setMeta(obj, meta)` and then `Chen.getMeta(obj)`
- **THEN** the returned meta value is `meta`

### Requirement: Null-only empty value
The runtime SHALL use `null` as the only empty-value concept and MUST NOT introduce `undefined`.

#### Scenario: Missing field returns null
- **WHEN** source reads a missing field with no prototype or meta hook result
- **THEN** the runtime returns `null`

### Requirement: Chen-JS truthiness
The runtime SHALL treat `false`, `null`, zero, and empty strings as falsey and other values as truthy.

#### Scenario: Empty string condition
- **WHEN** source evaluates `if ("") { console.log("yes") } else { console.log("no") }`
- **THEN** the else branch executes

#### Scenario: Non-empty object condition
- **WHEN** source evaluates `if ({}) { console.log("yes") }`
- **THEN** the then branch executes

### Requirement: JavaScript-style logical operators
The runtime SHALL make `&&` and `||` return operand values and `!` return a boolean.

#### Scenario: Or returns fallback operand
- **WHEN** source evaluates `null || "fallback"`
- **THEN** the result is `"fallback"`

#### Scenario: And returns first falsey operand
- **WHEN** source evaluates `0 && "x"`
- **THEN** the result is `0`

#### Scenario: Not returns boolean
- **WHEN** source evaluates `!""`
- **THEN** the result is `true`

### Requirement: JavaScript-style string addition
The runtime SHALL use `+` for string concatenation and convert mixed string additions to strings.

#### Scenario: String plus number
- **WHEN** source evaluates `"count: " + 3`
- **THEN** the result is `"count: 3"`

### Requirement: JavaScript collection API
The runtime SHALL expose JavaScript-style collection APIs for arrays, strings, and object enumeration.

#### Scenario: Array length
- **WHEN** source evaluates `[1, 2, 3].length`
- **THEN** the result is `3`

#### Scenario: String upper case
- **WHEN** source evaluates `"abc".toUpperCase()`
- **THEN** the result is `"ABC"`

#### Scenario: Array push
- **WHEN** source creates `let arr = []` and calls `arr.push(1)`
- **THEN** `arr.length` is `1`
