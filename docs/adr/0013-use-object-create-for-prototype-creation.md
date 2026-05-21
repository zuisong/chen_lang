# Use Object.create for prototype creation

JS-like Chen will use the JavaScript-familiar `Object.create(proto)` API for creating objects with a prototype. This keeps prototype-based object creation in the familiar JavaScript object surface instead of presenting it as a Chen-specific `Chen.create` extension.
