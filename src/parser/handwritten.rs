#![deny(missing_docs)]

use thiserror::Error;

use crate::expression::{
    Assign, Ast, BinaryOperation, Expression, FunctionCall, FunctionDeclaration, If, Literal,
    Local, Loop, Return, Statement, Unary,
};
use crate::token::Keyword;
use crate::token::Operator;
use crate::token::Token;
use crate::value::Value;

#[derive(Error, Debug)]
/// 语法分析错误
pub enum ParseError {
    /// 通用错误消息
    #[error("{0}")]
    Message(String),
    /// 遇到意外的 Token
    #[error("Unexpected token: {0:?}")]
    UnexpectedToken(Token),
    /// 意外的输入结束
    #[error("Unexpected end of input")]
    UnexpectedEndOfInput,
}

/// The Parser struct manages the state of parsing a stream of tokens.
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    /// Create a new parser from a vector of tokens.
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    /// Parse the tokens into an AST (list of statements).
    ///
    /// 此方法是 `Parser` 结构体的主要解析方法。它通过调用 `parse_block()` 方法
    /// 来处理顶层的语句块，从而构建整个程序的抽象语法树 (AST)。
    ///
    /// **解析流程概览：**
    /// 1.  从 Token 序列中逐个读取 Token。
    /// 2.  根据 Token 的类型和上下文，递归地调用不同的 `parse_*` 方法。
    /// 3.  每成功解析一个语法单元（如语句、表达式），就构建一个对应的 AST 节点。
    /// 4.  将这些 AST 节点组织成最终的 AST 结构。
    /// 5.  整个过程类似于一个"状态机"，通过 `current` 指针和函数调用栈维护解析状态。
    pub fn parse(&mut self) -> Result<Ast, ParseError> {
        self.parse_block()
    }

    // --- Helper Methods ---

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn peek_next(&self) -> Option<&Token> {
        self.tokens.get(self.current + 1)
    }

    fn previous(&self) -> Option<&Token> {
        if self.current > 0 {
            self.tokens.get(self.current - 1)
        } else {
            None
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn check(&self, token_type: &Token) -> bool {
        if self.is_at_end() {
            return false;
        }
        // Note: This is a loose check, might need refinement for enum variants with data
        // For now, we'll use discriminants or simple matching where possible.
        // Since Token derives PartialEq, we can check exact matches for keywords/ops.
        // For variants with data (Int, String), we might need `matches!`.
        match (self.peek().unwrap(), token_type) {
            (Token::Keyword(k1), Token::Keyword(k2)) => k1 == k2,
            (Token::Operator(o1), Token::Operator(o2)) => o1 == o2,
            (Token::LBig, Token::LBig) => true,
            (Token::RBig, Token::RBig) => true,
            (Token::LParen, Token::LParen) => true,
            (Token::RParen, Token::RParen) => true,
            (Token::LSquare, Token::LSquare) => true,
            (Token::RSquare, Token::RSquare) => true,
            (Token::Colon, Token::Colon) => true,
            (Token::Dot, Token::Dot) => true,
            (Token::HashLBig, Token::HashLBig) => true,
            (Token::NewLine, Token::NewLine) => true,
            (Token::COMMA, Token::COMMA) => true,
            // Add more as needed
            _ => false,
        }
    }

    fn match_token(&mut self, token_type: &Token) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, token_type: &Token, message: &str) -> Result<&Token, ParseError> {
        if self.check(token_type) {
            Ok(self.advance().unwrap())
        } else {
            Err(ParseError::Message(message.to_string()))
        }
    }

    fn skip_newlines(&mut self) {
        while self.match_token(&Token::NewLine) {}
    }

    // --- Parsing Logic ---

    /// 解析一个语句块（例如，函数体、`if` 或 `for` 语句后面的 `{...}` 部分）。
    ///
    /// **工作流程：**
    /// 1.  创建一个空的 `statements` 列表，用于存放解析出的语句 AST 节点。
    /// 2.  循环处理 Token，直到文件结束或遇到右大括号 `}`。
    ///     *   首先跳过所有空行 (`skip_newlines`)。
    ///     *   如果仍然没有结束且未遇到 `}`，则调用 `parse_statement()` 解析单个语句。
    ///     *   将解析出的语句添加到 `statements` 列表。
    ///     *   再次跳过空行。
    /// 3.  返回包含所有语句 AST 节点的列表。
    ///
    /// **流程图简述：**
    /// 开始 -> [初始化语句列表] -> 循环: [跳过空行] -> [判断是否结束或遇到 '}' ?]
    /// -> 是: 结束循环 -> 否: [解析单个语句 (parse_statement)] -> [添加语句到列表]
    /// -> [跳过空行] -> 返回语句列表 -> 结束
    fn parse_block(&mut self) -> Result<Ast, ParseError> {
        let mut statements = Vec::new();

        while !self.is_at_end() && !self.check(&Token::RBig) {
            self.skip_newlines();
            if self.is_at_end() || self.check(&Token::RBig) {
                break;
            }
            statements.push(self.parse_statement()?);
            self.skip_newlines();
        }

        Ok(statements)
    }

    /// 解析一个单独的语句。
    ///
    /// **工作流程：**
    /// 1.  **关键字识别**: 检查当前 Token 是否是 `let`, `for`, `def`, `return`, `break`, `continue` 等关键字。
    ///     *   如果是，则根据关键字类型，调用对应的解析方法（如 `parse_declare`, `parse_for`, `parse_function` 等）。
    ///     *   解析完成后，返回相应的 `Statement` AST 节点。
    /// 2.  **赋值或表达式语句**: 如果不是关键字，则检查当前 Token 是否可能是赋值语句或纯表达式语句。
    ///     *   通过 `peek()` 和 `peek_next()` 进行前瞻，判断是否为 `Identifier` 后跟 `Operator::Assign`。
    ///     *   如果是赋值语句，解析右侧表达式并构建 `Statement::Assign` 节点。
    ///     *   如果不是赋值语句，则将整个内容解析为一个表达式，构建 `Statement::Expression` 节点。
    ///
    /// **流程图简述：**
    /// 开始 -> [匹配 'let' ?] -> 是: [parse_declare] -> 否
    /// -> [匹配 'for' ?] -> 是: [parse_for] -> 否
    /// -> [匹配 'def' ?] -> 是: [parse_function] -> 否
    /// -> [匹配 'return' ?] -> 是: [parse_return] -> 否
    /// -> [匹配 'break' ?] -> 是: [Statement::Break] -> 否
    /// -> [匹配 'continue' ?] -> 是: [Statement::Continue] -> 否
    /// -> [前瞻: Identifier + Assign ?] -> 是: [解析赋值语句] -> 否
    /// -> [解析表达式 (parse_expression_logic)] -> 返回 Statement -> 结束
    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        if self.match_token(&Token::Keyword(Keyword::LET)) {
            return self.parse_declare();
        }
        if self.match_token(&Token::Keyword(Keyword::FOR)) {
            return self.parse_for();
        }

        if self.match_token(&Token::Keyword(Keyword::DEF)) {
            return self.parse_function();
        }
        if self.match_token(&Token::Keyword(Keyword::RETURN)) {
            return self.parse_return();
        }
        if self.match_token(&Token::Keyword(Keyword::BREAK)) {
            return Ok(Statement::Break);
        }
        if self.match_token(&Token::Keyword(Keyword::CONTINUE)) {
            return Ok(Statement::Continue);
        }

        // Assignment or Expression
        // Parse the expression first (which could be an l-value)
        let expr = self.parse_expression_logic()?;

        // Check if it is an assignment
        if self.match_token(&Token::Operator(Operator::Assign)) {
            let value = self.parse_expression_logic()?;
            return match expr {
                Expression::Identifier(name) => Ok(Statement::Assign(Assign {
                    name,
                    expr: Box::new(value),
                })),
                Expression::GetField { object, field } => Ok(Statement::SetField {
                    object: *object,
                    field,
                    value,
                }),
                Expression::Index { object, index } => Ok(Statement::SetIndex {
                    object: *object,
                    index: *index,
                    value,
                }),
                _ => Err(ParseError::Message("Invalid assignment target".to_string())),
            };
        }

        Ok(Statement::Expression(expr))
    }

    fn parse_declare(&mut self) -> Result<Statement, ParseError> {
        // let x = ...
        let name = if let Some(Token::Identifier(name)) = self.advance() {
            name.clone()
        } else {
            return Err(ParseError::Message(
                "Expected variable name after 'let'".to_string(),
            ));
        };

        self.consume(
            &Token::Operator(Operator::Assign),
            "Expected '=' after variable name",
        )?;

        let expr = self.parse_expression_logic()?;

        Ok(Statement::Local(Local {
            name,
            expression: expr,
        }))
    }

    fn parse_return(&mut self) -> Result<Statement, ParseError> {
        let expr = self.parse_expression_logic()?;
        Ok(Statement::Return(Return { expression: expr }))
    }

    fn parse_if(&mut self) -> Result<Expression, ParseError> {
        // if condition { ... } else { ... }
        let condition = self.parse_expression_logic()?;

        self.skip_newlines();
        self.consume(&Token::LBig, "Expected '{' after if condition")?;
        let then_branch = self.parse_block()?;
        self.consume(&Token::RBig, "Expected '}' after if block")?;

        let mut else_branch = Vec::new();
        self.skip_newlines();
        if self.match_token(&Token::Keyword(Keyword::ELSE)) {
            self.skip_newlines();
            self.consume(&Token::LBig, "Expected '{' after else")?;
            else_branch = self.parse_block()?;
            self.consume(&Token::RBig, "Expected '}' after else block")?;
        }

        Ok(Expression::If(If {
            test: Box::new(condition),
            body: then_branch,
            else_body: else_branch,
        }))
    }

    fn parse_for(&mut self) -> Result<Statement, ParseError> {
        // for condition { ... }
        let condition = self.parse_expression_logic()?;

        self.skip_newlines();
        self.consume(&Token::LBig, "Expected '{' after for condition")?;
        let body = self.parse_block()?;
        self.consume(&Token::RBig, "Expected '}' after for block")?;

        Ok(Statement::Loop(Loop {
            test: condition,
            body,
        }))
    }

    fn parse_function_definition(&mut self) -> Result<FunctionDeclaration, ParseError> {
        let name = if let Some(Token::Identifier(name)) = self.peek() {
            let n = name.clone();
            self.advance();
            Some(n)
        } else {
            None
        };

        self.consume(&Token::LParen, "Expected '(' after function name")?;

        let mut parameters = Vec::new();
        if !self.check(&Token::RParen) {
            loop {
                if let Some(Token::Identifier(param)) = self.advance() {
                    parameters.push(param.clone());
                } else {
                    return Err(ParseError::Message("Expected parameter name".to_string()));
                }

                if !self.match_token(&Token::COMMA) {
                    break;
                }
            }
        }
        self.consume(&Token::RParen, "Expected ')' after parameters")?;

        self.skip_newlines();
        self.consume(&Token::LBig, "Expected '{' before function body")?;
        let body = self.parse_block()?;
        self.consume(&Token::RBig, "Expected '}' after function body")?;

        Ok(FunctionDeclaration {
            name,
            parameters,
            body,
        })
    }

    fn parse_function(&mut self) -> Result<Statement, ParseError> {
        let decl = self.parse_function_definition()?;
        if decl.name.is_none() {
            return Err(ParseError::Message(
                "Function declaration as statement must have a name".to_string(),
            ));
        }
        Ok(Statement::FunctionDeclaration(decl))
    }

    // --- Expression Parsing ---
    // Using precedence climbing or a simplified Pratt parser approach.
    // For simplicity, reusing the existing logic but adapting it to stream.
    // Actually, the existing `parse_expression` was shunting-yard on a single line.
    // We should implement a proper recursive descent or precedence climbing here.

    /// 解析各种表达式，这是表达式解析的入口点。
    /// 该方法通过递归下降和运算符优先级解析（Precedence Climbing 或类似 Pratt 解析器的思想）来处理。
    ///
    /// **工作流程：**
    /// 1.  **块表达式**: 首先检查是否为 `{ ... }` 形式的块表达式，如果是则直接解析为 `Expression::Block`。
    /// 2.  **运算符优先级**: 按照运算符的优先级，从最低优先级（逻辑或 `||`）开始，逐级调用对应的解析方法。
    ///     *   `parse_logical_or` -> `parse_logical_and` -> `parse_equality` -> `parse_comparison` -> `parse_term` -> `parse_factor` -> `parse_unary` -> `parse_primary`。
    ///     *   每个方法负责处理特定优先级的运算符，并通过循环和递归调用来构建二元/一元运算的 AST 节点。
    ///
    /// **流程图简述：**
    /// 开始 -> [跳过空行] -> [匹配 '{' ?] -> 是: [parse_block] -> [匹配 '}' ] -> 返回 Block Expression -> 否:
    /// -> [parse_logical_or (处理 ||)] -> [parse_logical_and (处理 &&)] -> [parse_equality (处理 ==, !=)]
    /// -> [parse_comparison (处理 <, <=, >, >=)] -> [parse_term (处理 +, -)]
    /// -> [parse_factor (处理 *, /, %)] -> [parse_unary (处理 !, -)] -> [parse_primary (处理原子表达式)]
    /// -> 返回 Expression -> 结束
    fn parse_expression_logic(&mut self) -> Result<Expression, ParseError> {
        // Handle Block Expression first: { ... }
        self.skip_newlines();
        if self.match_token(&Token::LBig) {
            let stmts = self.parse_block()?;
            self.consume(&Token::RBig, "Expected '}' after block")?;
            return Ok(Expression::Block(stmts));
        }

        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_logical_and()?;

        while self.match_token(&Token::Operator(Operator::Or)) {
            let right = self.parse_logical_and()?;
            left = Expression::BinaryOperation(BinaryOperation {
                left: Box::new(left),
                operator: Operator::Or,
                right: Box::new(right),
            });
        }
        Ok(left)
    }

    fn parse_logical_and(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_equality()?;

        while self.match_token(&Token::Operator(Operator::And)) {
            let right = self.parse_equality()?;
            left = Expression::BinaryOperation(BinaryOperation {
                left: Box::new(left),
                operator: Operator::And,
                right: Box::new(right),
            });
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_comparison()?;

        while let Some(Token::Operator(op)) = self.peek() {
            if matches!(op, Operator::Equals | Operator::NotEquals) {
                let op = *op;
                self.advance();
                let right = self.parse_comparison()?;
                left = Expression::BinaryOperation(BinaryOperation {
                    left: Box::new(left),
                    operator: op,
                    right: Box::new(right),
                });
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_term()?;

        while let Some(Token::Operator(op)) = self.peek() {
            if matches!(
                op,
                Operator::Gt | Operator::GtE | Operator::Lt | Operator::LtE
            ) {
                let op = *op;
                self.advance();
                let right = self.parse_term()?;
                left = Expression::BinaryOperation(BinaryOperation {
                    left: Box::new(left),
                    operator: op,
                    right: Box::new(right),
                });
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_factor()?;

        while let Some(Token::Operator(op)) = self.peek() {
            if matches!(op, Operator::Add | Operator::Subtract) {
                let op = *op;
                self.advance();
                let right = self.parse_factor()?;
                left = Expression::BinaryOperation(BinaryOperation {
                    left: Box::new(left),
                    operator: op,
                    right: Box::new(right),
                });
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_unary()?;

        while let Some(Token::Operator(op)) = self.peek() {
            if matches!(op, Operator::Multiply | Operator::Divide | Operator::Mod) {
                let op = *op;
                self.advance();
                let right = self.parse_unary()?;
                left = Expression::BinaryOperation(BinaryOperation {
                    left: Box::new(left),
                    operator: op,
                    right: Box::new(right),
                });
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, ParseError> {
        if let Some(Token::Operator(op)) = self.peek() {
            if matches!(op, Operator::Not | Operator::Subtract) {
                let op = *op;
                self.advance();
                let right = self.parse_unary()?;
                return if op == Operator::Not {
                    Ok(Expression::Unary(Unary {
                        operator: Operator::Not,
                        expr: Box::new(right),
                    }))
                } else {
                    // Unary minus is 0 - expr
                    Ok(Expression::BinaryOperation(BinaryOperation {
                        left: Box::new(Expression::Literal(Literal::Value(Value::Int(0)))),
                        operator: Operator::Subtract,
                        right: Box::new(right),
                    }))
                };
            }
        }
        self.parse_postfix_expr()
    }

    fn parse_postfix_expr(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(&Token::LParen) {
                // Function Call: expr(...)
                let mut args = Vec::new();
                self.skip_newlines();
                if !self.check(&Token::RParen) {
                    loop {
                        self.skip_newlines();
                        args.push(self.parse_expression_logic()?);
                        self.skip_newlines();
                        if !self.match_token(&Token::COMMA) {
                            break;
                        }
                    }
                }
                self.skip_newlines();
                self.consume(&Token::RParen, "Expected ')' after arguments")?;

                expr = Expression::FunctionCall(FunctionCall {
                    callee: Box::new(expr),
                    arguments: args,
                });
            } else if self.match_token(&Token::Dot) {
                // GetField: expr.field
                if let Some(Token::Identifier(field)) = self.advance() {
                    expr = Expression::GetField {
                        object: Box::new(expr),
                        field: field.clone(),
                    };
                } else {
                    return Err(ParseError::Message(
                        "Expected identifier after '.'".to_string(),
                    ));
                }
            } else if self.match_token(&Token::LSquare) {
                // Index: expr[index]
                self.skip_newlines();
                let index = self.parse_expression_logic()?;
                self.skip_newlines();
                self.consume(&Token::RSquare, "Expected ']' after index")?;
                expr = Expression::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        self.skip_newlines();
        let token = self
            .advance()
            .ok_or(ParseError::UnexpectedEndOfInput)?
            .clone();

        match token {
            Token::Int(i) => Ok(Expression::Literal(Literal::Value(Value::Int(i)))),
            Token::Float(f) => Ok(Expression::Literal(Literal::Value(Value::Float(f)))),
            Token::Bool(b) => Ok(Expression::Literal(Literal::Value(Value::Bool(b)))),
            Token::String(s) => Ok(Expression::Literal(Literal::Value(Value::string(s)))),
            Token::Identifier(name) => Ok(Expression::Identifier(name)),
            Token::HashLBig => self.parse_object_literal(),
            Token::LSquare => self.parse_array_literal(),
            Token::Keyword(Keyword::IF) => self.parse_if(),
            Token::Keyword(Keyword::DEF) => {
                let decl = self.parse_function_definition()?;
                Ok(Expression::Function(decl))
            }
            Token::LParen => {
                self.skip_newlines();
                let expr = self.parse_expression_logic()?;
                self.skip_newlines();
                self.consume(&Token::RParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            _ => Err(ParseError::UnexpectedToken(token)),
        }
    }

    fn parse_object_literal(&mut self) -> Result<Expression, ParseError> {
        // #{ key: val, key2: val2 } or #{ 0: val, 1: val2 }
        let mut fields = Vec::new();
        self.skip_newlines();
        if !self.check(&Token::RBig) {
            loop {
                self.skip_newlines();
                // 支持标识符或整数作为键
                let key = match self.peek() {
                    Some(Token::Identifier(name)) => {
                        let k = name.clone();
                        self.advance();
                        k
                    }
                    Some(Token::Int(n)) => {
                        let k = n.to_string();
                        self.advance();
                        k
                    }
                    _ => {
                        return Err(ParseError::Message(
                            "Expected field name (identifier or integer)".to_string(),
                        ));
                    }
                };

                self.consume(&Token::Colon, "Expected ':' after field name")?;
                let val = self.parse_expression_logic()?;
                fields.push((key, val));

                self.skip_newlines();
                if !self.match_token(&Token::COMMA) {
                    break;
                }
            }
        }
        self.skip_newlines();
        self.consume(&Token::RBig, "Expected '}' after object fields")?;
        Ok(Expression::ObjectLiteral(fields))
    }

    fn parse_array_literal(&mut self) -> Result<Expression, ParseError> {
        // [ expr1, expr2 ]
        let mut elements = Vec::new();
        self.skip_newlines();
        if !self.check(&Token::RSquare) {
            loop {
                self.skip_newlines();
                let expr = self.parse_expression_logic()?;
                elements.push(expr);

                self.skip_newlines();
                if !self.match_token(&Token::COMMA) {
                    break;
                }
            }
        }
        self.skip_newlines();
        self.consume(&Token::RSquare, "Expected ']' after array elements")?;
        Ok(Expression::ArrayLiteral(elements))
    }
}

// Keep the old function signature for compatibility with lib.rs for now,
// but we will update lib.rs to use Parser directly.
// Actually, let's just expose a helper function that matches the old signature roughly,
// or better, just update lib.rs.
// But for now, let's provide a `parse` function that takes tokens.

/// Parse a vector of tokens into an AST.
///
/// 这是整个语法分析过程的入口函数。它首先创建一个 `Parser` 实例，
/// 然后调用 `Parser` 实例的 `parse` 方法来启动递归下降解析过程。
/// 最终将 Token 序列转换为抽象语法树 (AST)。
pub fn parse(tokens: Vec<Token>) -> Result<Ast, ParseError> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}
