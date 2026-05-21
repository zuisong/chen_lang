# Use JavaScript object literals

JS-like Chen will use `{ key: value }` for object literals and retire the old `${ key: value }` form. This is central to making Chen Lang source look like JavaScript, with parser ambiguity resolved by treating `{}` as an object literal in expression position and as a block in statement-block position.
