# Use Chen-JS truthiness

JS-like Chen will use a JavaScript-like truthiness rule without introducing JavaScript's `undefined` or `NaN`: `false`, `null`, zero, and empty strings are falsey, while other values are truthy. This matches JavaScript user expectations for common values while keeping Chen's empty-value model simpler.
