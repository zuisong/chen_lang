#![deny(missing_docs)]

use thiserror::Error;

use crate::expression::{
    Assign, Ast, BinaryOperation, Expression, FunctionCall, FunctionDeclaration, If, Literal, Local, Loop, Return,
    Statement, TryCatch, Unary,
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
    line: u32,
}

impl Parser {
    /// Create a new parser from a vector of tokens.
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            line: 1,
        }
    }

    /// Parse the tokens into an AST (list of statements).
    pub fn parse(&mut self) -> Result<Ast, ParseError> {
        self.parse_block()
    }

    // --- Helper Methods ---

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
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
            let token = &self.tokens[self.current];
            match token {
                Token::NewLine => self.line += 1,
                Token::String(s) => {
                    let newlines = s.chars().filter(|&c| c == '\n').count();
                    self.line += newlines as u32;
                }
                _ => {}
            }
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
            (Token::HashLBig, Token::HashLBig) => true,
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
            Err(ParseError::Message(message.to_string()))
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
        let start_line = self.line;
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
            return Ok(Statement::Break(start_line));
        }
        if self.match_token(&Token::Keyword(Keyword::CONTINUE)) {
            return Ok(Statement::Continue(start_line));
        }
        if self.match_token(&Token::Keyword(Keyword::TRY)) {
            return self.parse_try_catch();
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
                Expression::Identifier(name, _) => Ok(Statement::Assign(Assign {
                    name,
                    expr: Box::new(value),
                    line: start_line,
                })),
                Expression::GetField { object, field, .. } => Ok(Statement::SetField {
                    object: *object,
                    field,
                    value,
                    line: start_line,
                }),
                Expression::Index { object, index, .. } => Ok(Statement::SetIndex {
                    object: *object,
                    index: *index,
                    value,
                    line: start_line,
                }),
                _ => Err(ParseError::Message("Invalid assignment target".to_string())),
            };
        }

        Ok(Statement::Expression(expr))
    }

    fn parse_declare(&mut self) -> Result<Statement, ParseError> {
        let start_line = self.line;
        let name = if let Some(Token::Identifier(name)) = self.advance() {
            name.clone()
        } else {
            return Err(ParseError::Message(
                "Expected variable name after 'let'வுகளை".to_string(),
            ));
        };

        self.consume(&Token::Operator(Operator::Assign), "Expected '=' after variable name")?;

        let expr = self.parse_expression_logic()?;

        Ok(Statement::Local(Local {
            name,
            expression: expr,
            line: start_line,
        }))
    }

    fn parse_return(&mut self) -> Result<Statement, ParseError> {
        let start_line = self.line;
        let expr = self.parse_expression_logic()?;
        Ok(Statement::Return(Return {
            expression: expr,
            line: start_line,
        }))
    }

    fn parse_if(&mut self) -> Result<Expression, ParseError> {
        let start_line = self.line;
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
            line: start_line,
        }))
    }

    fn parse_for(&mut self) -> Result<Statement, ParseError> {
        let start_line = self.line;
        let condition = self.parse_expression_logic()?;

        self.skip_newlines();
        self.consume(&Token::LBig, "Expected '{' after for condition")?;
        let body = self.parse_block()?;
        self.consume(&Token::RBig, "Expected '}' after for block")?;

        Ok(Statement::Loop(Loop {
            test: condition,
            body,
            line: start_line,
        }))
    }

    fn parse_function_definition(&mut self) -> Result<FunctionDeclaration, ParseError> {
        let start_line = self.line;
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
            line: start_line,
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

    fn parse_expression_logic(&mut self) -> Result<Expression, ParseError> {
        let start_line = self.line;
        self.skip_newlines();
        if self.match_token(&Token::LBig) {
            let stmts = self.parse_block()?;
            self.consume(&Token::RBig, "Expected '}' after block")?;
            return Ok(Expression::Block(stmts, start_line));
        }

        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Result<Expression, ParseError> {
        let start_line = self.line;
        let mut left = self.parse_logical_and()?;

        while self.match_token(&Token::Operator(Operator::Or)) {
            let right = self.parse_logical_and()?;
            left = Expression::BinaryOperation(BinaryOperation {
                left: Box::new(left),
                operator: Operator::Or,
                right: Box::new(right),
                line: start_line, // Using start of OR chain approx
            });
        }
        Ok(left)
    }

    fn parse_logical_and(&mut self) -> Result<Expression, ParseError> {
        let start_line = self.line;
        let mut left = self.parse_equality()?;

        while self.match_token(&Token::Operator(Operator::And)) {
            let right = self.parse_equality()?;
            left = Expression::BinaryOperation(BinaryOperation {
                left: Box::new(left),
                operator: Operator::And,
                right: Box::new(right),
                line: start_line,
            });
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expression, ParseError> {
        let start_line = self.line;
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
                    line: start_line,
                });
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression, ParseError> {
        let start_line = self.line;
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
                    line: start_line,
                });
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expression, ParseError> {
        let start_line = self.line;
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
                    line: start_line,
                });
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expression, ParseError> {
        let start_line = self.line;
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
                    line: start_line,
                });
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, ParseError> {
        let start_line = self.line;
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
                    line: start_line,
                }))
            } else {
                Ok(Expression::BinaryOperation(BinaryOperation {
                    left: Box::new(Expression::Literal(Literal::Value(Value::Int(0)), start_line)),
                    operator: Operator::Subtract,
                    right: Box::new(right),
                    line: start_line,
                }))
            };
        }
        self.parse_postfix_expr()
    }

    fn parse_postfix_expr(&mut self) -> Result<Expression, ParseError> {
        let start_line = self.line;
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
                    line: start_line,
                });
            } else if self.match_token(&Token::Dot) {
                if let Some(Token::Identifier(field)) = self.advance() {
                    expr = Expression::GetField {
                        object: Box::new(expr),
                        field: field.clone(),
                        line: self.line,
                    };
                } else {
                    return Err(ParseError::Message("Expected identifier after '.'".to_string()));
                }
            } else if self.match_token(&Token::LSquare) {
                self.skip_newlines();
                let index = self.parse_expression_logic()?;
                self.skip_newlines();
                self.consume(&Token::RSquare, "Expected ']' after index")?;
                expr = Expression::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                    line: self.line,
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        let start_line = self.line;
        self.skip_newlines();
        let token = self.advance().ok_or(ParseError::UnexpectedEndOfInput)?.clone();

        match token {
            Token::Int(i) => Ok(Expression::Literal(Literal::Value(Value::Int(i)), start_line)),
            Token::Float(f) => Ok(Expression::Literal(Literal::Value(Value::Float(f)), start_line)),
            Token::Bool(b) => Ok(Expression::Literal(Literal::Value(Value::Bool(b)), start_line)),
            Token::String(s) => Ok(Expression::Literal(Literal::Value(Value::string(s)), start_line)),
            Token::Identifier(name) => Ok(Expression::Identifier(name, start_line)),
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
        let start_line = self.line;
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
        Ok(Expression::ObjectLiteral(fields, start_line))
    }

    fn parse_array_literal(&mut self) -> Result<Expression, ParseError> {
        let start_line = self.line;
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
        Ok(Expression::ArrayLiteral(elements, start_line))
    }

    fn parse_try_catch(&mut self) -> Result<Statement, ParseError> {
        let start_line = self.line;

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
            line: start_line,
        }))
    }

    fn parse_throw(&mut self) -> Result<Statement, ParseError> {
        let start_line = self.line;
        let value = self.parse_expression_logic()?;

        Ok(Statement::Throw {
            value,
            line: start_line,
        })
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Ast, ParseError> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}
