# ✅ VS Code 扩展构建成功！

## 📦 生成的文件

```
chen-lang-0.2.0.vsix (139.42 KB)
```

位置: `/Users/chen/src/github.com/zuisong/chen_lang/vscode-extension/chen-lang-0.2.0.vsix`

## 🚀 安装方法

### 方法 1: 命令行安装（推荐）

```bash
code --install-extension chen-lang-0.2.0.vsix
```

### 方法 2: VS Code UI 安装

1. 打开 VS Code
2. 按 `Cmd+Shift+P` (Mac) 或 `Ctrl+Shift+P` (Windows/Linux)
3. 输入并选择: **Extensions: Install from VSIX...**
4. 选择 `chen-lang-0.2.0.vsix` 文件
5. 重新加载窗口

## ✅ 验证安装

### 1. 检查扩展是否安装

1. 打开 VS Code
2. 点击左侧扩展图标
3. 搜索 "Chen Lang"
4. 应该看到已安装的扩展

### 2. 测试功能

打开测试文件:
```bash
code ../lsp/test_features.ch
```

尝试以下功能:

- **语法高亮** ✅ - 关键字应该有颜色
- **实时检查** ✅ - 语法错误会显示红色波浪线
- **代码补全** ✅ - 输入 `l` 然后按 `Ctrl+Space`
- **悬停提示** ✅ - 鼠标悬停在关键字上
- **跳转定义** ✅ - `Cmd+点击` 函数名或变量
- **查找引用** ✅ - 右键 > "查找所有引用"
- **重命名符号** ✅ - 右键 > "重命名符号" 或按 `F2`
- **文档大纲** ✅ - 按 `Cmd+Shift+O`

## 📋 包含的文件

扩展包含以下文件:
- `out/extension.js` (720 KB) - 编译后的扩展代码（包含 LSP 客户端）
- `syntaxes/chen.tmLanguage.json` - 语法高亮规则
- `language-configuration.json` - 语言配置
- `package.json` - 扩展元数据
- `README.md` - 说明文档

## 🔧 配置 LSP 服务器路径

如果 `chen_lang_lsp` 不在 PATH 中，需要配置路径:

1. 打开 VS Code 设置 (`Cmd+,`)
2. 搜索 "Chen Lang"
3. 设置 `Chen Lang: Lsp Path`:
   ```
   /Users/chen/.cargo/bin/chen_lang_lsp
   ```

或在 `settings.json` 中添加:
```json
{
  "chenLang.lsp.path": "/Users/chen/.cargo/bin/chen_lang_lsp"
}
```

## 🐛 故障排除

### 扩展未激活

**症状**: 打开 `.ch` 文件没有语法高亮

**解决**:
1. 重新加载窗口: `Cmd+Shift+P` > "Developer: Reload Window"
2. 检查扩展是否启用
3. 查看开发者工具: `Help` > `Toggle Developer Tools`

### LSP 功能不工作

**症状**: 没有代码补全、跳转等功能

**解决**:
1. 检查 LSP 服务器是否安装:
   ```bash
   which chen_lang_lsp
   ```

2. 查看输出面板:
   - `View` > `Output`
   - 选择 "Chen Lang Language Server"

3. 配置正确的 LSP 路径（见上方）

### 查看日志

启用详细日志:
1. 打开设置
2. 搜索 "Chen Lang: Trace Server"
3. 设置为 "verbose"
4. 重新加载窗口
5. 查看输出面板

## 📦 分享扩展

这个 `.vsix` 文件可以分享给其他人使用:

1. 将 `chen-lang-0.2.0.vsix` 发送给其他人
2. 他们使用相同的安装方法安装
3. 确保他们也安装了 `chen_lang_lsp`

## 🔄 更新扩展

如果修改了扩展代码:

1. 重新构建:
   ```bash
   cd vscode-extension
   bun run compile
   bun run package
   ```

2. 卸载旧版本:
   ```bash
   code --uninstall-extension chen-lang.chen-lang
   ```

3. 安装新版本:
   ```bash
   code --install-extension chen-lang-0.2.0.vsix
   ```

## 🎯 下一步

1. ✅ 安装扩展
2. ✅ 打开 `.ch` 文件测试
3. ✅ 享受完整的 IDE 支持！

---

**构建信息**:
- 构建工具: Bun
- 扩展大小: 139.42 KB
- 主文件大小: 720 KB (包含所有依赖)
- 构建时间: 2026-01-04

**技术栈**:
- TypeScript
- Bun (构建和打包)
- vscode-languageclient (LSP 客户端)
- @vscode/vsce (打包工具)
