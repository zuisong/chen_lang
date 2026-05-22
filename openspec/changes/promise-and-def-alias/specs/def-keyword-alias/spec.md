## ADDED Requirements

### Requirement: def 关键字作为函数定义别名
解析器和分词器必须允许将 `def` 关键字作为 `function` 关键字的直接替代品。无论是定义命名函数还是匿名函数，在手写分词器和 Pest 解析器模式下都必须正常解析。

#### Scenario: 手写解析器下的 def 函数定义
- **WHEN** 在手写分词器模式下，代码中包含 `let f = def(x) { return x + 1 }`
- **THEN** 代码分词器必须正常生成 `Token::Keyword(Keyword::FUNCTION)`，并由语法解析器成功解析为 Function 声明

#### Scenario: Pest 解析器下的 def 函数定义
- **WHEN** 在 Pest 模式下，代码包含 `def main() {}` 并被解析
- **THEN** Pest 应当将 `def` 匹配为 `FUNCTION` 规则，并输出正确的 `Statement::FunctionDeclaration` 抽象语法树节点
