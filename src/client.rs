use std::collections::HashMap;
use std::time::Duration;
use crate::HttpResponse;

/// HTTP клиент для отправки запросов
#[derive(Debug)]
pub struct HttpClient {
    headers: HashMap<String, String>,
    cookies: HashMap<String, String>,
    proxy: Option<String>,
    timeout: u64
}

impl HttpClient {
    /// Создание нового HTTP клиента
    pub fn new() -> Self {
        HttpClient {
            headers: HashMap::new(),
            cookies: HashMap::new(),
            proxy: None,
            timeout: 30
        }
    }

    /// Установить заголовки из JSON: {"Header-Name": "value", ...}
    pub fn set_headers(&mut self, headers_json: &str) -> Result<(), String> {
        let headers: HashMap<String, String> = serde_json::from_str(headers_json)
            .map_err(|e| format!("Ошибка парсинга заголовков: {}", e))?;
        self.headers = headers;
        Ok(())
    }

    /// Установить куки из JSON: {"cookie_name": "value", ...}
    pub fn set_cookies(&mut self, cookies_json: &str) -> Result<(), String> {
        let cookies: HashMap<String, String> = serde_json::from_str(cookies_json)
            .map_err(|e| format!("Ошибка парсинга куки: {}", e))?;
        self.cookies = cookies;
        Ok(())
    }

    /// Установить прокси
    pub fn set_proxy(&mut self, proxy_url: &str) {
        if proxy_url.is_empty() {
            self.proxy = None;
        } else {
            self.proxy = Some(proxy_url.to_string());
        }
    }

    pub fn set_timeout(&mut self, timeout: i64) {
        self.timeout = timeout as u64;
    }

    /// GET и DELETE запрос
    pub fn request_without_body(
        &self,
        method_name: &str,
        url: &str,
    ) -> Result<HttpResponse, HttpResponse> {
        let mut request = ureq::request(method_name, &url);

        request = request.timeout(Duration::from_secs(self.timeout));

        for (key, value) in &self.headers {
            request = request.set(key, value);
        }

        let cookie_header = self
            .cookies
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("; ");

        if !cookie_header.is_empty() {
            request = request.set("Cookie", &cookie_header);
        }

        match request.call() {
            Ok(response) => {
                let status = response.status();
                let mut headers = HashMap::new();

                for header_name in response.headers_names() {
                    if let Some(value) = response.header(&header_name) {
                        headers.insert(header_name, value.to_string());
                    }
                }

                let body = response
                    .into_string()
                    .map_err(|e| format!("Ошибка чтения ответа: {}", e))
                    .unwrap_or_default();

                Ok(HttpResponse {
                    status,
                    body,
                    headers,
                    error: None
                })
            }
            Err(e) => {
                let kind = e.kind();
                match e.into_response() {
                    Some(response) => {
                        let status = response.status();
                        let mut headers = HashMap::new();
                        for header_name in response.headers_names() {
                            if let Some(value) = response.header(&header_name) {
                                headers.insert(header_name, value.to_string());
                            }
                        }
                        let body_str = response
                            .into_string()
                            .map_err(|e| format!("Ошибка чтения ответа: {}", e))
                            .unwrap_or_default();
                        Err(HttpResponse {
                            status,
                            body: "{}".to_string(),
                            headers,
                            error: Some(body_str)
                        })
                    }
                    None => {
                        let status = 0;
                        let body_str = kind.to_string();
                        let headers = HashMap::new();
                        Err(HttpResponse {
                            status,
                            body: "{}".to_string(),
                            headers,
                            error: Some(body_str)
                        })
                    }
                }
            }
        }
    }

    /// POST, PUT, PATCH запрос
    pub fn request_with_body(
        &self,
        method_name: &str,
        url: &str,
        body: &str,
    ) -> Result<HttpResponse, HttpResponse> {
        let mut request = ureq::request(method_name, url).set("Content-Type", "application/json");

        request = request.timeout(Duration::from_secs(self.timeout));

        for (key, value) in &self.headers {
            request = request.set(key, value);
        }

        let cookie_header = self
            .cookies
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("; ");

        if !cookie_header.is_empty() {
            request = request.set("Cookie", &cookie_header);
        }

        match request.send_string(body) {
            Ok(response) => {
                let status = response.status();
                let mut headers = HashMap::new();
                for header_name in response.headers_names() {
                    if let Some(value) = response.header(&header_name) {
                        headers.insert(header_name, value.to_string());
                    }
                }

                let body_str = response
                    .into_string()
                    .map_err(|e| format!("Ошибка чтения ответа: {}", e))
                    .unwrap_or_default();

                Ok(HttpResponse {
                    status,
                    body: body_str,
                    headers,
                    error: None
                })
            }
            Err(e) => {
                let kind = e.kind();
                match e.into_response() {
                    Some(response) => {
                        let status = response.status();
                        let mut headers = HashMap::new();
                        for header_name in response.headers_names() {
                            if let Some(value) = response.header(&header_name) {
                                headers.insert(header_name, value.to_string());
                            }
                        }
                        let body_str = response
                            .into_string()
                            .map_err(|e| format!("Ошибка чтения ответа: {}", e))
                            .unwrap_or_default();
                        Err(HttpResponse {
                            status,
                            body: "{}".to_string(),
                            headers,
                            error: Some(body_str)
                        })
                    }
                    None => {
                        let status = 0;
                        let body_str = kind.to_string();
                        let headers = HashMap::new();
                        Err(HttpResponse {
                            status,
                            body: "{}".to_string(),
                            headers,
                            error: Some(body_str)
                        })
                    }
                }
            }
        }
    }
}