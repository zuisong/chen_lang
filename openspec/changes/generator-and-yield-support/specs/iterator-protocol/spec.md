## MODIFIED Requirements

### Requirement: Generic iterator interface
The system SHALL support objects as iterators if they provide a `next()` method, or have a property keyed by `Symbol.iterator` (or `Symbol.asyncIterator` for async).

#### Scenario: Object with next method is valid iterator
- **WHEN** using `for (let x of { next: function() { ... } })`
- **THEN** it SHALL be treated as an iterator

#### Scenario: Generator is valid iterator
- **WHEN** using `for (let x of generator_instance)`
- **THEN** it SHALL be treated as an iterator because it implements `Symbol.iterator`
