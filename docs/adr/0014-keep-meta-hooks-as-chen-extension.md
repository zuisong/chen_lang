# Keep meta hooks as Chen extension

JS-like Chen will keep metatable-style hooks as advanced Chen-specific APIs exposed through `Chen.setMeta` and `Chen.getMeta`, while normal object inheritance uses `Object.create`. This preserves Chen capabilities such as operator hooks without making metatables the primary object model.
