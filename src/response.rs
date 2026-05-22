use std::collections::HashMap;

/// Структура ответа HTTP
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
    pub headers: HashMap<String, String>,
    pub error: Option<String>
}