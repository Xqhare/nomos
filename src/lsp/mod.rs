/// LSP features (completion, diagnostics, hover)
pub mod capabilities;
/// LSP JSON-RPC models
pub mod rpc;
/// Stdin/Stdout transport layer
pub mod transport;

use std::collections::HashMap;
use std::env;
use std::io::{BufRead, Write};
use std::path::PathBuf;

use athena::Object;
use mawu::XffValue;

use crate::error::NomosResult;
use crate::lsp::capabilities::{get_completions, get_diagnostics, get_hover, uri_to_path};
use crate::lsp::rpc::{Notification, Request, Response};
use crate::lsp::transport::{LspReader, LspWriter};
use crate::nomos::Nomos;

/// The main LSP Server state
pub struct LspServer {
    nomos: Option<Nomos>,
    shutdown: bool,
    buffers: HashMap<String, String>,
}

impl LspServer {
    /// Create a new LSP Server instance
    pub fn new() -> Self {
        Self {
            nomos: None,
            shutdown: false,
            buffers: HashMap::new(),
        }
    }

    /// Run the server loop over standard streams
    pub fn run<R: BufRead, W: Write>(
        &mut self,
        mut reader: LspReader<R>,
        mut writer: LspWriter<W>,
    ) -> NomosResult<()> {
        while !self.shutdown {
            if let Some(msg) = reader.read_frame()? {
                if let Some(obj) = msg.as_object() {
                    if obj.contains_key("method") {
                        if obj.contains_key("id") {
                            // Request
                            if let Some(req) = Request::from_xff(&msg) {
                                let res = self.handle_request(req);
                                writer.write_frame(&res.to_xff())?;
                            }
                        } else {
                            // Notification
                            if let Some(notif) = Notification::from_xff(&msg) {
                                self.handle_notification(notif, &mut writer)?;
                            }
                        }
                    }
                }
            } else {
                break; // EOF
            }
        }
        Ok(())
    }

    fn handle_request(&mut self, req: Request) -> Response {
        match req.method.as_str() {
            "initialize" => {
                let config_path = find_global_config();
                if let Some(path) = config_path {
                    self.nomos = Nomos::new(path).ok();
                }

                let mut capabilities = Object::new();
                capabilities.insert("textDocumentSync", XffValue::from(1)); // Full sync

                let mut completion = Object::new();
                completion.insert(
                    "triggerCharacters",
                    XffValue::from(vec![
                        XffValue::from("+"),
                        XffValue::from("@"),
                        XffValue::from("\""),
                        XffValue::from(":"),
                    ]),
                );
                capabilities.insert("completionProvider", XffValue::from(completion));
                capabilities.insert("hoverProvider", XffValue::from(true));

                let mut result = Object::new();
                result.insert("capabilities", XffValue::from(capabilities));

                Response::new_success(req.id, XffValue::from(result))
            }
            "shutdown" => {
                self.shutdown = true;
                Response::new_success(req.id, XffValue::Null)
            }
            "textDocument/completion" => {
                if let Some(ref params) = req.params
                    && let Some(obj) = params.as_object()
                    && let Some(text_doc) = obj.get("textDocument")
                    && let Some(text_doc_obj) = text_doc.as_object()
                    && let Some(uri) = text_doc_obj.get("uri")
                    && let Some(uri_str) = uri.as_string()
                    && let Some(pos) = obj.get("position")
                    && let Some(pos_obj) = pos.as_object()
                    && let Some(line_val) = pos_obj.get("line")
                    && let Some(char_val) = pos_obj.get("character")
                {
                    let line = line_val
                        .as_number()
                        .and_then(|n| n.into_usize())
                        .unwrap_or(0);
                    let character = char_val
                        .as_number()
                        .and_then(|n| n.into_usize())
                        .unwrap_or(0);

                    let empty = String::new();
                    let content = self.buffers.get(uri_str).unwrap_or(&empty);
                    let lines: Vec<&str> = content.lines().collect();
                    let current_line = if line < lines.len() { lines[line] } else { "" };

                    // Deduce current project from URI path
                    let path_str = uri_to_path(uri_str);
                    let path = std::path::Path::new(&path_str);
                    let current_project = path
                        .parent()
                        .and_then(|p| p.file_name())
                        .and_then(|n| n.to_str())
                        .unwrap_or("");

                    let completions =
                        get_completions(&self.nomos, current_line, character, current_project);
                    Response::new_success(req.id, completions)
                } else {
                    Response::new_error(req.id, -32602, "Invalid params".to_string(), None)
                }
            }
            "textDocument/hover" => {
                if let Some(ref params) = req.params
                    && let Some(obj) = params.as_object()
                    && let Some(text_doc) = obj.get("textDocument")
                    && let Some(text_doc_obj) = text_doc.as_object()
                    && let Some(uri) = text_doc_obj.get("uri")
                    && let Some(uri_str) = uri.as_string()
                    && let Some(pos) = obj.get("position")
                    && let Some(pos_obj) = pos.as_object()
                    && let Some(line_val) = pos_obj.get("line")
                    && let Some(char_val) = pos_obj.get("character")
                {
                    let line = line_val
                        .as_number()
                        .and_then(|n| n.into_usize())
                        .unwrap_or(0);
                    let character = char_val
                        .as_number()
                        .and_then(|n| n.into_usize())
                        .unwrap_or(0);

                    let empty = String::new();
                    let content = self.buffers.get(uri_str).unwrap_or(&empty);
                    let lines: Vec<&str> = content.lines().collect();
                    let current_line = if line < lines.len() { lines[line] } else { "" };

                    let hover_val = get_hover(&self.nomos, current_line, character);
                    Response::new_success(req.id, hover_val)
                } else {
                    Response::new_error(req.id, -32602, "Invalid params".to_string(), None)
                }
            }
            _ => Response::new_error(
                req.id,
                -32601,
                format!("Method not found: {}", req.method),
                None,
            ),
        }
    }

    fn handle_notification<W: Write>(
        &mut self,
        notif: Notification,
        writer: &mut LspWriter<W>,
    ) -> NomosResult<()> {
        match notif.method.as_str() {
            "exit" => {
                self.shutdown = true;
            }
            "textDocument/didOpen" => {
                if let Some(ref params) = notif.params
                    && let Some(obj) = params.as_object()
                    && let Some(text_doc) = obj.get("textDocument")
                    && let Some(text_doc_obj) = text_doc.as_object()
                    && let Some(uri) = text_doc_obj.get("uri")
                    && let Some(uri_str) = uri.as_string()
                    && let Some(text) = text_doc_obj.get("text")
                    && let Some(text_str) = text.as_string()
                {
                    self.buffers
                        .insert(uri_str.to_string(), text_str.to_string());
                    let diag = get_diagnostics(uri_str, text_str);
                    let diag_notif = Notification {
                        method: "textDocument/publishDiagnostics".to_string(),
                        params: Some(diag),
                    };
                    writer.write_frame(&diag_notif.to_xff())?;
                }
            }
            "textDocument/didChange" => {
                if let Some(ref params) = notif.params
                    && let Some(obj) = params.as_object()
                    && let Some(text_doc) = obj.get("textDocument")
                    && let Some(text_doc_obj) = text_doc.as_object()
                    && let Some(uri) = text_doc_obj.get("uri")
                    && let Some(uri_str) = uri.as_string()
                    && let Some(content_changes) = obj.get("contentChanges")
                    && let Some(changes_arr) = content_changes.as_array()
                    && let Some(first_change) = changes_arr.get(0)
                    && let Some(change_obj) = first_change.as_object()
                    && let Some(text) = change_obj.get("text")
                    && let Some(text_str) = text.as_string()
                {
                    self.buffers
                        .insert(uri_str.to_string(), text_str.to_string());
                    let diag = get_diagnostics(uri_str, text_str);
                    let diag_notif = Notification {
                        method: "textDocument/publishDiagnostics".to_string(),
                        params: Some(diag),
                    };
                    writer.write_frame(&diag_notif.to_xff())?;
                }
            }
            "textDocument/didSave" => {
                // Refresh global workspace state
                let config_path = find_global_config();
                if let Some(path) = config_path {
                    self.nomos = Nomos::new(path).ok();
                }
            }
            _ => {}
        }
        Ok(())
    }
}

fn find_global_config() -> Option<PathBuf> {
    if let Ok(home) = env::var("HOME") {
        let path1 = PathBuf::from(&home).join(".config/nomos/config.json");
        if path1.exists() {
            return Some(path1);
        }
        let path2 = PathBuf::from(&home).join(".nomos/config.json");
        if path2.exists() {
            return Some(path2);
        }
    }
    None
}
