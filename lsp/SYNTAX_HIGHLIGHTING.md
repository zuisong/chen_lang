# Chen Lang 语法高亮配置

## VS Code

### 方式 1: 使用基础高亮（推荐开始）

在 `.vscode/settings.json` 中添加：

```json
{
  "files.associations": {
    "*.ch": "python"
  },
  "[python]": {
    "editor.defaultFormatter": null
  }
}
```

这会让 `.ch` 文件使用 Python 的高亮，适合快速开始。

### 方式 2: 自定义 TextMate 语法

创建 `.vscode/syntaxes/chen.tmLanguage.json`：

```json
{
  "scopeName": "source.chen",
  "patterns": [
    {
      "match": "\\b(let|def|if|else|for|return|break|continue|try|catch|finally|throw|import)\\b",
      "name": "keyword.control.chen"
    },
    {
      "match": "\\b(true|false|null)\\b",
      "name": "constant.language.chen"
    },
    {
      "match": "\"[^\"]*\"",
      "name": "string.quoted.double.chen"
    },
    {
      "match": "\\b\\d+\\.?\\d*\\b",
      "name": "constant.numeric.chen"
    },
    {
      "match": "\\b[a-zA-Z_][a-zA-Z0-9_]*\\b",
      "name": "variable.other.chen"
    },
    {
      "match": "#.*$",
      "name": "comment.line.chen"
    }
  ]
}
```

然后在 `.vscode/settings.json` 中添加：

```json
{
  "files.associations": {
    "*.ch": "chen"
  }
}
```

## Vim / Neovim

### 使用 vim-polyglot

```vim
" .vimrc 或 init.vim
autocmd BufRead,BufNewFile *.ch set filetype=python
```

### 自定义语法

创建 `~/.vim/syntax/chen.vim`：

```vim
" Chen Lang 语法高亮
syntax clear

syntax keyword chenKeyword let def if else for return break continue try catch finally throw import
syntax keyword chenConstant true false null
syntax match chenNumber "\d\+\.\?\d*"
syntax region chenString start='"' end='"'
syntax match chenComment "#.*$"
syntax match chenIdentifier "[a-zA-Z_][a-zA-Z0-9_]*"

highlight link chenKeyword Keyword
highlight link chenConstant Constant
highlight link chenNumber Number
highlight link chenString String
highlight link chenComment Comment
highlight link chenIdentifier Identifier

autocmd BufRead,BufNewFile *.ch set filetype=chen
```

## Emacs

创建 `~/.emacs.d/chen-mode.el`：

```elisp
(define-derived-mode chen-mode
  prog-mode "Chen"
  "Major mode for Chen Lang programming.")

(setq chen-keywords
      '("let" "def" "if" "else" "for" "return" "break" "continue"
        "try" "catch" "finally" "throw" "import"))

(setq chen-constants
      '("true" "false" "null"))

(setq chen-keywords-regexp
      (regexp-opt chen-keywords 'words))

(setq chen-constants-regexp
      (regexp-opt chen-constants 'words))

(setq chen-font-lock-keywords
      `(
        (,chen-keywords-regexp . font-lock-keyword-face)
        (,chen-constants-regexp . font-lock-constant-face)
        ("#.*$" . font-lock-comment-face)
        ("\"[^\"]*\"" . font-lock-string-face)
        ("\\b\\d+\\.?\\d*\\b" . font-lock-constant-face)
        ))

(define-key chen-mode-map (kbd "C-c C-c") 'compile)

(provide 'chen-mode)
```

在 `~/.emacs` 或 `~/.emacs.d/init.el` 中添加：

```elisp
(load "~/.emacs.d/chen-mode")
(add-to-list 'auto-mode-alist '("\\.ch\\'" . chen-mode))
```
