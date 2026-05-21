use std::collections::HashMap;

use crate::expression::{
    Assign, Ast, BinaryOperation, Expression, ForInLoop, FunctionCall,
    FunctionDeclaration, If, Literal, Local, Loop, Parameter,
    Return, Statement, TryCatch, TypeAnnotation, Unary,
};
use crate::tokenizer::{Keyword, Location, Operator, Token};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Unexpected token: {token:?} at {loc}")]
    UnexpectedToken { token: Token, loc: Location },
    #[error("Unexpected end of input")]
    UnexpectedEndOfInput,
    #[error("Parse error: {msg} at {loc}")]
    Message { msg: String, loc: Location },
}

/// The Parser struct manages the state of parsing a stream of tokens.
pub struct Parser {
    tokens: Vec<(Token, Location)>,
    current: usize,
    type_aliases: HashMap<String, TypeAnnotation>,
}

impl Parser {
    pub fn new(tokens: Vec<(Token, Location)>) -> Self {
        Parser {
            tokens,
            current: 0,
            type_aliases: HashMap::new(),
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current).map(|(t, _)| t)
    }

    fn peek_location(&self) -> Location {
        self.tokens
            .get(self.current)
            .map(|(_, l)| *l)
            .unwrap_or_else(|| {
                if self.tokens.is_empty() {
                    Location::default()
                } else {
                    let last_loc = self.tokens.last().unwrap().1;
                    Location {
                        index: last_loc.index + 1,
                        line: last_loc.line,
                        col: last_loc.col + 1,
                    }
                }
            })
    }

    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.tokens.get(self.current - 1).map(|(t, _)| t)
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
            (Token::Arrow, Token::Arrow) => true,
            (Token::Pipe, Token::Pipe) => true,
            (Token::Ampersand, Token::Ampersand) => true,
            (Token::Dot, Token::Dot) => true,
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

        if self.match_token(&Token::Keyword(Keyword::FUNCTION)) {
            return self.parse_function();
        }
        if self.match_token(&Token::Keyword(Keyword::WHILE)) {
            return self.parse_while();
        }
        if self.match_token(&Token::Keyword(Keyword::IF)) {
            return Ok(Statement::Expression(self.parse_if()?));
        }
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

        // Type annotation
        let mut type_annotation = None;
        if self.match_token(&Token::Colon) {
            type_annotation = Some(self.parse_type_annotation()?);
        }

        self.consume(
            &Token::Operator(Operator::Assign),
            "Expected '=' after variable name",
        )?;
        let expression = self.parse_expression_logic()?;

        Ok(Statement::Local(Local {
            name,
            type_annotation,
            expression,
            loc: name_loc,
        }))
    }

    fn parse_type_annotation(&mut self) -> Result<TypeAnnotation, ParseError> {
        self.skip_newlines();

        let mut annotations = Vec::new();

        loop {
            let annotation = if self.match_token(&Token::Keyword(Keyword::INT)) {
                TypeAnnotation::Int
            } else if self.match_token(&Token::Keyword(Keyword::FLOAT)) {
                TypeAnnotation::Float
            } else if self.match_token(&Token::Keyword(Keyword::BOOL)) {
                TypeAnnotation::Bool
            } else if self.match_token(&Token::Keyword(Keyword::STRING)) {
                TypeAnnotation::String
            } else if self.match_token(&Token::Keyword(Keyword::OBJECT)) {
                TypeAnnotation::Object
            } else if self.match_token(&Token::Keyword(Keyword::NULL)) {
                TypeAnnotation::Null
            } else if let Some(Token::Identifier(name)) = self.peek() {
                let name = name.clone();
                self.advance();

                if self.match_token(&Token::Operator(Operator::Lt)) {
                    let mut arguments = Vec::new();
                    loop {
                        arguments.push(self.parse_type_annotation()?);
                        if !self.match_token(&Token::COMMA) {
                            break;
                        }
                    }
                    self.consume(
                        &Token::Operator(Operator::Gt),
                        "Expected '>' after generic type arguments",
                    )?;
                    TypeAnnotation::Generic { name, arguments }
                } else {
                    TypeAnnotation::Named(name)
                }
            } else {
                return Err(ParseError::Message {
                    msg: "Expected type annotation".to_string(),
                    loc: self.peek_location(),
                });
            };

            annotations.push(annotation);

            if !self.match_token(&Token::Pipe) {
                break;
            }
        }

        if annotations.len() == 1 {
            Ok(annotations.remove(0))
        } else {
            Ok(TypeAnnotation::Union(annotations))
        }
    }

    fn parse_for(&mut self) -> Result<Statement, ParseError> {
        let start_loc = self.peek_location();

        // Check if it's a for-of loop: for (let x of iterable)
        if self.match_token(&Token::LParen) {
            if self.match_token(&Token::Keyword(Keyword::LET)) {
                let var = self.consume_identifier("Expected variable name in for-of loop")?;
                self.consume(&Token::Keyword(Keyword::OF), "Expected 'of' in for-of loop")?;
                let iterable = self.parse_expression_logic()?;
                self.consume(&Token::RParen, "Expected ')' after for-of header")?;

                self.consume(&Token::LBig, "Expected '{' after for-of loop")?;
                let body = self.parse_block()?;
                self.consume(&Token::RBig, "Expected '}' after for-of block")?;

                return Ok(Statement::ForIn(ForInLoop {
                    var,
                    iterable,
                    body,
                    loc: start_loc,
                }));
            }
            return Err(ParseError::Message {
                msg: "Traditional for loops are not yet supported. Use for (let x of iterable)".to_string(),
                loc: start_loc,
            });
        }

        Err(ParseError::Message {
            msg: "Expected '(' after 'for'".to_string(),
            loc: start_loc,
        })
    }

    fn parse_while(&mut self) -> Result<Statement, ParseError> {
        let start_loc = self.peek_location();

        self.consume(&Token::LParen, "Expected '(' after 'while'")?;
        let condition = self.parse_expression_logic()?;
        self.consume(&Token::RParen, "Expected ')' after while condition")?;

        self.consume(&Token::LBig, "Expected '{' after while condition")?;
        let body = self.parse_block()?;
        self.consume(&Token::RBig, "Expected '}' after while block")?;

        Ok(Statement::Loop(Loop {
            test: condition,
            body,
            loc: start_loc,
        }))
    }

    fn parse_function(&mut self) -> Result<Statement, ParseError> {
        let decl = self.parse_function_definition()?;
        Ok(Statement::FunctionDeclaration(decl))
    }

    fn parse_function_definition(&mut self) -> Result<FunctionDeclaration, ParseError> {
        let loc = self.peek_location();

        // Optional name
        let mut name = None;
        if let Some(Token::Identifier(n)) = self.peek() {
            name = Some(n.clone());
            self.advance();
        }

        self.consume(&Token::LParen, "Expected '(' after function name")?;
        let mut parameters = Vec::new();
        if !self.check(&Token::RParen) {
            loop {
                let param_loc = self.peek_location();
                let name = self.consume_identifier("Expected parameter name")?;

                let mut type_annotation = None;
                if self.match_token(&Token::Colon) {
                    type_annotation = Some(self.parse_type_annotation()?);
                }

                parameters.push(Parameter {
                    name,
                    type_annotation,
                    loc: param_loc,
                });
                if !self.match_token(&Token::COMMA) {
                    break;
                }
            }
        }
        self.consume(&Token::RParen, "Expected ')' after parameters")?;

        // Return type
        let mut return_type = None;
        if self.match_token(&Token::Arrow) {
            return_type = Some(self.parse_type_annotation()?);
        }

        self.consume(&Token::LBig, "Expected '{' before function body")?;
        let body = self.parse_block()?;
        self.consume(&Token::RBig, "Expected '}' after function body")?;

        Ok(FunctionDeclaration {
            name,
            parameters,
            return_type,
            body,
            loc,
        })
    }

    fn parse_if(&mut self) -> Result<Expression, ParseError> {
        let start_loc = self.peek_location();

        self.consume(&Token::LParen, "Expected '(' after 'if'")?;
        let condition = self.parse_expression_logic()?;
        self.consume(&Token::RParen, "Expected ')' after if condition")?;

        self.consume(&Token::LBig, "Expected '{' after if condition")?;
        let body = self.parse_block()?;
        self.consume(&Token::RBig, "Expected '}' after if block")?;

        let mut else_body = Vec::new();
        if self.match_token(&Token::Keyword(Keyword::ELSE)) {
            if self.match_token(&Token::Keyword(Keyword::IF)) {
                let inner_if = self.parse_if()?;
                else_body.push(Statement::Expression(inner_if));
            } else {
                self.consume(&Token::LBig, "Expected '{' after 'else'")?;
                else_body = self.parse_block()?;
                self.consume(&Token::RBig, "Expected '}' after else block")?;
            }
        }

        Ok(Expression::If(If {
            test: Box::new(condition),
            body,
            else_body,
            loc: start_loc,
        }))
    }

    fn parse_return(&mut self) -> Result<Statement, ParseError> {
        let loc = self.peek_location();
        let expr = if self.check(&Token::NewLine) || self.check(&Token::RBig) || self.is_at_end() {
            Expression::Literal(Literal::Value(crate::value::Value::Null), loc)
        } else {
            self.parse_expression_logic()?
        };
        Ok(Statement::Return(Return {
            expression: expr,
            loc,
        }))
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

        // Catch variable with parentheses: catch (error)
        let error_name = if self.match_token(&Token::LParen) {
            let name = self.consume_identifier("Expected variable name after '(' in catch")?;
            self.consume(&Token::RParen, "Expected ')' after catch variable")?;
            Some(name)
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
        let loc = self.peek_location();
        let value = self.parse_expression_logic()?;
        Ok(Statement::Throw { value, loc })
    }

    fn parse_expression_logic(&mut self) -> Result<Expression, ParseError> {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_logical_and()?;

        while self.match_token(&Token::Operator(Operator::Or)) {
            self.skip_newlines();
            let loc = self.peek_location();
            let right = self.parse_logical_and()?;
            left = Expression::BinaryOperation(BinaryOperation {
                left: Box::new(left),
                operator: Operator::Or,
                right: Box::new(right),
                loc,
            });
        }

        Ok(left)
    }

    fn parse_logical_and(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_equality()?;

        while self.match_token(&Token::Operator(Operator::And)) {
            self.skip_newlines();
            let loc = self.peek_location();
            let right = self.parse_equality()?;
            left = Expression::BinaryOperation(BinaryOperation {
                left: Box::new(left),
                operator: Operator::And,
                right: Box::new(right),
                loc,
            });
        }

        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_comparison()?;

        while self.check(&Token::Operator(Operator::Equals))
            || self.check(&Token::Operator(Operator::NotEquals))
        {
            let operator = match self.advance().unwrap() {
                Token::Operator(Operator::Equals) => Operator::Equals,
                Token::Operator(Operator::NotEquals) => Operator::NotEquals,
                _ => unreachable!(),
            };
            self.skip_newlines();
            let loc = self.peek_location();
            let right = self.parse_comparison()?;
            left = Expression::BinaryOperation(BinaryOperation {
                left: Box::new(left),
                operator,
                right: Box::new(right),
                loc,
            });
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_term()?;

        while self.check(&Token::Operator(Operator::Gt))
            || self.check(&Token::Operator(Operator::GtE))
            || self.check(&Token::Operator(Operator::Lt))
            || self.check(&Token::Operator(Operator::LtE))
        {
            let operator = match self.advance().unwrap() {
                Token::Operator(Operator::Gt) => Operator::Gt,
                Token::Operator(Operator::GtE) => Operator::GtE,
                Token::Operator(Operator::Lt) => Operator::Lt,
                Token::Operator(Operator::LtE) => Operator::LtE,
                _ => unreachable!(),
            };
            self.skip_newlines();
            let loc = self.peek_location();
            let right = self.parse_term()?;
            left = Expression::BinaryOperation(BinaryOperation {
                left: Box::new(left),
                operator,
                right: Box::new(right),
                loc,
            });
        }

        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_factor()?;

        while self.check(&Token::Operator(Operator::Add))
            || self.check(&Token::Operator(Operator::Subtract))
        {
            let operator = match self.advance().unwrap() {
                Token::Operator(Operator::Add) => Operator::Add,
                Token::Operator(Operator::Subtract) => Operator::Subtract,
                _ => unreachable!(),
            };
            self.skip_newlines();
            let loc = self.peek_location();
            let right = self.parse_factor()?;
            left = Expression::BinaryOperation(BinaryOperation {
                left: Box::new(left),
                operator,
                right: Box::new(right),
                loc,
            });
        }

        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_unary()?;

        while self.check(&Token::Operator(Operator::Multiply))
            || self.check(&Token::Operator(Operator::Divide))
            || self.check(&Token::Operator(Operator::Mod))
        {
            let operator = match self.advance().unwrap() {
                Token::Operator(Operator::Multiply) => Operator::Multiply,
                Token::Operator(Operator::Divide) => Operator::Divide,
                Token::Operator(Operator::Mod) => Operator::Mod,
                _ => unreachable!(),
            };
            self.skip_newlines();
            let loc = self.peek_location();
            let right = self.parse_unary()?;
            left = Expression::BinaryOperation(BinaryOperation {
                left: Box::new(left),
                operator,
                right: Box::new(right),
                loc,
            });
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, ParseError> {
        if self.check(&Token::Operator(Operator::Not))
            || self.check(&Token::Operator(Operator::Subtract))
        {
            let loc = self.peek_location();
            let operator = match self.advance().unwrap() {
                Token::Operator(Operator::Not) => Operator::Not,
                Token::Operator(Operator::Subtract) => Operator::Subtract,
                _ => unreachable!(),
            };
            let expr = self.parse_unary()?;
            return Ok(Expression::Unary(Unary {
                operator,
                expr: Box::new(expr),
                loc,
            }));
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_atom()?;

        loop {
            if self.match_token(&Token::LParen) {
                let loc = self.peek_location();
                let mut arguments = Vec::new();
                if !self.check(&Token::RParen) {
                    loop {
                        arguments.push(self.parse_expression_logic()?);
                        if !self.match_token(&Token::COMMA) {
                            break;
                        }
                    }
                }
                self.consume(&Token::RParen, "Expected ')' after arguments")?;
                expr = Expression::FunctionCall(FunctionCall {
                    callee: Box::new(expr),
                    arguments,
                    loc,
                });
            } else if self.match_token(&Token::Dot) {
                let loc = self.peek_location();
                let field = self.consume_identifier("Expected field name after '.'")?;
                expr = Expression::GetField {
                    object: Box::new(expr),
                    field,
                    loc,
                };
            } else if self.match_token(&Token::LSquare) {
                let loc = self.peek_location();
                let index = self.parse_expression_logic()?;
                self.consume(&Token::RSquare, "Expected ']' after index")?;
                expr = Expression::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                    loc,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_atom(&mut self) -> Result<Expression, ParseError> {
        let loc = self.peek_location();
        if let Some(token) = self.advance() {
            match token {
                Token::Int(i) => Ok(Expression::Literal(
                    Literal::Value(crate::value::Value::Int(*i)),
                    loc,
                )),
                Token::Float(f) => Ok(Expression::Literal(
                    Literal::Value(crate::value::Value::Float(*f)),
                    loc,
                )),
                Token::Bool(b) => Ok(Expression::Literal(
                    Literal::Value(crate::value::Value::Bool(*b)),
                    loc,
                )),
                Token::String(s) => Ok(Expression::Literal(
                    Literal::Value(crate::value::Value::string(s.clone())),
                    loc,
                )),
                Token::Keyword(Keyword::NULL) => Ok(Expression::Literal(
                    Literal::Value(crate::value::Value::Null),
                    loc,
                )),
                Token::Keyword(Keyword::THIS) => Ok(Expression::Identifier("this".to_string(), loc)),
                Token::Identifier(name) => Ok(Expression::Identifier(name.clone(), loc)),
                Token::LParen => {
                    let expr = self.parse_expression_logic()?;
                    self.consume(&Token::RParen, "Expected ')' after expression")?;
                    Ok(expr)
                }
                Token::LBig => {
                    self.skip_newlines();
                    // Detect if this is an object literal: { key: value } or {}
                    let is_object = if self.check(&Token::RBig) {
                        true
                    } else if let Some(Token::Identifier(_)) | Some(Token::String(_)) | Some(Token::Int(_)) | Some(Token::Float(_)) = self.peek() {
                        // Look ahead for colon
                        let mut lookahead = self.current + 1;
                        while lookahead < self.tokens.len() && matches!(self.tokens[lookahead].0, Token::NewLine) {
                            lookahead += 1;
                        }
                        lookahead < self.tokens.len() && matches!(self.tokens[lookahead].0, Token::Colon)
                    } else {
                        false
                    };

                    if is_object {
                        return self.parse_object_literal(loc);
                    }
                    
                    let body = self.parse_block()?;
                    self.consume(&Token::RBig, "Expected '}' after block")?;
                    Ok(Expression::Block(body, loc))
                }
                Token::LSquare => {
                    let mut elements = Vec::new();
                    if !self.check(&Token::RSquare) {
                        loop {
                            elements.push(self.parse_expression_logic()?);
                            if !self.match_token(&Token::COMMA) {
                                break;
                            }
                        }
                    }
                    self.consume(&Token::RSquare, "Expected ']' after array literal")?;
                    Ok(Expression::ArrayLiteral(elements, loc))
                }
                Token::Keyword(Keyword::FUNCTION) => {
                    let decl = self.parse_function_definition()?;
                    Ok(Expression::Function(decl))
                }
                Token::Keyword(Keyword::IF) => {
                    self.parse_if()
                }
                _ => Err(ParseError::UnexpectedToken {
                    token: token.clone(),
                    loc,
                }),
            }
        } else {
            Err(ParseError::UnexpectedEndOfInput)
        }
    }

    fn parse_object_literal(&mut self, loc: Location) -> Result<Expression, ParseError> {
        let mut fields = Vec::new();
        self.skip_newlines();
        while !self.check(&Token::RBig) {
            self.skip_newlines();
            let key = match self.peek() {
                Some(Token::Identifier(name)) => {
                    let name = name.clone();
                    self.advance();
                    name
                }
                Some(Token::String(s)) => {
                    let s = s.clone();
                    self.advance();
                    s
                }
                Some(Token::Int(i)) => {
                    let s = i.to_string();
                    self.advance();
                    s
                }
                Some(Token::Float(f)) => {
                    let s = f.to_string();
                    self.advance();
                    s
                }
                _ => {
                    return Err(ParseError::Message {
                        msg: "Expected field name in object literal".to_string(),
                        loc: self.peek_location(),
                    });
                }
            };

            self.skip_newlines();
            self.consume(&Token::Colon, "Expected ':' after field name")?;
            self.skip_newlines();
            let value = self.parse_expression_logic()?;
            fields.push((key, value));

            self.skip_newlines();
            if !self.match_token(&Token::COMMA) {
                self.skip_newlines();
                break;
            }
            self.skip_newlines();
        }
        self.skip_newlines();
        self.consume(&Token::RBig, "Expected '}' after object literal")?;
        Ok(Expression::ObjectLiteral(fields, loc))
    }

    fn consume_identifier(&mut self, message: &str) -> Result<String, ParseError> {
        if let Some(Token::Identifier(name)) = self.peek() {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(ParseError::Message {
                msg: message.to_string(),
                loc: self.peek_location(),
            })
        }
    }
}

pub fn parse(tokens: Vec<(Token, Location)>) -> Result<Ast, ParseError> {
    let mut parser = Parser::new(tokens);
    let mut ast = Vec::new();

    while !parser.is_at_end() {
        parser.skip_newlines();
        if parser.is_at_end() {
            break;
        }
        ast.push(parser.parse_statement()?);
        parser.skip_newlines();
    }

    Ok(ast)
}
