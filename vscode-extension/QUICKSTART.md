# 快速构建和安装指南

## 🚀 一键构建

```bash
cd vscode-extension
chmod +x build.sh
./build.sh
```

## 📦 手动构建

如果自动脚本不工作，手动执行以下步骤：

### 1. 安装依赖

```bash
cd vscode-extension
bun install
```

### 2. 编译

```bash
bun run compile
```

### 3. 打包

```bash
bun run package
```

这会生成 `chen-lang-0.2.0.vsix` 文件。

## 💾 安装扩展

### 方法 1: 命令行

```bash
code --install-extension chen-lang-0.2.0.vsix
```

### 方法 2: VS Code UI

1. 打开 VS Code
2. 按 `Cmd+Shift+P` (Mac) 或 `Ctrl+Shift+P` (Windows/Linux)
3. 输入 "Extensions: Install from VSIX..."
4. 选择 `chen-lang-0.2.0.vsix` 文件

## ✅ 验证安装

1. 重新加载 VS Code 窗口
2. 打开任何 `.ch` 文件
3. 检查：
   - 语法高亮是否工作
   - 左下角是否显示 "Chen Lang"
   - 输入代码时是否有补全提示

## 🔧 故障排除

### Bun 未安装

```bash
# macOS/Linux
curl -fsSL https://bun.sh/install | bash

# 或使用 Homebrew
brew install bun
```

### LSP 服务器未找到

确保 `chen_lang_lsp` 在 PATH 中：

```bash
which chen_lang_lsp
# 应该输出: /Users/chen/.cargo/bin/chen_lang_lsp
```

如果没有，在 VS Code 设置中指定完整路径：

```json
{
    "chenLang.lsp.path": "/Users/chen/.cargo/bin/chen_lang_lsp"
}
```

### 编译错误

清理并重新构建：

```bash
rm -rf node_modules out *.vsix
bun install
bun run compile
bun run package
```

## 📝 构建命令说明

- `bun install` - 安装依赖
- `bun run compile` - 编译 TypeScript 到 JavaScript
- `bun run watch` - 监听模式（开发用）
- `bun run package` - 打包成 VSIX 文件

## 🎯 下一步

安装成功后：

1. 打开 `lsp/test_features.ch` 测试功能
2. 尝试所有 LSP 功能（跳转、引用、重命名等）
3. 查看 [FEATURES.md](../lsp/FEATURES.md) 了解详细功能

## 💡 提示

- VSIX 文件可以分享给其他人使用
- 每次修改代码后需要重新打包
- 开发时使用 `bun run watch` 自动编译
