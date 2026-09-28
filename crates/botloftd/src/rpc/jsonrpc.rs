//! JSON-RPC 2.0 framing: parsing requests and writing responses and
//! notifications.

use botloft_core::protocol::error_code;
use serde_json::{Map, Value, json};

use crate::service::ApiError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

impl RpcError {
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl From<ApiError> for RpcError {
    fn from(err: ApiError) -> Self {
        let code = err.code();
        if code == error_code::INTERNAL_ERROR {
            // Store and I/O errors may name paths; they go to the log at
            // warn level, never message bodies or tokens.
            tracing::warn!(error = %err, source = ?std::error::Error::source(&err), "request failed");
        }
        Self::new(code, err.to_string())
    }
}

/// A well-formed request. `id` is `None` for a client notification, which
/// gets no response.
#[derive(Debug, Clone, PartialEq)]
pub struct Request {
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

/// Parses one WebSocket text frame. On failure returns the id to answer
/// with (`null` when unknown) and the error.
pub fn parse(text: &str) -> Result<Request, (Value, RpcError)> {
    let value: Value = serde_json::from_str(text).map_err(|_| {
        (
            Value::Null,
            RpcError::new(error_code::PARSE_ERROR, "parse error"),
        )
    })?;
    let Value::Object(mut object) = value else {
        return Err(invalid(Value::Null, "a request must be a JSON object"));
    };
    let id = object.remove("id");
    let reply_id = id.clone().unwrap_or(Value::Null);
    if !matches!(
        id,
        None | Some(Value::String(_) | Value::Number(_) | Value::Null)
    ) {
        return Err(invalid(
            Value::Null,
            "id must be a string, a number or null",
        ));
    }
    if object.get("jsonrpc") != Some(&Value::String("2.0".to_owned())) {
        return Err(invalid(reply_id, "jsonrpc must be \"2.0\""));
    }
    let Some(Value::String(method)) = object.remove("method") else {
        return Err(invalid(reply_id, "method must be a string"));
    };
    let params = object.remove("params");
    if !matches!(params, None | Some(Value::Object(_) | Value::Array(_))) {
        return Err(invalid(reply_id, "params must be an object or an array"));
    }
    Ok(Request { id, method, params })
}

fn invalid(id: Value, message: &str) -> (Value, RpcError) {
    (id, RpcError::new(error_code::INVALID_REQUEST, message))
}

pub fn success(id: &Value, result: Value) -> String {
    json!({ "jsonrpc": "2.0", "id": id, "result": result }).to_string()
}

pub fn failure(id: &Value, err: &RpcError) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": err.code, "message": err.message },
    })
    .to_string()
}

pub fn notification(method: &str, params: Value) -> String {
    json!({ "jsonrpc": "2.0", "method": method, "params": params }).to_string()
}

/// Params for methods that take none.
pub fn empty_params() -> Value {
    Value::Object(Map::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_request() {
        let req = parse(r#"{"jsonrpc":"2.0","id":7,"method":"crews.list"}"#).expect("parse");
        assert_eq!(req.id, Some(json!(7)));
        assert_eq!(req.method, "crews.list");
        assert_eq!(req.params, None);
    }

    #[test]
    fn a_request_without_id_is_a_notification() {
        let req = parse(r#"{"jsonrpc":"2.0","method":"x","params":{}}"#).expect("parse");
        assert_eq!(req.id, None);
    }

    #[test]
    fn rejects_malformed_frames_with_standard_codes() {
        let code = |text: &str| parse(text).map(|_| 0).unwrap_or_else(|(_, e)| e.code);
        assert_eq!(code("{"), error_code::PARSE_ERROR);
        assert_eq!(code("[1]"), error_code::INVALID_REQUEST);
        assert_eq!(
            code(r#"{"id":1,"method":"x"}"#),
            error_code::INVALID_REQUEST
        );
        assert_eq!(
            code(r#"{"jsonrpc":"2.0","id":{},"method":"x"}"#),
            error_code::INVALID_REQUEST
        );
        assert_eq!(
            code(r#"{"jsonrpc":"2.0","id":1,"method":2}"#),
            error_code::INVALID_REQUEST
        );
        assert_eq!(
            code(r#"{"jsonrpc":"2.0","id":1,"method":"x","params":3}"#),
            error_code::INVALID_REQUEST
        );
    }

    #[test]
    fn errors_keep_the_request_id_when_known() {
        let (id, _) = parse(r#"{"jsonrpc":"1.0","id":"a","method":"x"}"#).expect_err("invalid");
        assert_eq!(id, json!("a"));
    }

    #[test]
    fn writes_responses_and_notifications() {
        let ok: Value = serde_json::from_str(&success(&json!(1), json!([]))).expect("json");
        assert_eq!(ok, json!({"jsonrpc":"2.0","id":1,"result":[]}));
        let err = RpcError::new(-32002, "missing");
        let failed: Value = serde_json::from_str(&failure(&json!(1), &err)).expect("json");
        assert_eq!(failed["error"], json!({"code":-32002,"message":"missing"}));
        let note: Value =
            serde_json::from_str(&notification("crew.changed", json!({}))).expect("json");
        assert_eq!(
            note,
            json!({"jsonrpc":"2.0","method":"crew.changed","params":{}})
        );
    }
}
