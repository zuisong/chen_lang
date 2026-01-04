# Chen Lang LSP 开发指南

## 快速开始

```bash
# 进入 LSP 目录
cd lsp

# 编译
cargo build

# 运行
cargo run

# 发布版本
cargo build --release
```

## 项目结构

```
lsp/
├── src/
│   ├── lib.rs          # 库入口
│   ├── server.rs       # LSP 服务器实现
│   ├── document.rs     # 文档管理和同步
│   └── bin.rs         # 可执行文件入口
├── Cargo.toml
├── README.md
├── SYNTAX_HIGHLIGHTING.md
├── test.ch           # 测试文件
├── vscode-settings.json
└── helix-languages.toml
```

## 功能实现

### 已实现 ✅

1. **文档同步**
   - did_open: 打开文件时加载
   - did_change: 增量更新
   - did_close: 关闭文件时清理

2. **诊断信息**
   - 词法错误检测
   - 语法错误检测
   - 实时错误提示
   - 集成 chen_lang 解析器

3. **代码补全**
   - 关键字补全 (let, def, if, else, for, return, try, catch, import 等)
   - 触发字符: `.`

4. **悬停提示**
   - 关键字文档
   - Markdown 格式显示

5. **文档符号**
   - 函数定义 (def)
   - 变量声明 (let)
   - 侧边栏大纲视图

6. **跳转定义** (Go to Definition)
   - 函数定义跳转
   - 变量声明跳转

7. **查找引用** (Find References)
   - 查找所有使用位置
   - 智能词边界检测

8. **重命名符号** (Rename Symbol)
   - 智能重命名
   - 自动更新所有引用

### 待实现 🚧

1. **代码格式化** (Formatting)
   - 缩进标准化
   - 空格规范化

2. **语义高亮** (Semantic Highlighting)
   - 变量类型识别
   - 函数参数/返回值高亮

3. **代码折叠** (Folding Ranges)
   - 函数体折叠
   - 代码块折叠

## 扩展开发

### 添加新的 LSP 功能

在 `server.rs` 中的 `ChenLangLsp` trait 中添加方法：

```rust
async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
    let uri = params.text_document_position_params.text_document.uri;
    
    // 实现跳转逻辑
    
    Ok(None)
}
```

### 添加新的诊断规则

在 `server.rs` 的 `analyze` 函数中：

```rust
fn analyze(source: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    
    // 现有的词法和语法检查
    
    // 添加新的检查规则
    if source.contains("TODO") {
        diagnostics.push(Diagnostic {
            severity: Some(DiagnosticSeverity::WARNING),
            message: "TODO found".to_string(),
            // ...
        });
    }
    
    diagnostics
}
```

## 调试

### 启用详细日志

```bash
RUST_LOG=chen_lang_lsp=debug cargo run
```

### 使用 VS Code 调试

创建 `.vscode/launch.json`：

```json
{
   "version": "0.2.0",
   "configurations": [
      {
         "type": "lldb",
         "request": "launch",
         "name": "Debug LSP",
         "cargo": {
            "args": ["build"],
            "filter": {
               "name": "chen_lang_lsp",
               "kind": "bin"
            }
         },
         "args": [],
         "cwd": "${workspaceFolder}/lsp"
      }
   ]
}
```

## 测试

### 手动测试

1. 启动 LSP 服务器：

```bash
cargo run
```

2. 在另一个终端测试：

```bash
# 测试诊断（故意写一个有错误的文件）
cat > test_error.ch << 'EOF'
def incomplete( {
    let x =
}
EOF
```

3. 在支持 LSP 的编辑器中打开 `.ch` 文件查看效果

### 自动化测试（未来）

```bash
# 运行单元测试
cargo test

# 运行集成测试
cargo test --test integration
```

## 性能优化建议

1. **增量解析**: 只重新解析修改的部分
2. **符号缓存**: 缓存符号表避免重复计算
3. **延迟诊断**: 使用防抖避免频繁诊断
4. **并行处理**: 利用多线程处理多个文件

## 贡献指南

欢迎提交 PR！开发流程：

1. Fork 项目
2. 创建功能分支: `git checkout -b feature/xxx`
3. 提交更改: `git commit -am 'Add xxx'`
4. 推送分支: `git push origin feature/xxx`
5. 提交 Pull Request

## 参考资料

- [LSP 规范](https://microsoft.github.io/language-server-protocol/)
- [tower-lsp 文档](https://docs.rs/tower-lsp/)
- [VS Code 扩展开发](https://code.visualstudio.com/api)
