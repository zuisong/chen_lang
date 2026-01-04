# Chen Lang VS Code Extension

Chen Lang 的 VS Code 扩展，提供完整的 LSP 支持。

## 功能

- ✅ 语法高亮
- ✅ 实时语法检查
- ✅ 代码补全
- ✅ 悬停提示
- ✅ 跳转到定义
- ✅ 查找引用
- ✅ 重命名符号
- ✅ 文档符号（大纲视图）

## 构建和安装

### 前置要求

- [Bun](https://bun.sh/) - JavaScript 运行时和包管理器
- `chen_lang_lsp` - LSP 服务器（需要在 PATH 中）

### 构建步骤

1. 安装依赖：
   ```bash
   bun install
   ```

2. 编译扩展：
   ```bash
   bun run compile
   ```

3. 打包为 VSIX：
   ```bash
   bun run package
   ```

   这会生成 `chen-lang-0.2.0.vsix` 文件。

### 安装扩展

在 VS Code 中：

1. 打开命令面板（`Cmd+Shift+P` 或 `Ctrl+Shift+P`）
2. 输入 "Extensions: Install from VSIX..."
3. 选择生成的 `.vsix` 文件

或者使用命令行：

```bash
code --install-extension chen-lang-0.2.0.vsix
```

## 配置

扩展默认会在 PATH 中查找 `chen_lang_lsp`。

如果需要指定路径，在 VS Code 设置中修改：

```json
{
    "chenLang.lsp.path": "/path/to/chen_lang_lsp"
}
```

## 开发

### 监听模式

```bash
bun run watch
```

### 调试

1. 在 VS Code 中打开此目录
2. 按 `F5` 启动扩展开发主机
3. 在新窗口中测试扩展

## 项目结构

```
vscode-extension/
├── src/
│   └── extension.ts       # 扩展入口点
├── syntaxes/
│   └── chen.tmLanguage.json  # 语法高亮规则
├── out/                   # 编译输出（自动生成）
├── package.json           # 扩展配置
├── tsconfig.json          # TypeScript 配置
└── language-configuration.json  # 语言配置
```

## 许可证

MIT License
