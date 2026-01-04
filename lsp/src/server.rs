use std::sync::Arc;

use chen_lang::expression::{Ast, Expression, Statement};
use chen_lang::tokenizer::Location as AstLocation;
use ropey::Rope;
use tower_lsp_server::jsonrpc::Result as LspResult;
use tower_lsp_server::ls_types::*;
use tower_lsp_server::{Client, LanguageServer};
use tracing::info;

use super::document::Documents;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Symbol {
    name: String,
    kind: SymbolKind,
    range: Range,
    definition_range: Range,
}

#[derive(Clone)]
pub struct ChenLangLsp {
    client: Client,
    docs: Arc<Documents>,
}

impl ChenLangLsp {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            docs: Arc::new(Documents::new()),
        }
    }

    fn get_word_at_position(text: &Rope, position: Position) -> Option<String> {
        let char_idx = Self::lsp_position_to_char_idx(text, position)?;
        let line_idx = position.line as usize;
        let line = text.line(line_idx);
        let line_start = text.line_to_char(line_idx);
        let col_char = char_idx.saturating_sub(line_start);

        let line_str = line.to_string();
        let chars: Vec<char> = line_str.chars().collect();
        if col_char >= chars.len() {
            return None;
        }

        let mut start = col_char;
        let mut end = col_char;

        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }

        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }

        if start < end {
            Some(chars[start..end].iter().collect())
        } else {
            None
        }
    }

    fn lsp_position_to_char_idx(text: &Rope, position: Position) -> Option<usize> {
        let line_idx = position.line as usize;
        if line_idx >= text.len_lines() {
            return None;
        }
        let line = text.line(line_idx);
        let line_start = text.line_to_char(line_idx);
        let target_utf16 = position.character as usize;
        let mut utf16_count = 0;
        let mut char_count = 0;

        for c in line.chars() {
            if utf16_count >= target_utf16 {
                break;
            }
            utf16_count += c.len_utf16();
            char_count += 1;
            if utf16_count >= target_utf16 {
                break;
            }
        }

        Some(line_start + char_count)
    }

    fn loc_to_range(text: &Rope, loc: AstLocation, len_chars: usize) -> Option<Range> {
        if loc.line == 0 {
            return None;
        }
        let line_idx = loc.line.saturating_sub(1) as usize;
        if line_idx >= text.len_lines() {
            return None;
        }
        let line = text.line(line_idx);
        let col_start = loc.col.saturating_sub(1) as usize;

        let mut start_utf16 = 0usize;
        let mut end_utf16 = 0usize;
        let mut char_idx = 0usize;

        for c in line.chars() {
            if char_idx < col_start {
                start_utf16 += c.len_utf16();
            }
            if char_idx < col_start + len_chars {
                end_utf16 += c.len_utf16();
            }
            char_idx += 1;
        }

        let start = Position::new(line_idx as u32, start_utf16 as u32);
        let end = Position::new(line_idx as u32, end_utf16 as u32);
        Some(Range { start, end })
    }

    fn collect_refs(ast: &Ast, out: &mut Vec<IdentRef>) {
        for stmt in ast {
            Self::walk_stmt(stmt, out);
        }
    }

    fn walk_stmt(stmt: &Statement, out: &mut Vec<IdentRef>) {
        match stmt {
            Statement::Expression(expr) => Self::walk_expr(expr, out),
            Statement::Return(ret) => Self::walk_expr(&ret.expression, out),
            Statement::Local(local) => {
                out.push(IdentRef::new(local.name.clone(), local.loc));
                Self::walk_expr(&local.expression, out);
            }
            Statement::Assign(assign) => {
                out.push(IdentRef::new(assign.name.clone(), assign.loc));
                Self::walk_expr(&assign.expr, out);
            }
            Statement::FunctionDeclaration(fd) => {
                if let Some(name) = &fd.name {
                    out.push(IdentRef::new(name.clone(), fd.loc));
                }
                for stmt in &fd.body {
                    Self::walk_stmt(stmt, out);
                }
            }
            Statement::Loop(loop_) => {
                Self::walk_expr(&loop_.test, out);
                for stmt in &loop_.body {
                    Self::walk_stmt(stmt, out);
                }
            }
            Statement::SetField { object, value, .. } => {
                Self::walk_expr(object, out);
                Self::walk_expr(value, out);
            }
            Statement::SetIndex {
                object, index, value, ..
            } => {
                Self::walk_expr(object, out);
                Self::walk_expr(index, out);
                Self::walk_expr(value, out);
            }
            Statement::TryCatch(tc) => {
                for stmt in &tc.try_body {
                    Self::walk_stmt(stmt, out);
                }
                for stmt in &tc.catch_body {
                    Self::walk_stmt(stmt, out);
                }
                if let Some(finally_body) = &tc.finally_body {
                    for stmt in finally_body {
                        Self::walk_stmt(stmt, out);
                    }
                }
            }
            Statement::Throw { value, .. } => Self::walk_expr(value, out),
            Statement::Break(_) | Statement::Continue(_) => {}
        }
    }

    fn walk_expr(expr: &Expression, out: &mut Vec<IdentRef>) {
        match expr {
            Expression::Identifier(name, loc) => out.push(IdentRef::new(name.clone(), *loc)),
            Expression::FunctionCall(call) => {
                Self::walk_expr(&call.callee, out);
                for arg in &call.arguments {
                    Self::walk_expr(arg, out);
                }
            }
            Expression::MethodCall(call) => {
                Self::walk_expr(&call.object, out);
                for arg in &call.arguments {
                    Self::walk_expr(arg, out);
                }
            }
            Expression::BinaryOperation(bin) => {
                Self::walk_expr(&bin.left, out);
                Self::walk_expr(&bin.right, out);
            }
            Expression::Unary(unary) => Self::walk_expr(&unary.expr, out),
            Expression::Block(stmts, _) => {
                for stmt in stmts {
                    Self::walk_stmt(stmt, out);
                }
            }
            Expression::If(if_expr) => {
                Self::walk_expr(&if_expr.test, out);
                for stmt in &if_expr.body {
                    Self::walk_stmt(stmt, out);
                }
                for stmt in &if_expr.else_body {
                    Self::walk_stmt(stmt, out);
                }
            }
            Expression::ObjectLiteral(fields, _) => {
                for (_, expr) in fields {
                    Self::walk_expr(expr, out);
                }
            }
            Expression::ArrayLiteral(elements, _) => {
                for expr in elements {
                    Self::walk_expr(expr, out);
                }
            }
            Expression::GetField { object, .. } => Self::walk_expr(object, out),
            Expression::Index { object, index, .. } => {
                Self::walk_expr(object, out);
                Self::walk_expr(index, out);
            }
            Expression::Function(fd) => {
                if let Some(name) = &fd.name {
                    out.push(IdentRef::new(name.clone(), fd.loc));
                }
                for stmt in &fd.body {
                    Self::walk_stmt(stmt, out);
                }
            }
            Expression::Import { .. } | Expression::Literal(_, _) => {}
        }
    }

    fn collect_definitions(ast: &Ast, out: &mut Vec<IdentRef>) {
        for stmt in ast {
            Self::collect_defs_in_stmt(stmt, out);
        }
    }

    fn collect_defs_in_stmt(stmt: &Statement, out: &mut Vec<IdentRef>) {
        match stmt {
            Statement::Local(local) => out.push(IdentRef::new(local.name.clone(), local.loc)),
            Statement::Assign(assign) => out.push(IdentRef::new(assign.name.clone(), assign.loc)),
            Statement::FunctionDeclaration(fd) => {
                if let Some(name) = &fd.name {
                    out.push(IdentRef::new(name.clone(), fd.loc));
                }
                for stmt in &fd.body {
                    Self::collect_defs_in_stmt(stmt, out);
                }
            }
            Statement::Expression(expr) => Self::collect_defs_in_expr(expr, out),
            Statement::Loop(loop_) => {
                Self::collect_defs_in_expr(&loop_.test, out);
                for stmt in &loop_.body {
                    Self::collect_defs_in_stmt(stmt, out);
                }
            }
            Statement::Return(ret) => Self::collect_defs_in_expr(&ret.expression, out),
            Statement::SetField { value, object, .. } => {
                Self::collect_defs_in_expr(object, out);
                Self::collect_defs_in_expr(value, out);
            }
            Statement::SetIndex {
                value, object, index, ..
            } => {
                Self::collect_defs_in_expr(object, out);
                Self::collect_defs_in_expr(index, out);
                Self::collect_defs_in_expr(value, out);
            }
            Statement::TryCatch(tc) => {
                for stmt in &tc.try_body {
                    Self::collect_defs_in_stmt(stmt, out);
                }
                for stmt in &tc.catch_body {
                    Self::collect_defs_in_stmt(stmt, out);
                }
                if let Some(finally_body) = &tc.finally_body {
                    for stmt in finally_body {
                        Self::collect_defs_in_stmt(stmt, out);
                    }
                }
            }
            Statement::Throw { value, .. } => Self::collect_defs_in_expr(value, out),
            Statement::Break(_) | Statement::Continue(_) => {}
        }
    }

    fn collect_defs_in_expr(expr: &Expression, out: &mut Vec<IdentRef>) {
        match expr {
            Expression::Function(fd) => {
                if let Some(name) = &fd.name {
                    out.push(IdentRef::new(name.clone(), fd.loc));
                }
                for stmt in &fd.body {
                    Self::collect_defs_in_stmt(stmt, out);
                }
            }
            Expression::FunctionCall(call) => {
                Self::collect_defs_in_expr(&call.callee, out);
                for arg in &call.arguments {
                    Self::collect_defs_in_expr(arg, out);
                }
            }
            Expression::MethodCall(call) => {
                Self::collect_defs_in_expr(&call.object, out);
                for arg in &call.arguments {
                    Self::collect_defs_in_expr(arg, out);
                }
            }
            Expression::BinaryOperation(bin) => {
                Self::collect_defs_in_expr(&bin.left, out);
                Self::collect_defs_in_expr(&bin.right, out);
            }
            Expression::Unary(unary) => Self::collect_defs_in_expr(&unary.expr, out),
            Expression::Block(stmts, _) => {
                for stmt in stmts {
                    Self::collect_defs_in_stmt(stmt, out);
                }
            }
            Expression::If(if_expr) => {
                Self::collect_defs_in_expr(&if_expr.test, out);
                for stmt in &if_expr.body {
                    Self::collect_defs_in_stmt(stmt, out);
                }
                for stmt in &if_expr.else_body {
                    Self::collect_defs_in_stmt(stmt, out);
                }
            }
            Expression::ObjectLiteral(fields, _) => {
                for (_, expr) in fields {
                    Self::collect_defs_in_expr(expr, out);
                }
            }
            Expression::ArrayLiteral(elements, _) => {
                for expr in elements {
                    Self::collect_defs_in_expr(expr, out);
                }
            }
            Expression::GetField { object, .. } => Self::collect_defs_in_expr(object, out),
            Expression::Index { object, index, .. } => {
                Self::collect_defs_in_expr(object, out);
                Self::collect_defs_in_expr(index, out);
            }
            Expression::Identifier(_, _) | Expression::Import { .. } | Expression::Literal(_, _) => {}
        }
    }
}

#[derive(Debug, Clone)]
struct IdentRef {
    name: String,
    loc: AstLocation,
    len_chars: usize,
}

impl IdentRef {
    fn new(name: String, loc: AstLocation) -> Self {
        let len_chars = name.chars().count();
        Self { name, loc, len_chars }
    }
}

impl LanguageServer for ChenLangLsp {
    async fn initialize(&self, _params: InitializeParams) -> LspResult<InitializeResult> {
        info!("Initializing Chen Lang LSP");

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "Chen Lang Language Server".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::INCREMENTAL)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        info!("Chen Lang LSP initialized");
    }

    async fn shutdown(&self) -> LspResult<()> {
        info!("Shutting down Chen Lang LSP");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;

        info!("Opened document: {:?}", uri);

        self.docs.insert(uri.clone(), text.clone());

        // Publish diagnostics on open
        let diagnostics = Self::analyze(&text);
        self.docs.update_diagnostics(&uri, diagnostics.clone());
        let _ = self.client.publish_diagnostics(uri, diagnostics, None).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;

        info!("Changed document: {:?}", uri);

        self.docs.update(uri.clone(), params.content_changes);

        // Publish diagnostics after change
        if let Some(doc) = self.docs.get(&uri) {
            let diagnostics = Self::analyze(&doc.text.to_string());
            self.docs.update_diagnostics(&uri, diagnostics.clone());
            let _ = self.client.publish_diagnostics(uri, diagnostics, None).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;

        info!("Closed document: {:?}", uri);

        self.docs.remove(&uri);
        let _ = self.client.publish_diagnostics(uri, Vec::new(), None);
    }

    async fn completion(&self, params: CompletionParams) -> LspResult<Option<CompletionResponse>> {
        let _uri = params.text_document_position.text_document.uri;

        let keywords = vec![
            "let", "def", "if", "else", "for", "return", "break", "continue", "try", "catch", "finally", "throw",
            "true", "false", "null", "import",
        ];

        let mut items = Vec::new();

        for keyword in keywords {
            items.push(CompletionItem {
                label: keyword.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("keyword".to_string()),
                ..Default::default()
            });
        }

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn hover(&self, params: HoverParams) -> LspResult<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        if let Some(doc) = self.docs.get(&uri) {
            if let Some(word) = Self::get_word_at_position(&doc.text, pos) {
                let hover_text = match word.as_str() {
                    "let" => "let variable = value\n\nDeclare a new variable",
                    "def" => "def name(params) { ... }\n\nDefine a function",
                    "if" => "if condition { ... } else { ... }\n\nConditional expression",
                    "for" => "for condition { ... }\n\nLoop while condition is true",
                    "return" => "return value\n\nReturn a value from a function",
                    "try" => "try { ... } catch error { ... }\n\nException handling",
                    "import" => "import \"module\"\n\nImport a module",
                    _ => &word,
                };

                Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!("```chen\n{}\n```", hover_text),
                    }),
                    range: None,
                }))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    async fn document_symbol(&self, params: DocumentSymbolParams) -> LspResult<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;

        if let Some(doc) = self.docs.get(&uri) {
            let text = doc.text.to_string();

            let symbols = Self::extract_symbols(&text);

            Ok(Some(DocumentSymbolResponse::Nested(symbols)))
        } else {
            Ok(None)
        }
    }

    async fn goto_definition(&self, params: GotoDefinitionParams) -> LspResult<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        if let Some(doc) = self.docs.get(&uri) {
            if let Some(word) = Self::get_word_at_position(&doc.text, pos) {
                if let Ok(ast) = chen_lang::parser::parse_from_source(&doc.text.to_string()) {
                    let mut defs = Vec::new();
                    Self::collect_definitions(&ast, &mut defs);
                    for def in defs {
                        if def.name == word {
                            if let Some(range) = Self::loc_to_range(&doc.text, def.loc, def.len_chars) {
                                return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                                    uri: uri.clone(),
                                    range,
                                })));
                            }
                        }
                    }
                } else {
                    let symbols = Self::collect_symbols(&doc.text.to_string());
                    for symbol in symbols {
                        if symbol.name == word {
                            return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                                uri: uri.clone(),
                                range: symbol.definition_range,
                            })));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    async fn references(&self, params: ReferenceParams) -> LspResult<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;

        if let Some(doc) = self.docs.get(&uri) {
            if let Some(word) = Self::get_word_at_position(&doc.text, pos) {
                if let Ok(ast) = chen_lang::parser::parse_from_source(&doc.text.to_string()) {
                    let mut refs = Vec::new();
                    Self::collect_refs(&ast, &mut refs);
                    let mut locations = Vec::new();
                    for r in refs {
                        if r.name == word {
                            if let Some(range) = Self::loc_to_range(&doc.text, r.loc, r.len_chars) {
                                locations.push(Location {
                                    uri: uri.clone(),
                                    range,
                                });
                            }
                        }
                    }
                    if !locations.is_empty() {
                        return Ok(Some(locations));
                    }
                } else {
                    let locations = Self::find_references(&doc.text.to_string(), &word, &uri);
                    if !locations.is_empty() {
                        return Ok(Some(locations));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn rename(&self, params: RenameParams) -> LspResult<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;
        let new_name = params.new_name;

        if let Some(doc) = self.docs.get(&uri) {
            if let Some(word) = Self::get_word_at_position(&doc.text, pos) {
                let locations = if let Ok(ast) = chen_lang::parser::parse_from_source(&doc.text.to_string()) {
                    let mut refs = Vec::new();
                    Self::collect_refs(&ast, &mut refs);
                    let mut locations = Vec::new();
                    for r in refs {
                        if r.name == word {
                            if let Some(range) = Self::loc_to_range(&doc.text, r.loc, r.len_chars) {
                                locations.push(Location {
                                    uri: uri.clone(),
                                    range,
                                });
                            }
                        }
                    }
                    locations
                } else {
                    Self::find_references(&doc.text.to_string(), &word, &uri)
                };

                if !locations.is_empty() {
                    let edits: Vec<TextEdit> = locations
                        .iter()
                        .map(|loc| TextEdit {
                            range: loc.range,
                            new_text: new_name.clone(),
                        })
                        .collect();

                    let mut changes = std::collections::HashMap::new();
                    changes.insert(uri, edits);

                    return Ok(Some(WorkspaceEdit {
                        changes: Some(changes),
                        ..Default::default()
                    }));
                }
            }
        }

        Ok(None)
    }
}

impl ChenLangLsp {
    fn extract_symbols(source: &str) -> Vec<DocumentSymbol> {
        let mut symbols = Vec::new();

        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            if let Some(name) = trimmed.strip_prefix("def ") {
                if let Some(end) = name.find('(') {
                    let func_name = name[..end].trim();
                    symbols.push(DocumentSymbol {
                        name: func_name.to_string(),
                        detail: Some("function".to_string()),
                        kind: SymbolKind::FUNCTION,
                        range: Range {
                            start: Position::new(line_num as u32, 0),
                            end: Position::new(line_num as u32, line.len() as u32),
                        },
                        selection_range: Range {
                            start: Position::new(line_num as u32, 0),
                            end: Position::new(line_num as u32, line.len() as u32),
                        },
                        children: None,
                        deprecated: None,
                        tags: None,
                    });
                }
            }

            if let Some(name) = trimmed.strip_prefix("let ") {
                if let Some(end) = name.find('=') {
                    let var_name = name[..end].trim();
                    symbols.push(DocumentSymbol {
                        name: var_name.to_string(),
                        detail: Some("variable".to_string()),
                        kind: SymbolKind::VARIABLE,
                        range: Range {
                            start: Position::new(line_num as u32, 0),
                            end: Position::new(line_num as u32, line.len() as u32),
                        },
                        selection_range: Range {
                            start: Position::new(line_num as u32, 0),
                            end: Position::new(line_num as u32, line.len() as u32),
                        },
                        children: None,
                        deprecated: None,
                        tags: None,
                    });
                }
            }
        }

        symbols
    }

    fn analyze(source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Try to parse the source code
        match chen_lang::parser::parse_from_source(source) {
            Ok(_) => {
                // No errors
            }
            Err(e) => {
                // Extract error information
                let (message, line) = match &e {
                    chen_lang::parser::ParserError::Handwritten(err) => match err {
                        chen_lang::parser::handwritten::ParseError::Message { msg, loc } => (msg.clone(), loc.line),
                        chen_lang::parser::handwritten::ParseError::UnexpectedToken { token, loc } => {
                            (format!("Unexpected token: {:?}", token), loc.line)
                        }
                        chen_lang::parser::handwritten::ParseError::UnexpectedEndOfInput => (e.to_string(), 0),
                    },
                    chen_lang::parser::ParserError::Token(token_err) => match token_err {
                        chen_lang::tokenizer::TokenError::ParseErrorWithLocation { msg, line } => (msg.clone(), *line),
                        _ => (e.to_string(), 0),
                    },
                };

                let line_num = if line > 0 { line - 1 } else { 0 };

                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position::new(line_num, 0),
                        end: Position::new(line_num, 100),
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("chen_lang".to_string()),
                    message,
                    related_information: None,
                    tags: None,
                    data: None,
                });
            }
        }

        diagnostics
    }

    fn collect_symbols(source: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();

        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            // Find function definitions
            if let Some(name) = trimmed.strip_prefix("def ") {
                if let Some(end) = name.find('(') {
                    let func_name = name[..end].trim();
                    let start_col = line.find("def ").unwrap() + 4;
                    symbols.push(Symbol {
                        name: func_name.to_string(),
                        kind: SymbolKind::FUNCTION,
                        range: Range {
                            start: Position::new(line_num as u32, 0),
                            end: Position::new(line_num as u32, line.len() as u32),
                        },
                        definition_range: Range {
                            start: Position::new(line_num as u32, start_col as u32),
                            end: Position::new(line_num as u32, (start_col + func_name.len()) as u32),
                        },
                    });
                }
            }

            // Find variable declarations
            if let Some(name) = trimmed.strip_prefix("let ") {
                if let Some(end) = name.find('=') {
                    let var_name = name[..end].trim();
                    let start_col = line.find("let ").unwrap() + 4;
                    symbols.push(Symbol {
                        name: var_name.to_string(),
                        kind: SymbolKind::VARIABLE,
                        range: Range {
                            start: Position::new(line_num as u32, 0),
                            end: Position::new(line_num as u32, line.len() as u32),
                        },
                        definition_range: Range {
                            start: Position::new(line_num as u32, start_col as u32),
                            end: Position::new(line_num as u32, (start_col + var_name.len()) as u32),
                        },
                    });
                }
            }
        }

        symbols
    }

    fn find_references(source: &str, symbol_name: &str, uri: &Uri) -> Vec<Location> {
        let mut locations = Vec::new();

        for (line_num, line) in source.lines().enumerate() {
            let mut start = 0;
            while let Some(pos) = line[start..].find(symbol_name) {
                let actual_pos = start + pos;

                // Check if it's a whole word (not part of another identifier)
                let is_start_ok = actual_pos == 0 || {
                    let prev_char = line.chars().nth(actual_pos - 1).unwrap();
                    !prev_char.is_alphanumeric() && prev_char != '_'
                };

                let end_pos = actual_pos + symbol_name.len();
                let is_end_ok = end_pos >= line.len() || {
                    let next_char = line.chars().nth(end_pos).unwrap();
                    !next_char.is_alphanumeric() && next_char != '_'
                };

                if is_start_ok && is_end_ok {
                    locations.push(Location {
                        uri: uri.clone(),
                        range: Range {
                            start: Position::new(line_num as u32, actual_pos as u32),
                            end: Position::new(line_num as u32, end_pos as u32),
                        },
                    });
                }

                start = end_pos;
            }
        }

        locations
    }
}
