# Use JavaScript method semantics

JS-like Chen will use `obj.method(...)` as the normal method-call syntax and provide the receiver through a JavaScript-like `this` binding. The old colon method-call form is retired because keeping it would make the language continue to read as Lua-like rather than JavaScript-like.
