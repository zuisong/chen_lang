use std::sync::Arc;

use ropey::Rope;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, jsonrpc::Error};
use tracing::info;

use super::document::{Document, Documents};

pub type LspResult<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
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
        let line = text.line(position.line as usize);
        let line_str = line.to_string();

        let chars: Vec<char> = line_str.chars().collect();
        if position.character as usize >= chars.len() {
            return None;
        }

        let start = position.character as usize;
        let mut end = start;

        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }

        let mut scan_start = start;
        while scan_start > 0 && (chars[scan_start - 1].is_alphanumeric() || chars[scan_start - 1] == '_') {
            scan_start -= 1;
        }

        if scan_start < end {
            Some(line_str[scan_start..end].to_string())
        } else {
            None
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for ChenLangLsp {
    async fn initialize(&self, params: InitializeParams) -> LspResult<InitializeResult> {
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

        info!("Opened document: {}", uri);

        self.docs.insert(uri.clone(), text.clone());

        // Publish diagnostics on open
        let diagnostics = Self::analyze(&text);
        self.docs.update_diagnostics(&uri, diagnostics.clone());
        let _ = self.client.publish_diagnostics(uri, diagnostics, None).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;

        info!("Changed document: {}", uri);

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

        info!("Closed document: {}", uri);

        self.docs.remove(&uri);
        let _ = self.client.publish_diagnostics(uri, Vec::new(), None);
    }

    async fn completion(&self, params: CompletionParams) -> LspResult<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;

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

        Ok(None)
    }

    async fn references(&self, params: ReferenceParams) -> LspResult<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;

        if let Some(doc) = self.docs.get(&uri) {
            if let Some(word) = Self::get_word_at_position(&doc.text, pos) {
                let locations = Self::find_references(&doc.text.to_string(), &word, &uri);

                if !locations.is_empty() {
                    return Ok(Some(locations));
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
                let locations = Self::find_references(&doc.text.to_string(), &word, &uri);

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
                        chen_lang::parser::handwritten::ParseError::Message { msg, line } => (msg.clone(), *line),
                        chen_lang::parser::handwritten::ParseError::UnexpectedToken { token, line } => {
                            (format!("Unexpected token: {:?}", token), *line)
                        }
                        _ => (e.to_string(), 0),
                    },
                    chen_lang::parser::ParserError::Token(token_err) => match token_err {
                        chen_lang::tokenizer::TokenError::ParseErrorWithLocation { msg, line } => (msg.clone(), *line),
                        _ => (e.to_string(), 0),
                    },
                    _ => (e.to_string(), 0),
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

    fn find_references(source: &str, symbol_name: &str, uri: &Url) -> Vec<Location> {
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
