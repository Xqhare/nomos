/// LSP JSON-RPC models
pub mod rpc;
/// Stdin/Stdout transport layer
pub mod transport;

use std::env;
use std::io::{BufRead, Write};
use std::path::PathBuf;

use athena::Object;
use mawu::XffValue;

use crate::error::NomosResult;
use crate::nomos::Nomos;
use crate::lsp::rpc::{Notification, Request, Response};
use crate::lsp::transport::{LspReader, LspWriter};

/// The main LSP Server state
pub struct LspServer {
    nomos: Option<Nomos>,
    shutdown: bool,
}

impl LspServer {
    /// Create a new LSP Server instance
    pub fn new() -> Self {
        Self {
            nomos: None,
            shutdown: false,
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
                                self.handle_notification(notif);
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
            _ => Response::new_error(
                req.id,
                -32601,
                format!("Method not found: {}", req.method),
                None,
            ),
        }
    }

    fn handle_notification(&mut self, notif: Notification) {
        match notif.method.as_str() {
            "exit" => {
                self.shutdown = true;
            }
            _ => {}
        }
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
