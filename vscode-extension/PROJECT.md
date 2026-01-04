# Chen Lang VS Code 扩展项目

## 📁 项目结构

```
vscode-extension/
├── src/
│   └── extension.ts              # 扩展入口点（TypeScript）
├── syntaxes/
│   └── chen.tmLanguage.json      # 语法高亮规则
├── out/                          # 编译输出（自动生成）
│   └── extension.js              # 编译后的 JS
├── package.json                  # 扩展配置和依赖
├── tsconfig.json                 # TypeScript 配置
├── language-configuration.json   # 语言配置（括号、注释等）
├── build.sh                      # 自动构建脚本
├── README.md                     # 项目说明
├── QUICKSTART.md                 # 快速开始指南
├── .gitignore                    # Git 忽略文件
└── .vscodeignore                 # VSIX 打包忽略文件
```

## 🛠️ 技术栈

- **运行时**: Bun (快速的 JavaScript 运行时)
- **语言**: TypeScript
- **构建工具**: Bun build (内置)
- **打包工具**: @vscode/vsce
- **LSP 客户端**: vscode-languageclient v9.0.1

## ✨ 特性

### 已实现功能

- ✅ 语法高亮（关键字、字符串、数字、注释）
- ✅ LSP 集成（完整的语言服务器支持）
- ✅ 实时语法检查
- ✅ 代码补全
- ✅ 悬停提示
- ✅ 跳转到定义
- ✅ 查找引用
- ✅ 重命名符号
- ✅ 文档符号（大纲视图）
- ✅ 自动括号匹配
- ✅ 注释支持

### 配置选项

- `chenLang.lsp.path`: LSP 服务器路径（默认: `chen_lang_lsp`）
- `chenLang.trace.server`: 服务器日志级别（`off`/`messages`/`verbose`）

## 🚀 使用方法

### 构建 VSIX

```bash
cd vscode-extension

# 方法 1: 使用构建脚本
chmod +x build.sh
./build.sh

# 方法 2: 手动构建
bun install
bun run compile
bun run package
```

### 安装扩展

```bash
# 命令行安装
code --install-extension chen-lang-0.2.0.vsix

# 或在 VS Code 中
# Cmd+Shift+P > Extensions: Install from VSIX...
```

### 开发模式

```bash
# 监听模式（自动编译）
bun run watch

# 在 VS Code 中按 F5 启动调试
```

## 📋 构建命令

| 命令              | 说明            |
| ----------------- | --------------- |
| `bun install`     | 安装依赖        |
| `bun run compile` | 编译 TypeScript |
| `bun run watch`   | 监听模式        |
| `bun run package` | 打包 VSIX       |
| `./build.sh`      | 一键构建        |

## 🔧 依赖说明

### 运行时依赖

- `vscode-languageclient`: LSP 客户端库

### 开发依赖

- `@types/node`: Node.js 类型定义
- `@types/vscode`: VS Code API 类型定义
- `@vscode/vsce`: VS Code 扩展打包工具
- `typescript`: TypeScript 编译器

## 📝 配置文件说明

### package.json

扩展的核心配置文件，包含：

- 扩展元数据（名称、版本、描述）
- 语言定义（文件扩展名、语法文件）
- 配置选项
- 构建脚本
- 依赖列表

### tsconfig.json

TypeScript 编译配置：

- 目标: ES2020
- 模块: ESNext
- 模块解析: bundler（Bun 优化）
- 严格模式: 启用

### language-configuration.json

语言行为配置：

- 注释符号: `#`
- 括号对: `{}`, `[]`, `()`
- 自动闭合
- 代码折叠

### syntaxes/chen.tmLanguage.json

TextMate 语法定义：

- 关键字高亮
- 字符串、数字识别
- 函数定义识别
- 运算符高亮

## 🎯 工作流程

1. **编写代码**: 修改 `src/extension.ts`
2. **编译**: `bun run compile` 或 `bun run watch`
3. **测试**: 按 F5 在扩展开发主机中测试
4. **打包**: `bun run package` 生成 VSIX
5. **安装**: 安装 VSIX 到 VS Code
6. **使用**: 打开 `.ch` 文件享受 IDE 支持

## 🐛 常见问题

### Q: Bun 是什么？

A: Bun 是一个快速的 JavaScript 运行时，类似 Node.js
但更快。它内置了包管理器和构建工具。

### Q: 为什么使用 Bun 而不是 npm？

A: Bun 更快，安装依赖和构建都比 npm 快很多。而且内置了 TypeScript 支持。

### Q: 可以用 npm 代替 Bun 吗？

A: 可以，但需要修改 `package.json` 中的脚本，将 `bun` 替换为 `npm`，并使用 `tsc`
编译。

### Q: VSIX 文件是什么？

A: VSIX 是 VS Code 扩展的打包格式，可以直接安装到 VS Code 中。

### Q: 如何更新扩展？

A: 修改代码后重新构建 VSIX，然后重新安装即可。

## 📚 相关文档

- [VS Code 扩展 API](https://code.visualstudio.com/api)
- [LSP 规范](https://microsoft.github.io/language-server-protocol/)
- [Bun 文档](https://bun.sh/docs)
- [Chen Lang LSP 功能](../lsp/FEATURES.md)

## 🎉 总结

这个扩展项目使用现代工具链（Bun + TypeScript）构建，可以快速编译和打包。生成的
VSIX 文件可以直接安装到 VS Code，提供完整的 Chen Lang 开发支持。

**下一步**: 运行 `./build.sh` 开始构建！
