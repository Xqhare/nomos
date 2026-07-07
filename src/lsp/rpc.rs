//! JSON-RPC models and mappings for LSP.

use athena::Object;
use mawu::XffValue;

/// A JSON-RPC request.
#[derive(Debug, Clone)]
pub struct Request {
    /// The request ID.
    pub id: XffValue,
    /// The method name.
    pub method: String,
    /// The parameters.
    pub params: Option<XffValue>,
}

/// A JSON-RPC response.
#[derive(Debug, Clone)]
pub struct Response {
    /// The response ID.
    pub id: XffValue,
    /// The success result.
    pub result: Option<XffValue>,
    /// The error object if the request failed.
    pub error: Option<ResponseError>,
}

/// A JSON-RPC response error.
#[derive(Debug, Clone)]
pub struct ResponseError {
    /// The error code.
    pub code: i64,
    /// The error message.
    pub message: String,
    /// Optional data containing additional error details.
    pub data: Option<XffValue>,
}

/// A JSON-RPC notification.
#[derive(Debug, Clone)]
pub struct Notification {
    /// The method name.
    pub method: String,
    /// The parameters.
    pub params: Option<XffValue>,
}

impl Request {
    /// Parses a request from an `XffValue`.
    pub fn from_xff(val: &XffValue) -> Option<Self> {
        let obj = val.as_object()?;
        let id = obj.get("id")?.clone();
        let method = obj.get("method")?.as_string()?.to_string();
        let params = obj.get("params").cloned();
        Some(Self { id, method, params })
    }
}

impl Response {
    /// Creates a successful response.
    pub fn new_success(id: XffValue, result: XffValue) -> Self {
        Self {
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Creates an error response.
    pub fn new_error(id: XffValue, code: i64, message: String, data: Option<XffValue>) -> Self {
        Self {
            id,
            result: None,
            error: Some(ResponseError { code, message, data }),
        }
    }

    /// Converts the response to an `XffValue`.
    pub fn to_xff(&self) -> XffValue {
        let mut obj = Object::new();
        obj.insert("jsonrpc", XffValue::from("2.0"));
        obj.insert("id", self.id.clone());
        if let Some(ref res) = self.result {
            obj.insert("result", res.clone());
        }
        if let Some(ref err) = self.error {
            let mut err_obj = Object::new();
            err_obj.insert("code", XffValue::from(err.code));
            err_obj.insert("message", XffValue::from(err.message.clone()));
            if let Some(ref data) = err.data {
                err_obj.insert("data", data.clone());
            }
            obj.insert("error", XffValue::from(err_obj));
        }
        obj.into()
    }
}

impl Notification {
    /// Parses a notification from an `XffValue`.
    pub fn from_xff(val: &XffValue) -> Option<Self> {
        let obj = val.as_object()?;
        let method = obj.get("method")?.as_string()?.to_string();
        let params = obj.get("params").cloned();
        Some(Self { method, params })
    }

    /// Converts the notification to an `XffValue`.
    pub fn to_xff(&self) -> XffValue {
        let mut obj = Object::new();
        obj.insert("jsonrpc", XffValue::from("2.0"));
        obj.insert("method", XffValue::from(self.method.clone()));
        if let Some(ref params) = self.params {
            obj.insert("params", params.clone());
        }
        obj.into()
    }
}
