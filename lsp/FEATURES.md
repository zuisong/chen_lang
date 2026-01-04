# Chen Lang LSP 新功能使用指南

本指南介绍 Chen Lang LSP 最新实现的功能。

## 🎯 新增功能

### 1. 实时语法检查（诊断）

LSP 现在集成了 chen_lang 解析器，可以实时检测语法错误。

**功能说明：**

- 打开文件时自动检查语法
- 编辑时实时更新错误提示
- 错误会在编辑器中用红色波浪线标记

**示例：**

```chen
# 这会产生语法错误
def broken_function( {
    let x =
}
```

编辑器会显示：

- 错误位置：第 1 行
- 错误信息：Unexpected token 或 Parse error

### 2. 跳转到定义 (Go to Definition)

可以快速跳转到函数或变量的定义位置。

**使用方法：**

- **VS Code**: 按住 `Cmd` (Mac) 或 `Ctrl` (Windows/Linux) 并点击符号
- **Neovim**: 使用 `gd` 或 `:lua vim.lsp.buf.definition()`
- **Helix**: 使用 `gd`

**示例：**

```chen
def add(a, b) {
    a + b
}

let result = add(10, 20)  # 点击 'add' 会跳转到函数定义
```

**支持的符号：**

- ✅ 函数定义 (`def`)
- ✅ 变量声明 (`let`)

### 3. 查找引用 (Find References)

查找符号在整个文件中的所有使用位置。

**使用方法：**

- **VS Code**: 右键点击符号 → "查找所有引用" 或按 `Shift+F12`
- **Neovim**: 使用 `:lua vim.lsp.buf.references()`
- **Helix**: 使用 `gr`

**示例：**

```chen
let counter = 0

def increment() {
    counter = counter + 1  # 使用 1
}

def reset() {
    counter = 0  # 使用 2
}

println(counter)  # 使用 3
```

在 `counter` 上查找引用会显示所有 4 个位置（1 个定义 + 3 个使用）。

**特性：**

- 智能词边界检测（不会匹配部分单词）
- 显示所有引用的行号和位置
- 包括定义位置

### 4. 重命名符号 (Rename Symbol)

智能重命名变量或函数，自动更新所有引用。

**使用方法：**

- **VS Code**: 右键点击符号 → "重命名符号" 或按 `F2`
- **Neovim**: 使用 `:lua vim.lsp.buf.rename()`
- **Helix**: 使用 `Space+r`

**示例：**

```chen
let oldName = 10

def useOldName() {
    println(oldName)
}

let result = oldName + 5
```

将 `oldName` 重命名为 `newName` 会自动更新所有 3 个位置。

**特性：**

- 一次操作更新所有引用
- 保持代码一致性
- 支持撤销操作

## 📝 完整功能列表

| 功能       | 状态 | 快捷键 (VS Code)    | 说明         |
| ---------- | ---- | ------------------- | ------------ |
| 语法检查   | ✅   | 自动                | 实时显示错误 |
| 代码补全   | ✅   | `Ctrl+Space`        | 关键字补全   |
| 悬停提示   | ✅   | 鼠标悬停            | 显示文档     |
| 文档符号   | ✅   | `Ctrl+Shift+O`      | 大纲视图     |
| 跳转定义   | ✅   | `F12` 或 `Cmd+点击` | 跳转到定义   |
| 查找引用   | ✅   | `Shift+F12`         | 查找所有使用 |
| 重命名符号 | ✅   | `F2`                | 智能重命名   |
| 代码格式化 | ❌   | -                   | 计划中       |
| 代码折叠   | ❌   | -                   | 计划中       |

## 🔧 配置建议

### VS Code

在 `.vscode/settings.json` 中添加：

```json
{
    "files.associations": {
        "*.ch": "chen"
    },
    "editor.quickSuggestions": {
        "other": true,
        "comments": false,
        "strings": false
    },
    "editor.suggest.showKeywords": true,
    "editor.gotoLocation.multipleDefinitions": "goto",
    "editor.gotoLocation.multipleReferences": "goto"
}
```

### Neovim

在 LSP 配置中添加快捷键：

```lua
local on_attach = function(client, bufnr)
  local opts = { noremap=true, silent=true, buffer=bufnr }
  
  vim.keymap.set('n', 'gd', vim.lsp.buf.definition, opts)
  vim.keymap.set('n', 'gr', vim.lsp.buf.references, opts)
  vim.keymap.set('n', '<leader>rn', vim.lsp.buf.rename, opts)
  vim.keymap.set('n', 'K', vim.lsp.buf.hover, opts)
end

require'lspconfig'.chens.setup{
  cmd = { '/path/to/chen_lang_lsp' },
  filetypes = { 'chen' },
  on_attach = on_attach,
}
```

### Helix

在 `~/.config/helix/languages.toml` 中：

```toml
[language-server.chens]
command = "chen_lang_lsp"

[[language]]
name = "chen"
scope = "source.chen"
file-types = ["ch"]
roots = [".git/"]
language-servers = ["chens"]
```

## 🧪 测试新功能

使用提供的 `test_features.ch` 文件来测试所有功能：

1. 在编辑器中打开 `test_features.ch`
2. 尝试以下操作：
   - 将鼠标悬停在关键字上查看文档
   - 点击变量名跳转到定义
   - 右键点击函数名查找所有引用
   - 重命名一个变量并观察所有引用的更新
   - 取消注释底部的错误代码查看诊断

## 🐛 已知限制

1. **单文件支持**: 当前只支持在单个文件内查找引用和跳转
2. **简单解析**: 使用正则表达式匹配，可能在复杂情况下不准确
3. **无类型信息**: 不区分同名的不同作用域变量

## 🚀 未来计划

- [ ] 跨文件引用和跳转
- [ ] 更精确的语义分析
- [ ] 代码格式化
- [ ] 代码片段 (Snippets)
- [ ] 语义高亮
- [ ] 代码折叠
- [ ] 快速修复建议

## 📚 相关资源

- [LSP 规范](https://microsoft.github.io/language-server-protocol/)
- [Chen Lang 文档](../README.md)
- [开发指南](DEV.md)
