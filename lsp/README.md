# Chen Lang LSP

为 Chen Lang 语言提供 Language Server Protocol 支持的开发工具。

## 功能特性

- ✅ **语法检查**: 实时显示词法和语法错误
- ✅ **代码补全**: 关键字自动补全
- ✅ **悬停提示**: 关键字文档提示
- ✅ **文档符号**: 提取函数和变量定义
- ✅ **跳转定义**: 跳转到函数和变量定义处
- ✅ **查找引用**: 查找符号的所有使用位置
- ✅ **重命名符号**: 智能重命名变量和函数
- ✅ **增量同步**: 高效的文档同步

## 安装

```bash
cd lsp
cargo install --path .
```

## 使用

### VS Code

1. 创建 VS Code 扩展配置 `.vscode/extensions.json`:

```json
{
  "recommendations": ["vscodevim.vim"]
}
```

2. 创建 `.vscode/settings.json`:

```json
{
  "languageserver": {
    "chen": {
      "command": "chen_lang_lsp",
      "filetypes": ["chen"],
      "rootPatterns": [".git/"]
    }
  }
}
```

3. 将 `.ch` 文件关联到 chen 语言，创建 `.vscode/settings.json`:

```json
{
  "files.associations": {
    "*.ch": "chen"
  }
}
```

### Neovim (nvim-lspconfig)

```lua
require'lspconfig'.chens.setup{
  cmd = { 'chen_lang_lsp' },
  filetypes = { 'chen' },
  root_dir = require'lspconfig'.util.root_pattern('.git', '.'),
}
```

### Helix

在 `~/.config/helix/languages.toml` 中添加:

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

## 开发

```bash
# 运行 LSP 服务器
cargo run

# 运行测试
cargo test

# 代码格式化
cargo fmt

# 检查代码
cargo clippy
```

## LSP 功能实现状态

- [x] 文档同步 (did_open, did_change, did_close)
- [x] 诊断信息 (发布编译错误和警告)
- [x] 补全 (关键字补全)
- [x] 悬停 (关键字文档)
- [x] 文档符号 (提取函数和变量)
- [x] 跳转定义 (Go to Definition)
- [x] 查找引用 (Find References)
- [x] 重命名符号 (Rename Symbol)
- [ ] 代码格式化 (Formatting)
- [ ] 代码折叠 (Folding Ranges)
- [ ] 语义高亮 (Semantic Highlighting)

## 架构

```
lsp/
├── lib.rs          # 库入口
├── server.rs       # LSP 服务器实现
├── document.rs     # 文档管理
├── bin.rs         # 可执行文件入口
└── Cargo.toml     # 依赖配置
```

## 依赖项

- `tower-lsp`: LSP 协议实现
- `tokio`: 异步运行时
- `ropey`: 高效的文本编辑器数据结构
- `dashmap`: 并发安全的 HashMap
- `chen_lang`: Chen Lang 核心库（解析器和编译器）
