# Chen Lang LSP - 安装和使用指南

## 安装

```bash
cd lsp
cargo install --path .
```

## 使用方法

### 1. VS Code

#### 方式 A: 使用简单配置（快速开始）

在项目根目录创建 `.vscode/settings.json`:

```json
{
  "files.associations": {
    "*.ch": "chen"
  },
  "languageserver": {
    "chen": {
      "command": "chen_lang_lsp",
      "filetypes": ["chen"],
      "rootPatterns": [".git/"],
      "settings": {}
    }
  }
}
```

#### 方式 B: 使用完整扩展

1. 将 `vscode-settings.json` 复制到 `.vscode/settings.json`
2. 安装 VS Code 配置

### 2. Neovim (nvim-lspconfig)

在 `init.lua` 中添加：

```lua
require'lspconfig'.chens.setup{
  cmd = { 'chen_lang_lsp' },
  filetypes = { 'chen' },
  root_dir = function(fname)
      return require'lspconfig'.util.root_pattern('.git')(fname)
         or require'lspconfig'.util.path.dirname(fname)
  end,
  settings = {},
}
```

### 3. Vim (vim-lsp)

在 `.vimrc` 中添加：

```vim
if executable('chen_lang_lsp')
  au User lsp_setup call lsp#register_server({
    \ 'name': 'chen_lang_lsp',
    \ 'cmd': {server_info->['chen_lang_lsp']},
    \ 'whitelist': ['chen'],
    \ })
endif
```

### 4. Helix

1. 复制 `helix-languages.toml` 到 `~/.config/helix/languages.toml`
2. 或者追加到现有配置中

```toml
[language-server.chens]
command = "chen_lang_lsp"

[[language]]
name = "chen"
scope = "source.chen"
file-types = ["ch"]
roots = [".git/"]
language-servers = ["chens"]
indent = { tab-width = 4, unit = "    " }
```

### 5. Emacs (eglot)

在 `.emacs` 或 `init.el` 中添加：

```elisp
(add-to-list 'eglot-server-programs
             '(chen-mode . ("chen_lang_lsp")))

(add-hook 'chen-mode-hook 'eglot-ensure)
```

## 功能特性

### 已实现功能

| 功能 | 描述 |
|------|------|
| 🔍 语法检查 | 实时显示词法和语法错误 |
| 💡 代码补全 | 关键字自动补全 |
| 📖 悬停提示 | 关键字文档提示 |
| 📑 文档符号 | 函数和变量大纲视图 |
| 📝 增量同步 | 高效的文档更新 |

### 使用示例

#### 语法检查

在编辑器中输入错误的代码：

```chen
def incomplete( {
    let x = 
}
```

编辑器会立即显示红色错误提示：
- Line 1: Tokenization error: Expected closing parenthesis
- Line 2: Parse error: Expected expression

#### 代码补全

输入关键字的一部分，然后按 `.` 或 `Ctrl+Space`：

```chen
le    → let
d     → def
im    → import
```

#### 悬停提示

将鼠标悬停在关键字上：

```chen
def add(a, b) {  # 悬停在 "def" 上
    a + b
}
```

显示：
```chen
def name(params) { ... }

Define a function
```

#### 文档符号

打开侧边栏的大纲视图，可以看到：
```
main
  add
  x
  y
  sum
```

## 开发和调试

### 开发模式运行

```bash
cd lsp
RUST_LOG=chen_lang_lsp=debug cargo run
```

### 手动测试 LSP

创建测试文件 `test.ch`:

```chen
let x = 10
let y = 20

def add(a, b) {
    a + b
}

let sum = add(x, y)
println(sum)
```

在支持 LSP 的编辑器中打开此文件，你应该看到：
- 诊断信息：无错误（代码正确）
- 补全：输入 `l` 时补全 `let`
- 悬停：悬停在 `def` 上看到文档
- 符号：大纲视图中显示 `add` 和变量

### 故障排除

#### LSP 无法启动

检查是否正确安装：
```bash
which chen_lang_lsp
```

如果未找到，运行：
```bash
cargo install --path .
```

#### 没有语法高亮

参考 `SYNTAX_HIGHLIGHTING.md` 配置编辑器的语法高亮。

#### 诊断信息不显示

1. 检查日志：`RUST_LOG=chen_lang_lsp=debug cargo run`
2. 确认编辑器连接到 LSP
3. 尝试重新打开文件

## 下一步

1. ✅ 实现跳转定义
2. ✅ 查找引用
3. ✅ 代码格式化
4. ✅ 代码折叠
5. ✅ 语义高亮

欢迎贡献！查看 `DEV.md` 了解如何开发。
