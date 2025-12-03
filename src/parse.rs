#![deny(missing_docs)]

use anyhow::{Result, anyhow};

use crate::value::Value;
use crate::*;

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
    pub fn parse(&mut self) -> Result<Ast> {
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

    fn consume(&mut self, token_type: &Token, message: &str) -> Result<&Token> {
        if self.check(token_type) {
            Ok(self.advance().unwrap())
        } else {
            Err(anyhow!(message.to_string()))
        }
    }

    fn skip_newlines(&mut self) {
        while self.match_token(&Token::NewLine) {}
    }

    // --- Parsing Logic ---

    fn parse_block(&mut self) -> Result<Ast> {
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

    fn parse_statement(&mut self) -> Result<Statement> {
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

        // Assignment or Expression
        // We need to look ahead to distinguish `x = 1` (Assign) from `x + 1` (Expression)
        // Or `func()` (Expression)

        // Simple heuristic: if it starts with Identifier and next is Assign, it's assignment.
        if let Some(Token::Identifier(name)) = self.peek() {
            if let Some(Token::Operator(Operator::Assign)) = self.peek_next() {
                let name = name.clone();
                self.advance(); // consume identifier
                self.advance(); // consume =
                let expr = self.parse_expression_logic()?;
                return Ok(Statement::Assign(Assign {
                    name,
                    expr: Box::new(expr),
                }));
            }
        }

        let expr = self.parse_expression_logic()?;
        Ok(Statement::Expression(expr))
    }

    fn parse_declare(&mut self) -> Result<Statement> {
        // let x = ...
        let name = if let Some(Token::Identifier(name)) = self.advance() {
            name.clone()
        } else {
            return Err(anyhow!("Expected variable name after 'let'"));
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

    fn parse_return(&mut self) -> Result<Statement> {
        let expr = self.parse_expression_logic()?;
        Ok(Statement::Return(Return { expression: expr }))
    }

    fn parse_if(&mut self) -> Result<Expression> {
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

    fn parse_for(&mut self) -> Result<Statement> {
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

    fn parse_function(&mut self) -> Result<Statement> {
        // def name(arg1, arg2) { ... }
        let name = if let Some(Token::Identifier(name)) = self.advance() {
            name.clone()
        } else {
            return Err(anyhow!("Expected function name after 'def'"));
        };

        self.consume(&Token::LParen, "Expected '(' after function name")?;

        let mut parameters = Vec::new();
        if !self.check(&Token::RParen) {
            loop {
                if let Some(Token::Identifier(param)) = self.advance() {
                    parameters.push(param.clone());
                } else {
                    return Err(anyhow!("Expected parameter name"));
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

        Ok(Statement::FunctionDeclaration(FunctionDeclaration {
            name,
            parameters,
            body,
        }))
    }

    // --- Expression Parsing ---
    // Using precedence climbing or a simplified Pratt parser approach.
    // For simplicity, reusing the existing logic but adapting it to stream.
    // Actually, the existing `parse_expression` was shunting-yard on a single line.
    // We should implement a proper recursive descent or precedence climbing here.

    fn parse_expression_logic(&mut self) -> Result<Expression> {
        // Handle Block Expression first: { ... }
        self.skip_newlines();
        if self.match_token(&Token::LBig) {
            let stmts = self.parse_block()?;
            self.consume(&Token::RBig, "Expected '}' after block")?;
            return Ok(Expression::Block(stmts));
        }

        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Result<Expression> {
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

    fn parse_logical_and(&mut self) -> Result<Expression> {
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

    fn parse_equality(&mut self) -> Result<Expression> {
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

    fn parse_comparison(&mut self) -> Result<Expression> {
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

    fn parse_term(&mut self) -> Result<Expression> {
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

    fn parse_factor(&mut self) -> Result<Expression> {
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

    fn parse_unary(&mut self) -> Result<Expression> {
        if let Some(Token::Operator(op)) = self.peek() {
            if matches!(op, Operator::Not | Operator::Subtract) {
                let op = *op;
                self.advance();
                let right = self.parse_unary()?;
                if op == Operator::Not {
                    return Ok(Expression::Unary(Unary {
                        operator: Operator::Not,
                        expr: Box::new(right),
                    }));
                } else {
                    // Unary minus is 0 - expr
                    return Ok(Expression::BinaryOperation(BinaryOperation {
                        left: Box::new(Expression::Literal(Literal::Value(Value::Int(0)))),
                        operator: Operator::Subtract,
                        right: Box::new(right),
                    }));
                }
            }
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expression> {
        self.skip_newlines();
        let token = self
            .advance()
            .ok_or(anyhow!("Unexpected end of input"))?
            .clone();

        match token {
            Token::Int(i) => Ok(Expression::Literal(Literal::Value(Value::Int(i)))),
            Token::Float(f) => Ok(Expression::Literal(Literal::Value(Value::Float(f)))),
            Token::Bool(b) => Ok(Expression::Literal(Literal::Value(Value::Bool(b)))),
            Token::String(s) => Ok(Expression::Literal(Literal::Value(Value::string(s)))),
            Token::Identifier(name) => {
                // Check for function call
                if self.check(&Token::LParen) {
                    self.advance(); // consume '('
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
                    Ok(Expression::FunctionCall(FunctionCall {
                        name,
                        arguments: args,
                    }))
                } else {
                    Ok(Expression::Identifier(name))
                }
            }
            Token::Keyword(Keyword::IF) => self.parse_if(),
            Token::LParen => {
                self.skip_newlines();
                let expr = self.parse_expression_logic()?;
                self.skip_newlines();
                self.consume(&Token::RParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            _ => Err(anyhow!("Unexpected token: {:?}", token)),
        }
    }
}

// Keep the old function signature for compatibility with lib.rs for now,
// but we will update lib.rs to use Parser directly.
// Actually, let's just expose a helper function that matches the old signature roughly,
// or better, just update lib.rs.
// But for now, let's provide a `parse` function that takes tokens.

/// Parse a vector of tokens into an AST.
pub fn parse(tokens: Vec<Token>) -> Result<Ast> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}
