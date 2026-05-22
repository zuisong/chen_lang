# Map stdlib modules to runtime namespace

JS-like Chen will expose built-in modules as stable runtime namespace properties instead of requiring stdlib imports. The mapping is `stdlib/date` to `Chen.date`, `stdlib/timer` to `Chen.timer`, `stdlib/http` to `Chen.http`, and `stdlib/process` to `Chen.process`; user-defined modules load through `Chen.load(path)`.
