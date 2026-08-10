#![deny(missing_docs)]

//! Hand-written recursive descent parser for Luau-style syntax.
//!
//! Supports:
//! - `local`, `function`, `while`, `repeat`, `for`, `do`, `if`, `return`, `break`, `continue`
//! - `try`/`catch` (compatibility)
//! - `and`, `or`, `not` keywords
//! - `nil` keyword (maps to internal `Value::Null`)
//! - `..` concat, `#` len, `~=` not-equals, `//` floor-div
//! - Compound assignments (`+=`, `-=`, etc.) desugared to simple assignment
//! - `{ key = value }` table literals, `[ ... ]` array literals
//! - `:method()` calls
//! - Variadic `...` parameters

use thiserror::Error;

use crate::expression::{
    Assign, AssignMulti, Ast, BinaryOperation, Expression, ForInLoop, FunctionCall, FunctionDeclaration, If, Literal,
    Local, LocalList, Loop, MethodCall, Repeat, Return, Statement, TryCatch, Unary,
};
use crate::tokenizer::Keyword;
use crate::tokenizer::Location;
use crate::tokenizer::Operator;
use crate::tokenizer::Token;
use crate::value::Value;

#[derive(Error, Debug)]
/// Syntax analysis errors
pub enum ParseError {
    /// Generic error with message
    #[error("Line {loc}: {msg}")]
    Message {
        /// Error message
        msg: String,
        /// Location of the error
        loc: Location,
    },
    /// Unexpected token
    #[error("Line {loc}: Unexpected token: {token:?}")]
    UnexpectedToken {
        /// The token encountered
        token: Token,
        /// Location of the token
        loc: Location,
    },
    /// Unexpected end of input
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
        self.parse_block(&[])
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
            (Token::COMMA, Token::COMMA) => true,
            (Token::NewLine, Token::NewLine) => true,
            (Token::Vararg, Token::Vararg) => true,
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

    fn match_keyword(&mut self, kw: Keyword) -> bool {
        self.match_token(&Token::Keyword(kw))
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

    // --- Block Parsing ---

    /// Parse a sequence of statements terminated by any of the given keyword(s),
    /// end of input, or `}` (for try/catch compatibility).
    fn parse_block(&mut self, terminators: &[Keyword]) -> Result<Vec<Statement>, ParseError> {
        let mut statements = Vec::new();
        loop {
            self.skip_newlines();
            if self.is_at_end() {
                break;
            }
            if self.check(&Token::RBig) {
                break;
            }
            if let Some(Token::Keyword(kw)) = self.peek() {
                if terminators.contains(kw) {
                    break;
                }
            }
            let stmts = self.parse_statement()?;
            statements.extend(stmts);
            self.skip_newlines();
        }
        Ok(statements)
    }

    // --- Statement Parsing ---

    fn parse_statement(&mut self) -> Result<Vec<Statement>, ParseError> {
        let start_loc = self.peek_location();

        if self.match_keyword(Keyword::LOCAL) {
            if self.check(&Token::Keyword(Keyword::FUNCTION)) {
                return self.parse_local_function();
            }
            return self.parse_local();
        }
        if self.match_keyword(Keyword::FUNCTION) {
            return self.parse_function();
        }
        if self.match_keyword(Keyword::WHILE) {
            return self.parse_while();
        }
        if self.match_keyword(Keyword::REPEAT) {
            return self.parse_repeat();
        }
        if self.match_keyword(Keyword::FOR) {
            return self.parse_for();
        }
        if self.match_keyword(Keyword::DO) {
            return self.parse_do();
        }
        if self.match_keyword(Keyword::IF) {
            let expr = self.parse_if_tail()?;
            return Ok(vec![Statement::Expression(expr)]);
        }
        if self.match_keyword(Keyword::RETURN) {
            return self.parse_return();
        }
        if self.match_keyword(Keyword::BREAK) {
            return Ok(vec![Statement::Break(start_loc)]);
        }
        if self.match_keyword(Keyword::CONTINUE) {
            return Ok(vec![Statement::Continue(start_loc)]);
        }
        if self.match_keyword(Keyword::TRY) {
            return self.parse_try_catch();
        }
        if self.match_keyword(Keyword::THEN) {
            // This is used when `then` appears unexpectedly (e.g. inside if
            // parsing), but we want a descriptive error rather than crashing.
            return Err(ParseError::Message {
                msg: "Unexpected 'then' outside of if statement".to_string(),
                loc: start_loc,
            });
        }

        let expr = self.parse_expression_logic()?;

        // 多目标赋值: `a, b = expr[, expr]*`
        if matches!(expr, Expression::Identifier(..)) && self.check(&Token::COMMA) {
            let mut names = Vec::new();
            if let Expression::Identifier(name, _) = expr {
                names.push(name);
            }
            while self.match_token(&Token::COMMA) {
                self.skip_newlines();
                let name = self.consume_identifier()?;
                names.push(name);
            }
            self.skip_newlines();
            if self.match_token(&Token::Operator(Operator::Assign)) {
                let values = self.parse_expression_list()?;
                return Ok(vec![Statement::AssignMulti(AssignMulti {
                    names,
                    exprs: values,
                    loc: start_loc,
                })]);
            }
            return Err(ParseError::Message {
                msg: "Expected '=' after variable list".to_string(),
                loc: self.peek_location(),
            });
        }

        // Compound assignment operators: += -= *= /= //= %= ..=
        let compound_map = [
            (Operator::AddAssign, Operator::Add),
            (Operator::SubAssign, Operator::Subtract),
            (Operator::MulAssign, Operator::Multiply),
            (Operator::DivAssign, Operator::Divide),
            (Operator::FloorDivAssign, Operator::FloorDiv),
            (Operator::ModAssign, Operator::Mod),
            (Operator::ConcatAssign, Operator::Concat),
        ];

        for (compound_op, base_op) in &compound_map {
            if self.match_token(&Token::Operator(*compound_op)) {
                let value = self.parse_expression_logic()?;
                let rhs = Expression::BinaryOperation(BinaryOperation {
                    left: Box::new(expr.clone()),
                    operator: *base_op,
                    right: Box::new(value),
                    loc: self.peek_location(),
                });
                let stmt = self.emit_assignment(expr, rhs)?;
                return Ok(vec![stmt]);
            }
        }

        // Simple assignment =
        if self.match_token(&Token::Operator(Operator::Assign)) {
            let value = self.parse_expression_logic()?;
            let stmt = self.emit_assignment(expr, value)?;
            return Ok(vec![stmt]);
        }

        Ok(vec![Statement::Expression(expr)])
    }

    fn emit_assignment(&self, target: Expression, value: Expression) -> Result<Statement, ParseError> {
        let loc = self.peek_location();
        match target {
            Expression::Identifier(name, id_loc) => Ok(Statement::Assign(Assign {
                name,
                expr: Box::new(value),
                loc: id_loc,
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
                loc,
            }),
        }
    }

    fn consume_identifier(&mut self) -> Result<String, ParseError> {
        match self.peek() {
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            _ => Err(ParseError::Message {
                msg: "Expected variable name".to_string(),
                loc: self.peek_location(),
            }),
        }
    }

    fn parse_name_list(&mut self) -> Result<Vec<(String, Location)>, ParseError> {
        let mut names = Vec::new();
        match self.peek() {
            Some(Token::Identifier(_)) => {
                let name = self.advance().unwrap();
                if let Token::Identifier(s) = name {
                    names.push((s.clone(), self.peek_location()));
                }
            }
            _ => {
                return Err(ParseError::Message {
                    msg: "Expected variable name".to_string(),
                    loc: self.peek_location(),
                });
            }
        }
        while self.match_token(&Token::COMMA) {
            match self.peek() {
                Some(Token::Identifier(_)) => {
                    let name = self.advance().unwrap();
                    if let Token::Identifier(s) = name {
                        names.push((s.clone(), self.peek_location()));
                    }
                }
                _ => {
                    return Err(ParseError::Message {
                        msg: "Expected variable name after ','".to_string(),
                        loc: self.peek_location(),
                    });
                }
            }
        }
        Ok(names)
    }

    fn parse_expression_list(&mut self) -> Result<Vec<Expression>, ParseError> {
        let mut exprs = Vec::new();
        exprs.push(self.parse_expression_logic()?);
        while self.match_token(&Token::COMMA) {
            self.skip_newlines();
            exprs.push(self.parse_expression_logic()?);
        }
        Ok(exprs)
    }

    /// `local name [, name]* [= expr [, expr]*]`
    fn parse_local(&mut self) -> Result<Vec<Statement>, ParseError> {
        let start_loc = self.peek_location();
        let names = self.parse_name_list()?;

        let mut values = Vec::new();
        if self.match_token(&Token::Operator(Operator::Assign)) {
            values = self.parse_expression_list()?;
        }

        // 多变量或多值声明 -> LocalList（支持多返回值展开）
        if names.len() > 1 || values.len() > 1 {
            return Ok(vec![Statement::LocalList(LocalList {
                names: names.into_iter().map(|(n, _)| n).collect(),
                values,
                loc: start_loc,
            })]);
        }

        let mut stmts = Vec::new();
        for (i, (name, loc)) in names.into_iter().enumerate() {
            let val = values
                .get(i)
                .cloned()
                .unwrap_or(Expression::Literal(Literal::Value(Value::Null), loc));
            stmts.push(Statement::Local(Local {
                name,
                expression: val,
                loc,
            }));
        }
        Ok(stmts)
    }

    /// `local function name(params) body end`
    fn parse_local_function(&mut self) -> Result<Vec<Statement>, ParseError> {
        self.consume(&Token::Keyword(Keyword::FUNCTION), "Expected 'function' after 'local'")?;
        let decl = self.parse_function_definition()?;
        if decl.name.is_none() {
            return Err(ParseError::Message {
                msg: "Expected function name after 'local function'".to_string(),
                loc: self.peek_location(),
            });
        }
        Ok(vec![Statement::FunctionDeclaration(decl)])
    }

    /// `return [expr [, expr]*]`
    fn parse_return(&mut self) -> Result<Vec<Statement>, ParseError> {
        let start_loc = self.peek_location();
        self.skip_newlines();
        if self.is_at_end() || matches!(self.peek(), Some(Token::Keyword(Keyword::END))) {
            return Ok(vec![Statement::Return(Return {
                values: Vec::new(),
                loc: start_loc,
            })]);
        }
        let values = self.parse_expression_list()?;
        Ok(vec![Statement::Return(Return { values, loc: start_loc })])
    }

    /// `function name ( params ) body end`
    fn parse_function(&mut self) -> Result<Vec<Statement>, ParseError> {
        let decl = self.parse_function_definition()?;
        if decl.name.is_none() {
            return Err(ParseError::Message {
                msg: "Function declaration as statement must have a name".to_string(),
                loc: self.peek_location(),
            });
        }
        Ok(vec![Statement::FunctionDeclaration(decl)])
    }

    fn parse_function_definition(&mut self) -> Result<FunctionDeclaration, ParseError> {
        let start_loc = self.peek_location();
        let (name, name_loc) = if let Some(Token::Identifier(_)) = self.peek() {
            let nloc = self.peek_location();
            let name = self.advance().unwrap();
            if let Token::Identifier(s) = name {
                (Some(s.clone()), Some(nloc))
            } else {
                unreachable!()
            }
        } else {
            (None, None)
        };

        self.consume(&Token::LParen, "Expected '(' after function name")?;

        let mut parameters = Vec::new();
        let mut vararg = false;
        if !self.check(&Token::RParen) {
            loop {
                self.skip_newlines();
                if self.match_token(&Token::Vararg) {
                    vararg = true;
                } else if let Some(Token::Identifier(param)) = self.peek() {
                    parameters.push(param.clone());
                    self.advance();
                } else {
                    return Err(ParseError::Message {
                        msg: "Expected parameter name or '...'".to_string(),
                        loc: self.peek_location(),
                    });
                }

                self.skip_newlines();
                if !self.match_token(&Token::COMMA) {
                    break;
                }
            }
        }
        self.consume(&Token::RParen, "Expected ')' after parameters")?;

        self.skip_newlines();
        let body = self.parse_block(&[Keyword::END])?;
        self.consume(&Token::Keyword(Keyword::END), "Expected 'end' after function body")?;

        Ok(FunctionDeclaration {
            name,
            parameters,
            vararg,
            body,
            loc: name_loc.unwrap_or(start_loc),
        })
    }

    /// `while expr do block end`
    fn parse_while(&mut self) -> Result<Vec<Statement>, ParseError> {
        let start_loc = self.peek_location();
        let condition = self.parse_expression_logic()?;

        self.skip_newlines();
        self.consume(&Token::Keyword(Keyword::DO), "Expected 'do' after while condition")?;
        self.skip_newlines();
        let body = self.parse_block(&[Keyword::END])?;
        self.consume(&Token::Keyword(Keyword::END), "Expected 'end' after while block")?;

        Ok(vec![Statement::Loop(Loop {
            test: condition,
            body,
            loc: start_loc,
        })])
    }

    /// `repeat block until expr`
    fn parse_repeat(&mut self) -> Result<Vec<Statement>, ParseError> {
        let start_loc = self.peek_location();

        self.skip_newlines();
        let body = self.parse_block(&[Keyword::UNTIL])?;
        self.consume(&Token::Keyword(Keyword::UNTIL), "Expected 'until' after repeat block")?;
        let condition = self.parse_expression_logic()?;

        Ok(vec![Statement::Repeat(Repeat {
            body,
            test: condition,
            loc: start_loc,
        })])
    }

    /// `for name = expr, expr [, expr] do block end`
    /// `for namelist in explist do block end`
    fn parse_for(&mut self) -> Result<Vec<Statement>, ParseError> {
        let start_loc = self.peek_location();

        // Peek ahead to distinguish numeric for (name = ...) from generic for (name in ...)
        if let Some(Token::Identifier(_)) = self.peek() {
            // 跳过 `name [, name]*`，看是否以 `in` 结尾
            let mut i = self.current + 1;
            loop {
                while i < self.tokens.len() && self.tokens[i].0 == Token::NewLine {
                    i += 1;
                }
                match self.tokens.get(i).map(|(t, _)| t) {
                    Some(Token::COMMA) | Some(Token::Identifier(_)) => {
                        i += 1;
                        continue;
                    }
                    Some(Token::Keyword(Keyword::IN)) => {
                        return self.parse_for_in(start_loc);
                    }
                    _ => break,
                }
            }
        }

        self.parse_for_numeric(start_loc)
    }

    /// `for name = start, end [, step] do block end`
    /// Desugars to a while loop with local variable + increment.
    fn parse_for_numeric(&mut self, start_loc: Location) -> Result<Vec<Statement>, ParseError> {
        let var_loc = self.peek_location();
        let var_name = match self.advance() {
            Some(Token::Identifier(name)) => name.clone(),
            _ => {
                return Err(ParseError::Message {
                    msg: "Expected loop variable name after 'for'".to_string(),
                    loc: self.peek_location(),
                });
            }
        };

        self.consume(&Token::Operator(Operator::Assign), "Expected '=' after loop variable")?;
        let start = self.parse_expression_logic()?;

        self.consume(&Token::COMMA, "Expected ',' after start value in for loop")?;
        self.skip_newlines();
        let end = self.parse_expression_logic()?;

        let step = if self.match_token(&Token::COMMA) {
            self.skip_newlines();
            self.parse_expression_logic()?
        } else {
            Expression::Literal(Literal::Value(Value::Int(1)), start_loc)
        };

        self.skip_newlines();
        self.consume(&Token::Keyword(Keyword::DO), "Expected 'do' after for loop header")?;
        self.skip_newlines();
        let body = self.parse_block(&[Keyword::END])?;
        self.consume(&Token::Keyword(Keyword::END), "Expected 'end' after for loop body")?;

        // Desugar to:
        //   local var = start
        //   local step = <step>
        //   while (step >= 0 and var <= end) or (step < 0 and var >= end) do
        //     body
        //     var = var + step
        //   end
        let mut loop_body = body;
        loop_body.push(Statement::Assign(Assign {
            name: var_name.clone(),
            expr: Box::new(Expression::BinaryOperation(BinaryOperation {
                left: Box::new(Expression::Identifier(var_name.clone(), var_loc)),
                operator: Operator::Add,
                right: Box::new(Expression::Identifier("@step".to_string(), start_loc)),
                loc: start_loc,
            })),
            loc: start_loc,
        }));

        let var_expr = Expression::Identifier(var_name.clone(), var_loc);
        let step_expr = Expression::Identifier("@step".to_string(), start_loc);
        let end_expr = end;

        // (step >= 0)
        let step_ge_zero = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(step_expr.clone()),
            operator: Operator::GtE,
            right: Box::new(Expression::Literal(Literal::Value(Value::Int(0)), start_loc)),
            loc: start_loc,
        });
        // (step >= 0 and var <= end)
        let asc_cond = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(step_ge_zero),
            operator: Operator::And,
            right: Box::new(Expression::BinaryOperation(BinaryOperation {
                left: Box::new(var_expr.clone()),
                operator: Operator::LtE,
                right: Box::new(end_expr.clone()),
                loc: start_loc,
            })),
            loc: start_loc,
        });
        // (step < 0)
        let step_lt_zero = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(step_expr.clone()),
            operator: Operator::Lt,
            right: Box::new(Expression::Literal(Literal::Value(Value::Int(0)), start_loc)),
            loc: start_loc,
        });
        // (step < 0 and var >= end)
        let desc_cond = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(step_lt_zero),
            operator: Operator::And,
            right: Box::new(Expression::BinaryOperation(BinaryOperation {
                left: Box::new(var_expr.clone()),
                operator: Operator::GtE,
                right: Box::new(end_expr.clone()),
                loc: start_loc,
            })),
            loc: start_loc,
        });
        // (asc) or (desc)
        let test = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(asc_cond),
            operator: Operator::Or,
            right: Box::new(desc_cond),
            loc: start_loc,
        });

        Ok(vec![
            Statement::Local(Local {
                name: var_name.clone(),
                expression: start,
                loc: var_loc,
            }),
            Statement::Local(Local {
                name: "@step".to_string(),
                expression: step,
                loc: start_loc,
            }),
            Statement::Loop(Loop {
                test,
                body: loop_body,
                loc: start_loc,
            }),
        ])
    }

    /// `for namelist in explist do block end`
    fn parse_for_in(&mut self, start_loc: Location) -> Result<Vec<Statement>, ParseError> {
        let mut vars = Vec::new();
        vars.push(self.consume_identifier()?);
        while self.match_token(&Token::COMMA) {
            self.skip_newlines();
            vars.push(self.consume_identifier()?);
        }

        self.consume(&Token::Keyword(Keyword::IN), "Expected 'in' after variable name")?;
        let iterable = self.parse_expression_logic()?;

        self.skip_newlines();
        self.consume(&Token::Keyword(Keyword::DO), "Expected 'do' after for-in iterable")?;
        self.skip_newlines();
        let body = self.parse_block(&[Keyword::END])?;
        self.consume(&Token::Keyword(Keyword::END), "Expected 'end' after for-in block")?;

        Ok(vec![Statement::ForIn(ForInLoop {
            vars,
            iterable,
            body,
            loc: start_loc,
        })])
    }

    /// `do block end`
    fn parse_do(&mut self) -> Result<Vec<Statement>, ParseError> {
        let start_loc = self.peek_location();
        self.skip_newlines();
        let body = self.parse_block(&[Keyword::END])?;
        self.consume(&Token::Keyword(Keyword::END), "Expected 'end' after do block")?;
        Ok(vec![Statement::Expression(Expression::Block(body, start_loc))])
    }

    /// `if expr then block {elseif expr then block} [else block] end`
    /// Called after `if` keyword has been consumed.
    fn parse_if_tail(&mut self) -> Result<Expression, ParseError> {
        let start_loc = self.peek_location();
        let condition = self.parse_expression_logic()?;

        self.skip_newlines();
        self.consume(&Token::Keyword(Keyword::THEN), "Expected 'then' after if condition")?;
        self.skip_newlines();
        let then_body = self.parse_block(&[Keyword::ELSEIF, Keyword::ELSE, Keyword::END])?;

        let else_body = self.parse_else_chain()?;

        self.skip_newlines();
        self.consume(
            &Token::Keyword(Keyword::END),
            "Expected 'end' after if/elseif/else block",
        )?;

        Ok(Expression::If(If {
            test: Box::new(condition),
            body: then_body,
            else_body,
            loc: start_loc,
        }))
    }

    fn parse_else_chain(&mut self) -> Result<Vec<Statement>, ParseError> {
        self.skip_newlines();
        if self.match_keyword(Keyword::ELSEIF) {
            let elseif_expr = self.parse_if_tail()?;
            Ok(vec![Statement::Expression(elseif_expr)])
        } else if self.match_keyword(Keyword::ELSE) {
            self.skip_newlines();
            self.parse_block(&[Keyword::END])
        } else {
            Ok(Vec::new())
        }
    }

    /// `try body catch [name] body [finally body] end` (Luau style)
    fn parse_try_catch(&mut self) -> Result<Vec<Statement>, ParseError> {
        let start_loc = self.peek_location();

        self.skip_newlines();
        let try_body = self.parse_block(&[Keyword::CATCH, Keyword::END, Keyword::FINALLY])?;

        self.skip_newlines();
        self.consume(&Token::Keyword(Keyword::CATCH), "Expected 'catch' after try block")?;

        let error_name = if let Some(Token::Identifier(name)) = self.peek() {
            let n = name.clone();
            self.advance();
            Some(n)
        } else {
            None
        };

        self.skip_newlines();
        let catch_body = self.parse_block(&[Keyword::FINALLY, Keyword::END])?;

        let finally_body = if self.match_keyword(Keyword::FINALLY) {
            self.skip_newlines();
            let body = self.parse_block(&[Keyword::END])?;
            Some(body)
        } else {
            None
        };

        self.skip_newlines();
        self.consume(&Token::Keyword(Keyword::END), "Expected 'end' after try-catch block")?;

        Ok(vec![Statement::TryCatch(TryCatch {
            try_body,
            error_name,
            catch_body,
            finally_body,
            loc: start_loc,
        })])
    }

    // --- Expression Precedence Chain ---

    fn parse_expression_logic(&mut self) -> Result<Expression, ParseError> {
        self.skip_newlines();
        self.parse_logical_or()
    }

    /// `or` (lowest precedence)
    fn parse_logical_or(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_logical_and()?;
        loop {
            let op_loc = self.peek_location();
            let matched = self.match_keyword(Keyword::OR) || self.match_token(&Token::Operator(Operator::Or));
            if !matched {
                break;
            }
            let right = self.parse_logical_and()?;
            left = Expression::BinaryOperation(BinaryOperation {
                left: Box::new(left),
                operator: Operator::Or,
                right: Box::new(right),
                loc: op_loc,
            });
        }
        Ok(left)
    }

    /// `and`
    fn parse_logical_and(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_equality()?;
        loop {
            let op_loc = self.peek_location();
            let matched = self.match_keyword(Keyword::AND) || self.match_token(&Token::Operator(Operator::And));
            if !matched {
                break;
            }
            let right = self.parse_equality()?;
            left = Expression::BinaryOperation(BinaryOperation {
                left: Box::new(left),
                operator: Operator::And,
                right: Box::new(right),
                loc: op_loc,
            });
        }
        Ok(left)
    }

    /// `==` `~=`
    fn parse_equality(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_comparison()?;
        loop {
            let op_loc = self.peek_location();
            let op = match self.peek() {
                Some(Token::Operator(Operator::Equals)) => Some(Operator::Equals),
                Some(Token::Operator(Operator::NotEquals)) => Some(Operator::NotEquals),
                _ => None,
            };
            if let Some(op) = op {
                self.advance();
                let right = self.parse_comparison()?;
                left = Expression::BinaryOperation(BinaryOperation {
                    left: Box::new(left),
                    operator: op,
                    right: Box::new(right),
                    loc: op_loc,
                });
            } else {
                break;
            }
        }
        Ok(left)
    }

    /// `<` `<=` `>` `>=`
    fn parse_comparison(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_concat()?;
        loop {
            let op_loc = self.peek_location();
            let op = match self.peek() {
                Some(Token::Operator(op))
                    if matches!(op, Operator::Gt | Operator::GtE | Operator::Lt | Operator::LtE) =>
                {
                    Some(*op)
                }
                _ => None,
            };
            if let Some(op) = op {
                self.advance();
                let right = self.parse_concat()?;
                left = Expression::BinaryOperation(BinaryOperation {
                    left: Box::new(left),
                    operator: op,
                    right: Box::new(right),
                    loc: op_loc,
                });
            } else {
                break;
            }
        }
        Ok(left)
    }

    /// `..` (concat operator)
    fn parse_concat(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_term()?;
        loop {
            let op_loc = self.peek_location();
            if !self.match_token(&Token::Operator(Operator::Concat)) {
                break;
            }
            let right = self.parse_term()?;
            left = Expression::BinaryOperation(BinaryOperation {
                left: Box::new(left),
                operator: Operator::Concat,
                right: Box::new(right),
                loc: op_loc,
            });
        }
        Ok(left)
    }

    /// `+` `-`
    fn parse_term(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_factor()?;
        loop {
            let op_loc = self.peek_location();
            let op = match self.peek() {
                Some(Token::Operator(Operator::Add)) => Some(Operator::Add),
                Some(Token::Operator(Operator::Subtract)) => Some(Operator::Subtract),
                _ => None,
            };
            if let Some(op) = op {
                self.advance();
                let right = self.parse_factor()?;
                left = Expression::BinaryOperation(BinaryOperation {
                    left: Box::new(left),
                    operator: op,
                    right: Box::new(right),
                    loc: op_loc,
                });
            } else {
                break;
            }
        }
        Ok(left)
    }

    /// `*` `/` `//` `%`
    fn parse_factor(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_unary()?;
        loop {
            let op_loc = self.peek_location();
            let op = match self.peek() {
                Some(Token::Operator(Operator::Multiply)) => Some(Operator::Multiply),
                Some(Token::Operator(Operator::Divide)) => Some(Operator::Divide),
                Some(Token::Operator(Operator::FloorDiv)) => Some(Operator::FloorDiv),
                Some(Token::Operator(Operator::Mod)) => Some(Operator::Mod),
                _ => None,
            };
            if let Some(op) = op {
                self.advance();
                let right = self.parse_unary()?;
                left = Expression::BinaryOperation(BinaryOperation {
                    left: Box::new(left),
                    operator: op,
                    right: Box::new(right),
                    loc: op_loc,
                });
            } else {
                break;
            }
        }
        Ok(left)
    }

    /// `not` `-` `#` (unary operators)
    fn parse_unary(&mut self) -> Result<Expression, ParseError> {
        let start_loc = self.peek_location();

        let matched_not = self.match_keyword(Keyword::NOT) || self.match_token(&Token::Operator(Operator::Not));
        if matched_not {
            let right = self.parse_unary()?;
            return Ok(Expression::Unary(Unary {
                operator: Operator::Not,
                expr: Box::new(right),
                loc: start_loc,
            }));
        }

        if self.match_token(&Token::Operator(Operator::Subtract)) {
            let right = self.parse_unary()?;
            return Ok(Expression::BinaryOperation(BinaryOperation {
                left: Box::new(Expression::Literal(Literal::Value(Value::Int(0)), start_loc)),
                operator: Operator::Subtract,
                right: Box::new(right),
                loc: start_loc,
            }));
        }

        if self.match_token(&Token::Operator(Operator::Len)) {
            let right = self.parse_unary()?;
            return Ok(Expression::Unary(Unary {
                operator: Operator::Len,
                expr: Box::new(right),
                loc: start_loc,
            }));
        }

        self.parse_power()
    }

    /// `^` (幂运算，右结合，优先级高于一元运算符)
    fn parse_power(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_postfix()?;
        if self.match_token(&Token::Operator(Operator::Pow)) {
            let right = self.parse_power()?;
            left = Expression::BinaryOperation(BinaryOperation {
                left: Box::new(left),
                operator: Operator::Pow,
                right: Box::new(right),
                loc: self.peek_location(),
            });
        }
        Ok(left)
    }

    /// Calls, field access, index access, method calls
    fn parse_postfix(&mut self) -> Result<Expression, ParseError> {
        let start_loc = self.peek_location();
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(&Token::LParen) {
                let args = self.parse_argument_list()?;
                expr = Expression::FunctionCall(FunctionCall {
                    callee: Box::new(expr),
                    arguments: args,
                    loc: start_loc,
                });
            } else if self.match_token(&Token::Dot) {
                if let Some(Token::Identifier(field)) = self.peek() {
                    let field_name = field.clone();
                    let field_loc = self.peek_location();
                    self.advance();
                    expr = Expression::GetField {
                        object: Box::new(expr),
                        field: field_name,
                        loc: field_loc,
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
                self.consume(&Token::RSquare, "Expected ']' after index expression")?;
                expr = Expression::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                    loc: self.peek_location(),
                };
            } else if self.match_token(&Token::Colon) {
                if let Some(Token::Identifier(method)) = self.peek() {
                    let method_name = method.clone();
                    self.advance();
                    self.skip_newlines();
                    self.consume(&Token::LParen, "Expected '(' after method name")?;
                    let args = self.parse_argument_list()?;
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

    fn parse_argument_list(&mut self) -> Result<Vec<Expression>, ParseError> {
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
        Ok(args)
    }

    // --- Primary Expressions ---

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
            Token::LBig => self.parse_table_literal(),
            Token::LSquare => self.parse_array_literal(),
            Token::Keyword(Keyword::FUNCTION) => {
                let decl = self.parse_function_definition()?;
                Ok(Expression::Function(decl))
            }
            Token::Keyword(Keyword::IF) => self.parse_if_tail(),
            Token::Keyword(Keyword::NIL) => Ok(Expression::Literal(Literal::Value(Value::Null), start_loc)),
            Token::Keyword(Keyword::TRUE) => Ok(Expression::Literal(Literal::Value(Value::Bool(true)), start_loc)),
            Token::Keyword(Keyword::FALSE) => Ok(Expression::Literal(Literal::Value(Value::Bool(false)), start_loc)),
            Token::LParen => {
                self.skip_newlines();
                let expr = self.parse_expression_logic()?;
                self.skip_newlines();
                self.consume(&Token::RParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            Token::Vararg => Ok(Expression::Vararg(start_loc)),
            _ => Err(ParseError::UnexpectedToken { token, loc: start_loc }),
        }
    }

    /// `{ [key = value,] [, value]... }` — Luau table literal
    fn parse_table_literal(&mut self) -> Result<Expression, ParseError> {
        let start_loc = self.peek_location();
        let mut fields = Vec::new();
        let mut array_elems = Vec::new();

        self.skip_newlines();
        if !self.check(&Token::RBig) {
            loop {
                self.skip_newlines();
                if self.check(&Token::RBig) {
                    break;
                }
                // Check for key-value pair: identifier = expr
                if let Some(Token::Identifier(key_name)) = self.peek() {
                    // Look ahead for '=' (skip newlines)
                    let mut lookahead = self.current + 1;
                    while lookahead < self.tokens.len() && self.tokens[lookahead].0 == Token::NewLine {
                        lookahead += 1;
                    }
                    if lookahead < self.tokens.len()
                        && matches!(self.tokens[lookahead].0, Token::Operator(Operator::Assign))
                    {
                        // It's a key-value pair
                        let key = key_name.clone();
                        self.advance(); // consume identifier
                        self.skip_newlines();
                        self.consume(&Token::Operator(Operator::Assign), "Expected '=' after key")?;
                        let val = self.parse_expression_logic()?;
                        fields.push((key, val));
                    } else {
                        // It's a value expression
                        let val = self.parse_expression_logic()?;
                        array_elems.push(val);
                    }
                } else if self.check(&Token::LSquare) {
                    // Computed key: [expr] = expr (skip for now, treat as expression)
                    let val = self.parse_expression_logic()?;
                    array_elems.push(val);
                } else {
                    // Value expression (array-style element)
                    let val = self.parse_expression_logic()?;
                    array_elems.push(val);
                }

                self.skip_newlines();
                if !self.match_token(&Token::COMMA) {
                    break;
                }
            }
        }

        self.skip_newlines();
        self.consume(&Token::RBig, "Expected '}' after table literal")?;

        // If there are mixed fields and array elements, combine them.
        // For simplicity, array elements are appended to the object literal
        // with numeric string keys "1", "2", etc.
        if !fields.is_empty() || array_elems.is_empty() {
            let mut result = fields;
            for (i, elem) in array_elems.into_iter().enumerate() {
                result.push(((i + 1).to_string(), elem));
            }
            Ok(Expression::ObjectLiteral(result, start_loc))
        } else {
            Ok(Expression::ArrayLiteral(array_elems, start_loc))
        }
    }

    /// `[ expr [, expr]* ]`
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
}

/// Parse a token stream into an AST (list of statements).
///
/// # Arguments
/// * `tokens` - Token stream with location information
///
/// # Returns
/// * `Ok(Ast)` on success
/// * `Err(ParseError)` on failure
pub fn parse(tokens: Vec<(Token, Location)>) -> Result<Ast, ParseError> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}
