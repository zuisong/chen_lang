# Remove old syntax without compatibility layer

JS-like Chen will remove old Chen syntax instead of supporting both old and new forms during migration. This keeps the parser and tests focused on the new language surface and avoids long-term ambiguity around constructs such as `${...}`, `def`, colon method calls, and old condition-style `for` loops.
