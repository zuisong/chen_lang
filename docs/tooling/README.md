# Chen Lang Tooling & IDE Setup

This document describes how to set up your development environment for JS-like Chen Lang.

## Source File Extension: `*.chen.js`

Chen Lang source files use the `.chen.js` extension (e.g., `main.chen.js`, `utils.chen.js`).

> [!IMPORTANT]
> - **Chen Source Files**: `*.chen.js` files are **Chen Lang source files**, not ordinary JavaScript. They are executed by the Chen Lang VM.
> - **Tooling Compatibility**: The extension and JavaScript-like syntax are used **exclusively to leverage existing JavaScript tooling** for syntax highlighting, autocomplete/IntelliSense, bracket matching, and code formatting in editors.
> - **Comment Syntax**: Chen source should use JavaScript-style `//` comments.
> - **No `undefined`**: Chen does **not** have `undefined`. The only empty-value concept in Chen is `null`.
> - **Runtime Globals & Built-ins**: Chen's built-in capabilities come from `Chen` (namespaces like `Chen.fs`, `Chen.http`, `Chen.process`, `Chen.timer`, `Chen.date`, `Chen.io`), `console`, `JSON`, and `Object`. Other standard JS global variables, classes, or Node/browser APIs are not supported at runtime.

## VS Code Setup

To get the best experience in VS Code, including IntelliSense for Chen-specific globals:

1.  **TypeScript Declarations:** 
    Ensure `docs/tooling/chen.d.ts` is in your workspace.
2.  **`jsconfig.json`:**
    Create a `jsconfig.json` file in your project root to tell VS Code to include the Chen declarations:

    ```json
    {
      "compilerOptions": {
        "module": "commonjs",
        "target": "es6",
        "checkJs": true,
        "lib": ["es6"]
      },
      "include": [
        "**/*.chen.js",
        "docs/tooling/chen.d.ts"
      ]
    }
    ```

3.  **File Associations:**
    If VS Code doesn't automatically recognize `.chen.js` as JavaScript, add this to your `settings.json`:

    ```json
    "files.associations": {
      "*.chen.js": "javascript"
    }
    ```

## Other Editors

Most modern editors (Sublime Text, WebStorm, Vim/Emacs with LSP) that support JavaScript will work well with `.chen.js` files. For IntelliSense, ensure the editor is configured to use the `chen.d.ts` declaration file.
