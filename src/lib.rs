use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write as IoWrite;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_LOG_SIZE: usize = 8000;

pub trait LogMessage {
    fn to_log_string(&self) -> String;
}

impl LogMessage for &str {
    fn to_log_string(&self) -> String {
        self.to_string()
    }
}

impl LogMessage for String {
    fn to_log_string(&self) -> String {
        self.clone()
    }
}

impl LogMessage for i32 {
    fn to_log_string(&self) -> String {
        self.to_string()
    }
}

impl LogMessage for i64 {
    fn to_log_string(&self) -> String {
        self.to_string()
    }
}

impl LogMessage for u32 {
    fn to_log_string(&self) -> String {
        self.to_string()
    }
}

impl LogMessage for u64 {
    fn to_log_string(&self) -> String {
        self.to_string()
    }
}

impl LogMessage for f32 {
    fn to_log_string(&self) -> String {
        self.to_string()
    }
}

impl LogMessage for f64 {
    fn to_log_string(&self) -> String {
        self.to_string()
    }
}

impl LogMessage for bool {
    fn to_log_string(&self) -> String {
        self.to_string()
    }
}

impl LogMessage for serde_json::Value {
    fn to_log_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| format!("{:?}", self))
    }
}

impl LogMessage for Vec<&str> {
    fn to_log_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| format!("{:?}", self))
    }
}

impl LogMessage for Vec<String> {
    fn to_log_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| format!("{:?}", self))
    }
}

#[derive(Clone)]
pub struct Context {
    pub req: ContextRequest,
    pub res: ContextResponse,
    logger: Logger,
}

impl Context {
    pub fn new(logger: Logger) -> Self {
        Context {
            req: ContextRequest::new(),
            res: ContextResponse::new(),
            logger,
        }
    }

    pub fn log<T: LogMessage>(&self, message: T) {
        let msg = message.to_log_string();
        self.logger.write(vec![msg], LoggerType::Log, false);
    }

    pub fn log_multiple(&self, messages: Vec<String>) {
        self.logger.write(messages, LoggerType::Log, false);
    }

    pub fn error<T: LogMessage>(&self, message: T) {
        let msg = message.to_log_string();
        self.logger.write(vec![msg], LoggerType::Error, false);
    }

    pub fn error_multiple(&self, messages: Vec<String>) {
        self.logger.write(messages, LoggerType::Error, false);
    }

    pub fn get_logger(&self) -> &Logger {
        &self.logger
    }

    pub fn get_logger_mut(&mut self) -> &mut Logger {
        &mut self.logger
    }
}

#[derive(Clone, Debug)]
pub struct ContextRequest {
    pub headers: HashMap<String, String>,
    pub method: String,
    pub url: String,
    pub scheme: String,
    pub host: String,
    pub port: u16,
    pub path: String,
    pub query_string: String,
    pub query: HashMap<String, String>,
    body_binary: Vec<u8>,
    body_parsed: Option<serde_json::Value>,
}

impl ContextRequest {
    pub fn new() -> Self {
        ContextRequest {
            headers: HashMap::new(),
            method: String::new(),
            url: String::new(),
            scheme: String::new(),
            host: String::new(),
            port: 80,
            path: String::new(),
            query_string: String::new(),
            query: HashMap::new(),
            body_binary: Vec::new(),
            body_parsed: None,
        }
    }

    pub fn set_body_binary(&mut self, data: Vec<u8>) {
        self.body_binary = data;
        self.body_parsed = None;
    }

    pub fn body_binary(&self) -> Vec<u8> {
        self.body_binary.clone()
    }

    pub fn body_text(&self) -> String {
        String::from_utf8_lossy(&self.body_binary).to_string()
    }

    pub fn body_json<T>(&mut self) -> Result<T, serde_json::Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        if self.body_parsed.is_none() {
            let value: serde_json::Value = serde_json::from_slice(&self.body_binary)?;
            self.body_parsed = Some(value);
        }

        if let Some(ref parsed) = self.body_parsed {
            serde_json::from_value(parsed.clone())
        } else {
            serde_json::from_slice(&self.body_binary)
        }
    }

    pub fn body(&mut self) -> serde_json::Value {
        let content_type = self
            .headers
            .get("content-type")
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        if content_type.contains("application/json") {
            if self.body_binary.is_empty() {
                return serde_json::Value::Object(serde_json::Map::new());
            }

            if self.body_parsed.is_none() {
                if let Ok(value) = serde_json::from_slice(&self.body_binary) {
                    self.body_parsed = Some(value);
                }
            }
            self.body_parsed
                .clone()
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()))
        } else {
            serde_json::Value::String(self.body_text())
        }
    }

    #[deprecated(note = "Use body_binary() instead")]
    pub fn body_raw(&self) -> Vec<u8> {
        self.body_binary()
    }
}

impl Default for ContextRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct Response {
    pub status_code: u16,
    pub body: Vec<u8>,
    pub headers: HashMap<String, String>,
}

impl Response {
    pub fn new() -> Self {
        Response {
            status_code: 200,
            body: Vec::new(),
            headers: HashMap::new(),
        }
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct ContextResponse;

impl ContextResponse {
    pub fn new() -> Self {
        ContextResponse
    }

    pub fn text<S: Into<String>>(
        &self,
        text: S,
        status_code: Option<u16>,
        headers: Option<HashMap<String, String>>,
    ) -> Response {
        let text_string = text.into();
        let mut response = Response {
            status_code: status_code.unwrap_or(200),
            body: text_string.into_bytes(),
            headers: headers.unwrap_or_default(),
        };

        if !response.headers.contains_key("content-type") {
            response
                .headers
                .insert("content-type".to_string(), "text/plain".to_string());
        }

        response
    }

    pub fn json<T: Serialize>(
        &self,
        data: T,
        status_code: Option<u16>,
        headers: Option<HashMap<String, String>>,
    ) -> Response {
        let json_string = serde_json::to_string(&data).unwrap_or_else(|_| "{}".to_string());
        let mut response = Response {
            status_code: status_code.unwrap_or(200),
            body: json_string.into_bytes(),
            headers: headers.unwrap_or_default(),
        };

        if !response.headers.contains_key("content-type") {
            response
                .headers
                .insert("content-type".to_string(), "application/json".to_string());
        }

        response
    }

    pub fn binary(
        &self,
        data: Vec<u8>,
        status_code: Option<u16>,
        headers: Option<HashMap<String, String>>,
    ) -> Response {
        let mut response = Response {
            status_code: status_code.unwrap_or(200),
            body: data,
            headers: headers.unwrap_or_default(),
        };

        if !response.headers.contains_key("content-type") {
            response.headers.insert(
                "content-type".to_string(),
                "application/octet-stream".to_string(),
            );
        }

        response
    }

    pub fn empty(&self) -> Response {
        Response {
            status_code: 204,
            body: Vec::new(),
            headers: HashMap::new(),
        }
    }

    pub fn redirect<S: Into<String>>(
        &self,
        url: S,
        status_code: Option<u16>,
        headers: Option<HashMap<String, String>>,
    ) -> Response {
        let url_string = url.into();
        let mut response_headers = headers.unwrap_or_default();
        response_headers.insert("location".to_string(), url_string);

        Response {
            status_code: status_code.unwrap_or(301),
            body: Vec::new(),
            headers: response_headers,
        }
    }

    #[deprecated(note = "Use text(), json(), or binary() instead")]
    pub fn send<S: Into<String>>(&self, data: S) -> Response {
        self.text(data, None, None)
    }
}

impl Default for ContextResponse {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub enum LoggerType {
    Log,
    Error,
}

#[derive(Clone)]
pub struct Logger {
    pub id: String,
    enabled: bool,
    include_native: bool,
    logs: Arc<Mutex<Vec<serde_json::Value>>>,
}

impl Logger {
    pub fn new(logging: &str, log_id: Option<String>) -> Result<Self, String> {
        let enabled = logging == "" || logging == "enabled";
        let include_native = logging == "enabled" || logging == "disabled";

        let id = if let Some(provided_id) = log_id {
            provided_id
        } else if std::env::var("OPEN_RUNTIMES_ENV").unwrap_or_default() == "development" {
            "dev".to_string()
        } else {
            Self::generate_id()
        };

        Ok(Logger {
            id,
            enabled,
            include_native,
            logs: Arc::new(Mutex::new(Vec::new())),
        })
    }

    fn generate_id() -> String {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let sec = now.as_secs();
        let msec = now.subsec_millis();

        let sec_hex = format!("{:x}", sec);
        let msec_hex = format!("{:05x}", msec);

        let mut random_padding = String::new();
        for _ in 0..7 {
            let rand_digit = rand::random::<u8>() % 16;
            random_padding.push_str(&format!("{:x}", rand_digit));
        }

        format!("{}{}{}", sec_hex, msec_hex, random_padding)
    }

    pub fn write(&self, messages: Vec<String>, log_type: LoggerType, native: bool) {
        if !native && !self.enabled {
            return;
        }

        if native && !self.include_native {
            return;
        }

        let type_str = match log_type {
            LoggerType::Log => "log",
            LoggerType::Error => "error",
        };

        let stream = match log_type {
            LoggerType::Log => "stdout",
            LoggerType::Error => "stderr",
        };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let mut message = messages.join(" ");

        if message.len() > MAX_LOG_SIZE {
            let mut safe_len = MAX_LOG_SIZE;
            while safe_len > 0 && !message.is_char_boundary(safe_len) {
                safe_len -= 1;
            }
            message.truncate(safe_len);
            message.push_str("... Log truncated due to size limit (8000 characters)");
        }

        let log_entry = json!({
            "timestamp": timestamp,
            "type": type_str,
            "message": message,
            "stream": stream,
        });

        if let Ok(mut logs) = self.logs.lock() {
            logs.push(log_entry);
        }

        if native {
            let message = messages.join(" ");
            match log_type {
                LoggerType::Log => println!("{}", message),
                LoggerType::Error => eprintln!("{}", message),
            }
        }
    }

    pub fn override_native_logs(&mut self) {
        // In Rust, capturing stdout/stderr is complex and not typically done
        // We'll handle native logs through our write method instead
    }

    pub fn revert_native_logs(&mut self) {
        // Matching the override method
    }

    pub fn end(&self) {
        if !self.enabled {
            return;
        }

        let logs_dir = "/mnt/logs";
        if let Err(_) = create_dir_all(logs_dir) {
            eprintln!("Failed to create logs directory");
            return;
        }

        let logs_file_path = format!("{}/{}_logs.log", logs_dir, self.id);
        let errors_file_path = format!("{}/{}_errors.log", logs_dir, self.id);

        let mut logs_file = match OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&logs_file_path)
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to open logs file: {}", e);
                return;
            }
        };

        let mut errors_file = match OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&errors_file_path)
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to open errors file: {}", e);
                return;
            }
        };

        if let Ok(logs) = self.logs.lock() {
            for log in logs.iter() {
                let log_type = log.get("type").and_then(|v| v.as_str()).unwrap_or("");

                if let Ok(log_str) = serde_json::to_string(log) {
                    let file_to_write = if log_type == "error" {
                        &mut errors_file
                    } else {
                        &mut logs_file
                    };

                    if let Err(e) = writeln!(file_to_write, "{}", log_str) {
                        eprintln!("Failed to write log: {}", e);
                    }
                }
            }
        }

        if let Err(e) = logs_file.flush() {
            eprintln!("Failed to flush logs file: {}", e);
        }

        if let Err(e) = errors_file.flush() {
            eprintln!("Failed to flush errors file: {}", e);
        }
    }
}

pub fn format_log_message(value: &dyn std::fmt::Debug) -> String {
    format!("{:?}", value)
}
