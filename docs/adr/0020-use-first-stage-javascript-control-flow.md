# Use first-stage JavaScript control flow

JS-like Chen will start with JavaScript-looking `if (...)`, `while (...)`, and `for (let x of iterable)` control flow, while retiring the old condition-style `for` loops. Full JavaScript `for (init; condition; step)` is deferred because it is parser-heavy and not required to settle the main syntax direction.
