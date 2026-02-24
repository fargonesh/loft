use crate::runtime::builtin::{BuiltinMethod, BuiltinStruct};
use crate::runtime::permission_context::check_net_permission;
use crate::runtime::value::Value;
use crate::runtime::{RuntimeError, RuntimeResult};
use loft_builtin_macros::loft_builtin;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde_json;
use std::collections::HashMap;

/// Buffer type for representing binary data
#[derive(Clone, Debug, PartialEq)]
pub struct Buffer {
    pub data: Vec<u8>,
}

impl Buffer {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn from_string(s: &str) -> Self {
        Self {
            data: s.as_bytes().to_vec(),
        }
    }

    pub fn to_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.data.clone())
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// HTTP method enumeration
#[derive(Clone, Debug, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
    TRACE,
    CONNECT,
}

impl HttpMethod {
    pub fn from_string(s: &str) -> Result<Self, RuntimeError> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            "PUT" => Ok(HttpMethod::PUT),
            "DELETE" => Ok(HttpMethod::DELETE),
            "PATCH" => Ok(HttpMethod::PATCH),
            "HEAD" => Ok(HttpMethod::HEAD),
            "OPTIONS" => Ok(HttpMethod::OPTIONS),
            "TRACE" => Ok(HttpMethod::TRACE),
            "CONNECT" => Ok(HttpMethod::CONNECT),
            _ => Err(RuntimeError::new(format!("Invalid HTTP method: {}", s))),
        }
    }

    pub fn to_reqwest_method(&self) -> reqwest::Method {
        match self {
            HttpMethod::GET => reqwest::Method::GET,
            HttpMethod::POST => reqwest::Method::POST,
            HttpMethod::PUT => reqwest::Method::PUT,
            HttpMethod::DELETE => reqwest::Method::DELETE,
            HttpMethod::PATCH => reqwest::Method::PATCH,
            HttpMethod::HEAD => reqwest::Method::HEAD,
            HttpMethod::OPTIONS => reqwest::Method::OPTIONS,
            HttpMethod::TRACE => reqwest::Method::TRACE,
            HttpMethod::CONNECT => reqwest::Method::CONNECT,
        }
    }
}

/// HTTP Response structure
#[derive(Clone, Debug)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Buffer,
}

impl HttpResponse {
    pub fn new(status: u16, headers: HashMap<String, String>, body: Buffer) -> Self {
        Self {
            status,
            headers,
            body,
        }
    }
}

/// HTTP Request Builder structure
#[derive(Clone, Debug)]
pub struct RequestBuilder {
    pub url: String,
    pub method: HttpMethod,
    pub headers: HashMap<String, String>,
    pub body: Option<Buffer>,
    pub timeout: Option<u64>,
    pub follow_redirects: bool,
}

impl RequestBuilder {
    pub fn new(url: String) -> Self {
        Self {
            url,
            method: HttpMethod::GET,
            headers: HashMap::new(),
            body: None,
            timeout: None,
            follow_redirects: true,
        }
    }
}

// Convert Buffer to/from Value
impl From<Buffer> for Value {
    fn from(buffer: Buffer) -> Self {
        // Represent Buffer as a struct with data field
        let mut fields = HashMap::new();
        let length = buffer.len();
        fields.insert(
            "data".to_string(),
            Value::Array(
                buffer
                    .data
                    .into_iter()
                    .map(|b| Value::Number(Decimal::from(b)))
                    .collect(),
            ),
        );
        fields.insert("length".to_string(), Value::Number(Decimal::from(length)));

        Value::Struct {
            name: "Buffer".to_string(),
            fields,
        }
    }
}

impl TryFrom<&Value> for Buffer {
    type Error = RuntimeError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Struct { name, fields } if name == "Buffer" => {
                if let Some(Value::Array(data_array)) = fields.get("data") {
                    let mut data = Vec::new();
                    for item in data_array {
                        if let Value::Number(n) = item {
                            if let Some(byte) = n.to_u8() {
                                data.push(byte);
                            } else {
                                return Err(RuntimeError::new(
                                    "Buffer data must contain valid bytes (0-255)",
                                ));
                            }
                        } else {
                            return Err(RuntimeError::new(
                                "Buffer data must be an array of numbers",
                            ));
                        }
                    }
                    Ok(Buffer::new(data))
                } else {
                    Err(RuntimeError::new("Buffer struct must have a 'data' field"))
                }
            }
            Value::String(s) => Ok(Buffer::from_string(s)),
            _ => Err(RuntimeError::new("Cannot convert value to Buffer")),
        }
    }
}

// Convert HttpResponse to Value
impl From<HttpResponse> for Value {
    fn from(response: HttpResponse) -> Self {
        let mut fields = HashMap::new();
        fields.insert(
            "status".to_string(),
            Value::Number(Decimal::from(response.status)),
        );

        // Convert headers to Value::Struct
        let mut header_fields = HashMap::new();
        for (key, value) in response.headers {
            header_fields.insert(key, Value::String(value));
        }
        fields.insert(
            "headers".to_string(),
            Value::Struct {
                name: "Headers".to_string(),
                fields: header_fields,
            },
        );

        // Body as a Promise<Buffer>
        fields.insert(
            "body".to_string(),
            Value::Promise(Box::new(response.body.into())),
        );

        Value::Struct {
            name: "Response".to_string(),
            fields,
        }
    }
}

// Convert RequestBuilder to Value
impl From<RequestBuilder> for Value {
    fn from(builder: RequestBuilder) -> Self {
        let mut fields = HashMap::new();
        fields.insert("url".to_string(), Value::String(builder.url));
        fields.insert(
            "method".to_string(),
            Value::String(format!("{:?}", builder.method)),
        );

        let mut header_fields = HashMap::new();
        for (key, value) in builder.headers {
            header_fields.insert(key, Value::String(value));
        }
        fields.insert(
            "headers".to_string(),
            Value::Struct {
                name: "Headers".to_string(),
                fields: header_fields,
            },
        );

        if let Some(body) = builder.body {
            fields.insert("body".to_string(), body.into());
        }

        if let Some(timeout) = builder.timeout {
            fields.insert("timeout".to_string(), Value::Number(Decimal::from(timeout)));
        }

        fields.insert(
            "followRedirects".to_string(),
            Value::Boolean(builder.follow_redirects),
        );

        Value::Struct {
            name: "RequestBuilder".to_string(),
            fields,
        }
    }
}

/// Create a new HTTP request builder
#[loft_builtin(web.request)]
fn web_request(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("web.request() requires a URL argument"));
    }

    let url = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(RuntimeError::new("web.request() URL must be a string")),
    };

    let builder = RequestBuilder::new(url);
    Ok(builder.into())
}

/// Set HTTP method on request builder
#[loft_builtin(web.method)]
fn web_method(this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("method() requires a method argument"));
    }

    let method_str = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(RuntimeError::new("method() argument must be a string")),
    };

    let method = HttpMethod::from_string(&method_str)?;

    if let Value::Struct { name, mut fields } = this.clone() {
        if name == "RequestBuilder" {
            fields.insert("method".to_string(), Value::String(format!("{:?}", method)));
            return Ok(Value::Struct { name, fields });
        }
    }

    Err(RuntimeError::new(
        "method() can only be called on RequestBuilder",
    ))
}

/// Add header to request builder
#[loft_builtin(web.header)]
fn web_header(this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.len() < 2 {
        return Err(RuntimeError::new(
            "header() requires key and value arguments",
        ));
    }

    let key = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(RuntimeError::new("header() key must be a string")),
    };

    let value = match &args[1] {
        Value::String(s) => s.clone(),
        _ => return Err(RuntimeError::new("header() value must be a string")),
    };

    if let Value::Struct { name, mut fields } = this.clone() {
        if name == "RequestBuilder" {
            // Get existing headers or create new ones
            let mut header_fields = HashMap::new();
            if let Some(Value::Struct {
                fields: existing_headers,
                ..
            }) = fields.get("headers")
            {
                header_fields = existing_headers.clone();
            }

            header_fields.insert(key, Value::String(value));
            fields.insert(
                "headers".to_string(),
                Value::Struct {
                    name: "Headers".to_string(),
                    fields: header_fields,
                },
            );

            return Ok(Value::Struct { name, fields });
        }
    }

    Err(RuntimeError::new(
        "header() can only be called on RequestBuilder",
    ))
}

/// Set request body
#[loft_builtin(web.body)]
fn web_body(this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("body() requires a body argument"));
    }

    let body = Buffer::try_from(&args[0])?;

    if let Value::Struct { name, mut fields } = this.clone() {
        if name == "RequestBuilder" {
            fields.insert("body".to_string(), body.into());
            return Ok(Value::Struct { name, fields });
        }
    }

    Err(RuntimeError::new(
        "body() can only be called on RequestBuilder",
    ))
}

/// Set request timeout in milliseconds
#[loft_builtin(web.timeout)]
fn web_timeout(this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("timeout() requires a timeout argument"));
    }

    let timeout_ms = match &args[0] {
        Value::Number(n) => n
            .to_u64()
            .ok_or_else(|| RuntimeError::new("timeout must be a positive number"))?,
        _ => return Err(RuntimeError::new("timeout() argument must be a number")),
    };

    if let Value::Struct { name, mut fields } = this.clone() {
        if name == "RequestBuilder" {
            fields.insert(
                "timeout".to_string(),
                Value::Number(Decimal::from(timeout_ms)),
            );
            return Ok(Value::Struct { name, fields });
        }
    }

    Err(RuntimeError::new(
        "timeout() can only be called on RequestBuilder",
    ))
}

/// Set whether to follow redirects
#[loft_builtin(web.followRedirects)]
fn web_follow_redirects(this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new(
            "followRedirects() requires a boolean argument",
        ));
    }

    let follow = match &args[0] {
        Value::Boolean(b) => *b,
        _ => {
            return Err(RuntimeError::new(
                "followRedirects() argument must be a boolean",
            ))
        }
    };

    if let Value::Struct { name, mut fields } = this.clone() {
        if name == "RequestBuilder" {
            fields.insert("followRedirects".to_string(), Value::Boolean(follow));
            return Ok(Value::Struct { name, fields });
        }
    }

    Err(RuntimeError::new(
        "followRedirects() can only be called on RequestBuilder",
    ))
}

/// Send HTTP request and return response
#[loft_builtin(web.send)]
fn web_send(this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    if let Value::Struct { name, fields } = this {
        if name == "RequestBuilder" {
            // Extract request details
            let url = if let Some(Value::String(url)) = fields.get("url") {
                url.clone()
            } else {
                return Err(RuntimeError::new("RequestBuilder missing URL"));
            };

            // Extract host from URL for permission check
            let host = url::Url::parse(&url)
                .map(|u| u.host_str().unwrap_or("unknown").to_string())
                .unwrap_or_else(|_| url.clone());

            // Check network permission
            check_net_permission(&host, Some("web.send()")).map_err(|e| RuntimeError::new(e))?;

            let method_str = if let Some(Value::String(method)) = fields.get("method") {
                method.clone()
            } else {
                "GET".to_string()
            };

            let method = HttpMethod::from_string(&method_str)?;

            // Build reqwest client and request
            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_millis(30000)) // Default 30s timeout
                .build()
                .map_err(|e| RuntimeError::new(format!("Failed to create HTTP client: {}", e)))?;

            let mut request = client.request(method.to_reqwest_method(), &url);

            // Add headers
            if let Some(Value::Struct {
                fields: header_fields,
                ..
            }) = fields.get("headers")
            {
                for (key, value) in header_fields {
                    if let Value::String(header_value) = value {
                        request = request.header(key, header_value);
                    }
                }
            }

            // Add body
            if let Some(body_value) = fields.get("body") {
                let buffer = Buffer::try_from(body_value)?;
                request = request.body(buffer.data);
            }

            // Set timeout
            if let Some(Value::Number(timeout)) = fields.get("timeout") {
                if let Some(timeout_ms) = timeout.to_u64() {
                    request = request.timeout(std::time::Duration::from_millis(timeout_ms));
                }
            }

            // Execute request
            let response = request
                .send()
                .map_err(|e| RuntimeError::new(format!("HTTP request failed: {}", e)))?;

            // Extract response data
            let status = response.status().as_u16();

            let mut headers = HashMap::new();
            for (key, value) in response.headers() {
                if let Ok(value_str) = value.to_str() {
                    headers.insert(key.to_string(), value_str.to_string());
                }
            }

            let body_bytes = response
                .bytes()
                .map_err(|e| RuntimeError::new(format!("Failed to read response body: {}", e)))?;
            let body = Buffer::new(body_bytes.to_vec());

            let http_response = HttpResponse::new(status, headers, body);
            return Ok(Value::Promise(Box::new(http_response.into())));
        }
    }

    Err(RuntimeError::new(
        "send() can only be called on RequestBuilder",
    ))
}

/// Parse response body as JSON
#[loft_builtin(web.json)]
fn web_json(this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    if let Value::Struct { name, fields } = this {
        if name == "Response" {
            if let Some(Value::Promise(body_promise)) = fields.get("body") {
                if let Value::Struct {
                    name: buffer_name,
                    fields: _buffer_fields,
                } = body_promise.as_ref()
                {
                    if buffer_name == "Buffer" {
                        let buffer = Buffer::try_from(body_promise.as_ref())?;
                        let json_str = buffer.to_string().map_err(|e| {
                            RuntimeError::new(format!("Invalid UTF-8 in response body: {}", e))
                        })?;

                        let json_value: serde_json::Value = serde_json::from_str(&json_str)
                            .map_err(|e| RuntimeError::new(format!("Invalid JSON: {}", e)))?;

                        let twang_value = json_to_loft_value(json_value)?;
                        return Ok(Value::Promise(Box::new(twang_value)));
                    }
                }
            }
        }
    }

    Err(RuntimeError::new("json() can only be called on Response"))
}

/// Parse response body as text
#[loft_builtin(web.text)]
fn web_text(this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    if let Value::Struct { name, fields } = this {
        if name == "Response" {
            if let Some(Value::Promise(body_promise)) = fields.get("body") {
                let buffer = Buffer::try_from(body_promise.as_ref())?;
                let text = buffer.to_string().map_err(|e| {
                    RuntimeError::new(format!("Invalid UTF-8 in response body: {}", e))
                })?;

                return Ok(Value::Promise(Box::new(Value::String(text))));
            }
        }
    }

    Err(RuntimeError::new("text() can only be called on Response"))
}

/// Create a Buffer from string
#[loft_builtin(web.buffer)]
fn web_buffer(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Ok(Buffer::new(Vec::new()).into());
    }

    match &args[0] {
        Value::String(s) => Ok(Buffer::from_string(s).into()),
        Value::Array(arr) => {
            let mut data = Vec::new();
            for item in arr {
                if let Value::Number(n) = item {
                    if let Some(byte) = n.to_u8() {
                        data.push(byte);
                    } else {
                        return Err(RuntimeError::new(
                            "Buffer data must contain valid bytes (0-255)",
                        ));
                    }
                } else {
                    return Err(RuntimeError::new("Buffer data must be an array of numbers"));
                }
            }
            Ok(Buffer::new(data).into())
        }
        _ => Err(RuntimeError::new(
            "Buffer can be created from string or array of numbers",
        )),
    }
}

/// Helper function to convert serde_json::Value to loft Value
fn json_to_loft_value(json: serde_json::Value) -> RuntimeResult<Value> {
    match json {
        serde_json::Value::Null => Ok(Value::Unit),
        serde_json::Value::Bool(b) => Ok(Value::Boolean(b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Number(Decimal::from(i)))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Number(Decimal::try_from(f).map_err(|e| {
                    RuntimeError::new(format!("Invalid number: {}", e))
                })?))
            } else {
                Err(RuntimeError::new("Invalid JSON number"))
            }
        }
        serde_json::Value::String(s) => Ok(Value::String(s)),
        serde_json::Value::Array(arr) => {
            let mut values = Vec::new();
            for item in arr {
                values.push(json_to_loft_value(item)?);
            }
            Ok(Value::Array(values))
        }
        serde_json::Value::Object(obj) => {
            let mut fields = HashMap::new();
            for (key, value) in obj {
                fields.insert(key, json_to_loft_value(value)?);
            }
            Ok(Value::Struct {
                name: "Object".to_string(),
                fields,
            })
        }
    }
}

/// GET request shorthand
#[loft_builtin(web.get)]
fn web_get(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("web.get() requires a URL argument"));
    }

    let url = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(RuntimeError::new("web.get() URL must be a string")),
    };

    let builder = RequestBuilder::new(url);
    let builder_value: Value = builder.into();

    // Call send() on the builder
    web_send(&builder_value, &[])
}

/// POST request shorthand with optional body
#[loft_builtin(web.post)]
fn web_post(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("web.post() requires a URL argument"));
    }

    let url = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(RuntimeError::new("web.post() URL must be a string")),
    };

    let mut builder = RequestBuilder::new(url);
    builder.method = HttpMethod::POST;

    // Add body if provided
    if args.len() > 1 {
        builder.body = Some(Buffer::try_from(&args[1])?);
    }

    let builder_value: Value = builder.into();
    web_send(&builder_value, &[])
}

/// PUT request shorthand with optional body
#[loft_builtin(web.put)]
fn web_put(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("web.put() requires a URL argument"));
    }

    let url = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(RuntimeError::new("web.put() URL must be a string")),
    };

    let mut builder = RequestBuilder::new(url);
    builder.method = HttpMethod::PUT;

    // Add body if provided
    if args.len() > 1 {
        builder.body = Some(Buffer::try_from(&args[1])?);
    }

    let builder_value: Value = builder.into();
    web_send(&builder_value, &[])
}

/// DELETE request shorthand
#[loft_builtin(web.delete)]
fn web_delete(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("web.delete() requires a URL argument"));
    }

    let url = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(RuntimeError::new("web.delete() URL must be a string")),
    };

    let mut builder = RequestBuilder::new(url);
    builder.method = HttpMethod::DELETE;

    let builder_value: Value = builder.into();
    web_send(&builder_value, &[])
}

/// Create the Web builtin struct
pub fn create_web_builtin() -> BuiltinStruct {
    let mut web = BuiltinStruct::new("web");

    // Request building methods
    web.add_method("request", web_request as BuiltinMethod);
    web.add_method("method", web_method as BuiltinMethod);
    web.add_method("header", web_header as BuiltinMethod);
    web.add_method("body", web_body as BuiltinMethod);
    web.add_method("timeout", web_timeout as BuiltinMethod);
    web.add_method("followRedirects", web_follow_redirects as BuiltinMethod);
    web.add_method("send", web_send as BuiltinMethod);

    // Response processing methods
    web.add_method("json", web_json as BuiltinMethod);
    web.add_method("text", web_text as BuiltinMethod);

    // Utility methods
    web.add_method("buffer", web_buffer as BuiltinMethod);

    // Shorthand HTTP methods
    web.add_method("get", web_get as BuiltinMethod);
    web.add_method("post", web_post as BuiltinMethod);
    web.add_method("put", web_put as BuiltinMethod);
    web.add_method("delete", web_delete as BuiltinMethod);

    web
}

// Register the builtin automatically
crate::submit_builtin!("web", create_web_builtin);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_creation() {
        let buffer = Buffer::from_string("Hello");
        assert_eq!(buffer.len(), 5);
        assert_eq!(buffer.to_string().unwrap(), "Hello");
    }

    #[test]
    fn test_buffer_to_value() {
        let buffer = Buffer::from_string("Test");
        let value: Value = buffer.into();

        match value {
            Value::Struct { name, .. } => {
                assert_eq!(name, "Buffer");
            }
            _ => panic!("Expected Struct"),
        }
    }

    #[test]
    fn test_http_method_parsing() {
        assert!(HttpMethod::from_string("GET").is_ok());
        assert!(HttpMethod::from_string("POST").is_ok());
        assert!(HttpMethod::from_string("PUT").is_ok());
        assert!(HttpMethod::from_string("DELETE").is_ok());
        assert!(HttpMethod::from_string("get").is_ok());
        assert!(HttpMethod::from_string("INVALID").is_err());
    }

    #[test]
    fn test_request_builder_creation() {
        let builder = RequestBuilder::new("https://example.com".to_string());
        assert_eq!(builder.url, "https://example.com");
        assert_eq!(builder.method, HttpMethod::GET);
        assert!(builder.follow_redirects);
        assert!(builder.body.is_none());
    }

    #[test]
    fn test_web_buffer_function() {
        let result = web_buffer(&Value::Unit, &[Value::String("Test".to_string())]);
        assert!(result.is_ok());

        let value = result.unwrap();
        match value {
            Value::Struct { name, .. } => {
                assert_eq!(name, "Buffer");
            }
            _ => panic!("Expected Struct"),
        }
    }

    #[test]
    fn test_web_request_creation() {
        let result = web_request(
            &Value::Unit,
            &[Value::String("https://example.com".to_string())],
        );
        assert!(result.is_ok());

        let value = result.unwrap();
        match value {
            Value::Struct { name, .. } => {
                assert_eq!(name, "RequestBuilder");
            }
            _ => panic!("Expected RequestBuilder struct"),
        }
    }
}
