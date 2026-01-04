use std::clone::Clone;
use std::sync::Arc;

use dashmap::DashMap;
use ropey::Rope;
use tower_lsp_server::ls_types::*;

pub struct Document {
    pub text: Rope,
    pub version: i32,
    pub diagnostics: Vec<Diagnostic>,
}

impl Clone for Document {
    fn clone(&self) -> Self {
        Self {
            text: self.text.clone(),
            version: self.version,
            diagnostics: self.diagnostics.clone(),
        }
    }
}

pub struct Documents {
    docs: Arc<DashMap<Uri, Document>>,
}

impl Documents {
    pub fn new() -> Self {
        Self {
            docs: Arc::new(DashMap::new()),
        }
    }

    pub fn insert(&self, uri: Uri, text: String) {
        self.docs.insert(
            uri.clone(),
            Document {
                text: Rope::from_str(&text),
                version: 0,
                diagnostics: Vec::new(),
            },
        );
    }

    pub fn update(&self, uri: Uri, changes: Vec<TextDocumentContentChangeEvent>) {
        if let Some(mut doc) = self.docs.get_mut(&uri) {
            for change in changes {
                if let Some(range) = change.range {
                    let start = Self::lsp_pos_to_utf16(&doc.text, range.start);
                    let end = Self::lsp_pos_to_utf16(&doc.text, range.end);

                    let text = change.text;
                    if text.is_empty() {
                        doc.text.remove(start..end);
                    } else {
                        doc.text.remove(start..end);
                        doc.text.insert(start, &text);
                    }
                } else {
                    let text = change.text;
                    doc.text = Rope::from_str(&text);
                }
            }
            doc.version += 1;
        }
    }

    pub fn get(&self, uri: &Uri) -> Option<Document> {
        self.docs.get(uri).map(|d| Document {
            text: d.text.clone(),
            version: d.version,
            diagnostics: d.diagnostics.clone(),
        })
    }

    pub fn remove(&self, uri: &Uri) {
        self.docs.remove(uri);
    }

    pub fn update_diagnostics(&self, uri: &Uri, diagnostics: Vec<Diagnostic>) {
        if let Some(mut doc) = self.docs.get_mut(uri) {
            doc.diagnostics = diagnostics;
        }
    }

    fn lsp_pos_to_utf16(text: &Rope, pos: Position) -> usize {
        let line = text.line(pos.line as usize);
        let line_start = text.line_to_char(pos.line as usize);
        let target_utf16 = pos.character as usize;
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

        line_start + char_count
    }
}
