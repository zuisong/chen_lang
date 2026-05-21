# Use Chen extension namespace

Chen-specific runtime capabilities will be exposed through a global `Chen` namespace using JavaScript-style camelCase names such as `Chen.setMeta` and `Chen.getMeta`. This keeps non-JavaScript features explicit without scattering special builtins across the global namespace.
