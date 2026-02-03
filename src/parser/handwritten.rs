#![deny(missing_docs)]

//! 手写递归下降解析器模块
//!
//! 提供基于 Token 流的手写解析器实现。

use thiserror::Error;

use crate::expression::{
    Assign, Ast, BinaryOperation, Expression, ForInLoop, FunctionCall, FunctionDeclaration, If, Literal, Local, Loop,
    MethodCall, Return, Statement, TryCatch, Unary,
};
use crate::tokenizer::Keyword;
use crate::tokenizer::Location;
use crate::tokenizer::Operator;
use crate::tokenizer::Token;
use crate::value::Value;

#[derive(Error, Debug)]
/// 语法分析错误
pub enum ParseError {
    /// 通用错误消息
    #[error("Line {loc}: {msg}")]
    Message {
        /// 错误消息内容
        msg: String,
        /// 发生错误的位置
        loc: Location,
    },
    /// 遇到意外的 Token
    #[error("Line {loc}: Unexpected token: {token:?}")]
    UnexpectedToken {
        /// 遇到的 Token
        token: Token,
        /// 发生错误的位置
        loc: Location,
    },
    /// 意外的输入结束
    #[error("Unexpected end of input")]
    UnexpectedEndOfInput,
}

/// The Parser struct manages the state of parsing a stream of tokens.
pub struct Parser {
    tokens: Vec<(Token, Location)>,
    current: usize,
}

impl Parser {
    /// Create a new parser from a vector of tokens.
    pub fn new(tokens: Vec<(Token, Location)>) -> Self {
        Self { tokens, current: 0 }
    }

    /// Parse the tokens into an AST (list of statements).
    pub fn parse(&mut self) -> Result<Ast, ParseError> {
        self.parse_block()
    }

    // --- Helper Methods ---

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current).map(|(t, _)| t)
    }

    fn peek_location(&self) -> Location {
        if self.current < self.tokens.len() {
            self.tokens[self.current].1
        } else {
            self.tokens.last().map(|(_, l)| *l).unwrap_or_default()
        }
    }

    fn previous(&self) -> Option<&Token> {
        if self.current > 0 {
            self.tokens.get(self.current - 1).map(|(t, _)| t)
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
            (Token::DollarLBig, Token::DollarLBig) => true,
            (Token::NewLine, Token::NewLine) => true,
            (Token::COMMA, Token::COMMA) => true,
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
            Err(ParseError::Message {
                msg: message.to_string(),
                loc: self.peek_location(),
            })
        }
    }

    fn skip_newlines(&mut self) {
        while self.match_token(&Token::NewLine) {}
    }

    // --- Parsing Logic ---

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

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        let start_loc = self.peek_location();
        if self.match_token(&Token::Keyword(Keyword::LET)) {
            return self.parse_declare();
        }
        if self.match_token(&Token::Keyword(Keyword::FOR)) {
            return self.parse_for();
        }

        if self.match_token(&Token::Keyword(Keyword::DEF)) {
            return self.parse_function();
        }
        // Async check removed
        if self.match_token(&Token::Keyword(Keyword::RETURN)) {
            return self.parse_return();
        }
        if self.match_token(&Token::Keyword(Keyword::BREAK)) {
            return Ok(Statement::Break(start_loc));
        }
        if self.match_token(&Token::Keyword(Keyword::CONTINUE)) {
            return Ok(Statement::Continue(start_loc));
        }
        if self.match_token(&Token::Keyword(Keyword::TRY)) {
            return self.parse_try_catch();
        }
        if self.match_token(&Token::Keyword(Keyword::THROW)) {
            return self.parse_throw();
        }
        if self.match_token(&Token::Keyword(Keyword::THROW)) {
            return self.parse_throw();
        }

        // Assignment or Expression
        let expr = self.parse_expression_logic()?;

        // Check if it is an assignment
        if self.match_token(&Token::Operator(Operator::Assign)) {
            let value = self.parse_expression_logic()?;
            return match expr {
                Expression::Identifier(name, loc) => Ok(Statement::Assign(Assign {
                    name,
                    expr: Box::new(value),
                    loc,
                })),
                Expression::GetField { object, field, loc } => Ok(Statement::SetField {
                    object: *object,
                    field,
                    value,
                    loc,
                }),
                Expression::Index { object, index, loc } => Ok(Statement::SetIndex {
                    object: *object,
                    index: *index,
                    value,
                    loc,
                }),
                _ => Err(ParseError::Message {
                    msg: "Invalid assignment target".to_string(),
                    loc: self.peek_location(),
                }),
            };
        }

        Ok(Statement::Expression(expr))
    }

    fn parse_declare(&mut self) -> Result<Statement, ParseError> {
        let name_loc = self.peek_location();
        let name = if let Some(Token::Identifier(name)) = self.advance() {
            name.clone()
        } else {
            return Err(ParseError::Message {
                msg: "Expected variable name after 'let'".to_string(),
                loc: self.peek_location(),
            });
        };

        self.consume(&Token::Operator(Operator::Assign), "Expected '=' after variable name")?;

        let expr = self.parse_expression_logic()?;

        Ok(Statement::Local(Local {
            name,
            expression: expr,
            loc: name_loc,
        }))
    }

    fn parse_return(&mut self) -> Result<Statement, ParseError> {
        let start_loc = self.peek_location();
        let expr = self.parse_expression_logic()?;
        Ok(Statement::Return(Return {
            expression: expr,
            loc: start_loc,
        }))
    }

    fn parse_if(&mut self) -> Result<Expression, ParseError> {
        let start_loc = self.peek_location();
        let condition = self.parse_expression_logic()?;

        self.skip_newlines();
        self.consume(&Token::LBig, "Expected '{' after if condition")?;
        let then_branch = self.parse_block()?;
        self.consume(&Token::RBig, "Expected '}' after if block")?;

        let mut else_branch = Vec::new();
        self.skip_newlines();
        if self.match_token(&Token::Keyword(Keyword::ELSE)) {
            self.skip_newlines();
            if self.check(&Token::Keyword(Keyword::IF)) {
                self.advance(); // Consume 'if'
                let next_if = self.parse_if()?;
                else_branch = vec![Statement::Expression(next_if)];
            } else {
                self.consume(&Token::LBig, "Expected '{' after else")?;
                else_branch = self.parse_block()?;
                self.consume(&Token::RBig, "Expected '}' after else block")?;
            }
        }

        Ok(Expression::If(If {
            test: Box::new(condition),
            body: then_branch,
            else_body: else_branch,
            loc: start_loc,
        }))
    }

    fn parse_for(&mut self) -> Result<Statement, ParseError> {
        let start_loc = self.peek_location();

        // 1. Check for infinite loop: for { ... }
        if self.check(&Token::LBig) {
            self.advance(); // consume '{'
            let body = self.parse_block()?;
            self.consume(&Token::RBig, "Expected '}' after for block")?;
            return Ok(Statement::Loop(Loop {
                test: Expression::Literal(Literal::Value(Value::Bool(true)), start_loc),
                body,
                loc: start_loc,
            }));
        }

        // 2. Check for for-in or while-style:
        // We look ahead to see if it's an Identifier followed by IN
        let is_for_in = if let Some(Token::Identifier(_)) = self.peek() {
            self.tokens
                .get(self.current + 1)
                .map(|(t, _)| t == &Token::Keyword(Keyword::IN))
                .unwrap_or(false)
        } else {
            false
        };

        if is_for_in {
            let var_name = if let Some(Token::Identifier(name)) = self.advance() {
                name.clone()
            } else {
                unreachable!()
            };

            self.consume(&Token::Keyword(Keyword::IN), "Expected 'in' after variable name")?;
            let iterable = self.parse_expression_logic()?;

            self.skip_newlines();
            self.consume(&Token::LBig, "Expected '{' after for-in iterable")?;
            let body = self.parse_block()?;
            self.consume(&Token::RBig, "Expected '}' after for-in block")?;

            Ok(Statement::ForIn(ForInLoop {
                var: var_name,
                iterable,
                body,
                loc: start_loc,
            }))
        } else {
            // While style: for condition { ... }
            let condition = self.parse_expression_logic()?;

            self.skip_newlines();
            self.consume(&Token::LBig, "Expected '{' after for condition")?;
            let body = self.parse_block()?;
            self.consume(&Token::RBig, "Expected '}' after for block")?;

            Ok(Statement::Loop(Loop {
                test: condition,
                body,
                loc: start_loc,
            }))
        }
    }

    fn parse_function_definition(&mut self) -> Result<FunctionDeclaration, ParseError> {
        let start_loc = self.peek_location();
        let (name, name_loc) = if let Some(Token::Identifier(name)) = self.peek() {
            let n = name.clone();
            let nloc = self.peek_location();
            self.advance();
            (Some(n), Some(nloc))
        } else {
            (None, None)
        };

        self.consume(&Token::LParen, "Expected '(' after function name")?;

        let mut parameters = Vec::new();
        if !self.check(&Token::RParen) {
            loop {
                if let Some(Token::Identifier(param)) = self.advance() {
                    parameters.push(param.clone());
                } else {
                    return Err(ParseError::Message {
                        msg: "Expected parameter name".to_string(),
                        loc: self.peek_location(),
                    });
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
            loc: name_loc.unwrap_or(start_loc),
        })
    }

    fn parse_function(&mut self) -> Result<Statement, ParseError> {
        let decl = self.parse_function_definition()?;
        if decl.name.is_none() {
            return Err(ParseError::Message {
                msg: "Function declaration as statement must have a name".to_string(),
                loc: self.peek_location(),
            });
        }
        Ok(Statement::FunctionDeclaration(decl))
    }

    fn parse_expression_logic(&mut self) -> Result<Expression, ParseError> {
        let start_loc = self.peek_location();
        self.skip_newlines();
        if self.match_token(&Token::LBig) {
            let stmts = self.parse_block()?;
            self.consume(&Token::RBig, "Expected '}' after block")?;
            return Ok(Expression::Block(stmts, start_loc));
        }

        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Result<Expression, ParseError> {
        let start_loc = self.peek_location();
        let mut left = self.parse_logical_and()?;

        while self.match_token(&Token::Operator(Operator::Or)) {
            let right = self.parse_logical_and()?;
            left = Expression::BinaryOperation(BinaryOperation {
                left: Box::new(left),
                operator: Operator::Or,
                right: Box::new(right),
                loc: start_loc, // Using start of OR chain approx
            });
        }
        Ok(left)
    }

    fn parse_logical_and(&mut self) -> Result<Expression, ParseError> {
        let start_loc = self.peek_location();
        let mut left = self.parse_equality()?;

        while self.match_token(&Token::Operator(Operator::And)) {
            let right = self.parse_equality()?;
            left = Expression::BinaryOperation(BinaryOperation {
                left: Box::new(left),
                operator: Operator::And,
                right: Box::new(right),
                loc: start_loc,
            });
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expression, ParseError> {
        let start_loc = self.peek_location();
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
                    loc: start_loc,
                });
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression, ParseError> {
        let start_loc = self.peek_location();
        let mut left = self.parse_term()?;

        while let Some(Token::Operator(op)) = self.peek() {
            if matches!(op, Operator::Gt | Operator::GtE | Operator::Lt | Operator::LtE) {
                let op = *op;
                self.advance();
                let right = self.parse_term()?;
                left = Expression::BinaryOperation(BinaryOperation {
                    left: Box::new(left),
                    operator: op,
                    right: Box::new(right),
                    loc: start_loc,
                });
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expression, ParseError> {
        let start_loc = self.peek_location();
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
                    loc: start_loc,
                });
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expression, ParseError> {
        let start_loc = self.peek_location();
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
                    loc: start_loc,
                });
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, ParseError> {
        let start_loc = self.peek_location();
        if let Some(Token::Operator(op)) = self.peek()
            && matches!(op, Operator::Not | Operator::Subtract)
        {
            let op = *op;
            self.advance();
            let right = self.parse_unary()?;
            return if op == Operator::Not {
                Ok(Expression::Unary(Unary {
                    operator: Operator::Not,
                    expr: Box::new(right),
                    loc: start_loc,
                }))
            } else {
                Ok(Expression::BinaryOperation(BinaryOperation {
                    left: Box::new(Expression::Literal(Literal::Value(Value::Int(0)), start_loc)),
                    operator: Operator::Subtract,
                    right: Box::new(right),
                    loc: start_loc,
                }))
            };
        }
        // Await check removed
        self.parse_postfix_expr()
    }

    fn parse_postfix_expr(&mut self) -> Result<Expression, ParseError> {
        let start_loc = self.peek_location();
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(&Token::LParen) {
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
                    loc: start_loc,
                });
            } else if self.match_token(&Token::Dot) {
                if let Some(Token::Identifier(field)) = self.advance() {
                    expr = Expression::GetField {
                        object: Box::new(expr),
                        field: field.clone(),
                        loc: self.peek_location(),
                    };
                } else {
                    return Err(ParseError::Message {
                        msg: "Expected identifier after '.'".to_string(),
                        loc: self.peek_location(),
                    });
                }
            } else if self.match_token(&Token::LSquare) {
                self.skip_newlines();
                let index = self.parse_expression_logic()?;
                self.skip_newlines();
                self.consume(&Token::RSquare, "Expected ']' after index")?;
                expr = Expression::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                    loc: self.peek_location(),
                };
            } else if self.match_token(&Token::Colon) {
                if let Some(Token::Identifier(method)) = self.advance() {
                    let method_name = method.clone();
                    self.skip_newlines();
                    self.consume(&Token::LParen, "Expected '(' after method name")?;
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

                    expr = Expression::MethodCall(MethodCall {
                        object: Box::new(expr),
                        method: method_name,
                        arguments: args,
                        loc: self.peek_location(),
                    });
                } else {
                    return Err(ParseError::Message {
                        msg: "Expected method name after ':'".to_string(),
                        loc: self.peek_location(),
                    });
                }
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        let start_loc = self.peek_location();
        self.skip_newlines();
        let token = self.advance().ok_or(ParseError::UnexpectedEndOfInput)?.clone();

        match token {
            Token::Int(i) => Ok(Expression::Literal(Literal::Value(Value::Int(i)), start_loc)),
            Token::Float(f) => Ok(Expression::Literal(Literal::Value(Value::Float(f)), start_loc)),
            Token::Bool(b) => Ok(Expression::Literal(Literal::Value(Value::Bool(b)), start_loc)),
            Token::String(s) => Ok(Expression::Literal(Literal::Value(Value::string(s)), start_loc)),
            Token::Identifier(name) => Ok(Expression::Identifier(name, start_loc)),
            Token::DollarLBig => self.parse_object_literal(),
            Token::LSquare => self.parse_array_literal(),
            Token::Keyword(Keyword::IF) => self.parse_if(),
            Token::Keyword(Keyword::DEF) => {
                let decl = self.parse_function_definition()?;
                Ok(Expression::Function(decl))
            }
            // ASYNC check removed
            Token::Keyword(Keyword::IMPORT) => {
                // import("path")
                self.skip_newlines();
                self.consume(&Token::LParen, "Expected '(' after 'import'")?;
                self.skip_newlines();
                if let Some(Token::String(s)) = self.advance() {
                    let path = s.clone();
                    self.skip_newlines();
                    self.consume(&Token::RParen, "Expected ')' after import path")?;
                    Ok(Expression::Import {
                        path,
                        loc: start_loc,
                    })
                } else {
                    Err(ParseError::Message {
                        msg: "Expected string path inside import(...)".to_string(),
                        loc: self.peek_location(),
                    })
                }
            }
            Token::LParen => {
                self.skip_newlines();
                let expr = self.parse_expression_logic()?;
                self.skip_newlines();
                self.consume(&Token::RParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            _ => Err(ParseError::UnexpectedToken { token, loc: start_loc }),
        }
    }

    fn parse_object_literal(&mut self) -> Result<Expression, ParseError> {
        let start_loc = self.peek_location();
        let mut fields = Vec::new();
        self.skip_newlines();
        if !self.check(&Token::RBig) {
            loop {
                self.skip_newlines();
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
                        return Err(ParseError::Message {
                            msg: "Expected field name (identifier or integer)".to_string(),
                            loc: self.peek_location(),
                        });
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
        Ok(Expression::ObjectLiteral(fields, start_loc))
    }

    fn parse_array_literal(&mut self) -> Result<Expression, ParseError> {
        let start_loc = self.peek_location();
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
        Ok(Expression::ArrayLiteral(elements, start_loc))
    }

    fn parse_try_catch(&mut self) -> Result<Statement, ParseError> {
        let start_loc = self.peek_location();

        // Parse try block
        self.skip_newlines();
        self.consume(&Token::LBig, "Expected '{' after 'try'")?;
        let try_body = self.parse_block()?;
        self.consume(&Token::RBig, "Expected '}' after try block")?;

        // Parse catch
        self.skip_newlines();
        self.consume(&Token::Keyword(Keyword::CATCH), "Expected 'catch' after try block")?;

        // Optional error variable name
        let error_name = if let Some(Token::Identifier(name)) = self.peek() {
            let n = name.clone();
            self.advance();
            Some(n)
        } else {
            None
        };

        // Parse catch block
        self.skip_newlines();
        self.consume(&Token::LBig, "Expected '{' after 'catch'")?;
        let catch_body = self.parse_block()?;
        self.consume(&Token::RBig, "Expected '}' after catch block")?;

        // Optional finally block
        self.skip_newlines();
        let finally_body = if self.match_token(&Token::Keyword(Keyword::FINALLY)) {
            self.skip_newlines();
            self.consume(&Token::LBig, "Expected '{' after 'finally'")?;
            let body = self.parse_block()?;
            self.consume(&Token::RBig, "Expected '}' after finally block")?;
            Some(body)
        } else {
            None
        };

        Ok(Statement::TryCatch(TryCatch {
            try_body,
            error_name,
            catch_body,
            finally_body,
            loc: start_loc,
        }))
    }

    fn parse_throw(&mut self) -> Result<Statement, ParseError> {
        let start_loc = self.peek_location();
        let value = self.parse_expression_logic()?;

        Ok(Statement::Throw { value, loc: start_loc })
    }
}

/// 解析 Token 流为 AST
///
/// # 参数
/// - `tokens`: 带有位置信息的 Token 列表
///
/// # 返回
/// - `Ok(Ast)`: 解析成功
/// - `Err(ParseError)`: 解析失败
pub fn parse(tokens: Vec<(Token, Location)>) -> Result<Ast, ParseError> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}
