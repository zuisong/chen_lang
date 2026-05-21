# Expose built-in Chen modules under Chen namespace

JS-like Chen will expose built-in Chen modules directly under the global `Chen` runtime namespace, such as `Chen.fs`, `Chen.http`, and `Chen.process`, while JavaScript-familiar runtime objects keep familiar names such as `console.log` and `JSON.stringify`. User-defined Chen modules are loaded with `Chen.load(path)`, which separates built-in runtime capabilities from module loading.
