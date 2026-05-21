## ADDED Requirements

### Requirement: Chen runtime namespace
The runtime SHALL provide a global `Chen` object for Chen-specific capabilities and built-in modules.

#### Scenario: Access Chen namespace
- **WHEN** source references `Chen`
- **THEN** the runtime resolves it to an object

### Requirement: Built-in modules under Chen
The runtime SHALL expose built-in modules directly under `Chen`.

#### Scenario: Access file-system module
- **WHEN** source references `Chen.fs`
- **THEN** the runtime resolves it to the file-system module object

#### Scenario: Access process module
- **WHEN** source references `Chen.process`
- **THEN** the runtime resolves it to the process module object

### Requirement: User module loading
The runtime SHALL load user-defined modules through `Chen.load(path)`.

#### Scenario: Load user module
- **WHEN** source calls `Chen.load("tests/fixtures/math_utils.chen.js")`
- **THEN** the runtime loads, executes, caches, and returns the module value

### Requirement: Old stdlib import is rejected
The parser or runtime MUST reject old `import("stdlib/...")` module loading.

#### Scenario: Reject stdlib import expression
- **WHEN** source contains `let fs = import("stdlib/fs")`
- **THEN** parsing or execution fails because stdlib imports are no longer supported

### Requirement: Console runtime object
The runtime SHALL provide a global `console` object for console I/O.

#### Scenario: Console log writes a line
- **WHEN** source calls `console.log("hello")`
- **THEN** stdout receives `hello` followed by a newline

#### Scenario: Console print writes without newline
- **WHEN** source calls `console.print("hello")`
- **THEN** stdout receives `hello` without appending a newline

### Requirement: JSON runtime object
The runtime SHALL provide a global `JSON` object with `stringify` and `parse`.

#### Scenario: JSON stringify
- **WHEN** source calls `JSON.stringify({ ok: true })`
- **THEN** the runtime returns a JSON string representing the object

#### Scenario: JSON parse
- **WHEN** source calls `JSON.parse("{\"ok\":true}")`
- **THEN** the runtime returns an object whose `ok` field is true

### Requirement: Object runtime object
The runtime SHALL provide a global `Object` object with prototype and enumeration helpers.

#### Scenario: Object create
- **WHEN** source calls `Object.create(proto)`
- **THEN** the runtime returns a new object whose inherited lookup uses `proto`

#### Scenario: Object keys
- **WHEN** source calls `Object.keys({ a: 1, b: 2 })`
- **THEN** the runtime returns an array containing `a` and `b`

#### Scenario: Object entries
- **WHEN** source calls `Object.entries({ a: 1 })`
- **THEN** the runtime returns an array containing an entry with key `a` and value `1`

### Requirement: Runtime declarations
The project SHALL provide TypeScript declaration files for JS-like Chen runtime globals.

#### Scenario: Declaration file exists
- **WHEN** the project is built or checked
- **THEN** a `.d.ts` declaration file exists for `Chen`, `console`, `JSON`, and `Object`
