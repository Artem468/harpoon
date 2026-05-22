mod client;
mod response;

use crate::client::HttpClient;
use crate::response::HttpResponse;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub unsafe extern "C" fn http_client_new() -> *mut HttpClient {
    Box::into_raw(Box::new(HttpClient::new()))
}

#[no_mangle]
pub unsafe extern "C" fn http_client_set_headers(
    client: *mut HttpClient,
    headers_json: *const c_char,
) -> *const c_char {
    let client = unsafe { &mut *client };
    let headers_json = unsafe { CStr::from_ptr(headers_json).to_string_lossy().into_owned() };

    match client.set_headers(&headers_json) {
        Ok(_) => {
            let c_string = CString::new("OK").unwrap();
            Box::into_raw(c_string.into_boxed_c_str()) as *const c_char
        }
        Err(e) => {
            let c_string = CString::new(e).unwrap();
            Box::into_raw(c_string.into_boxed_c_str()) as *const c_char
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn http_client_set_cookies(
    client: *mut HttpClient,
    cookies_json: *const c_char,
) -> *const c_char {
    let client = unsafe { &mut *client };
    let cookies_json = unsafe { CStr::from_ptr(cookies_json).to_string_lossy().into_owned() };

    match client.set_cookies(&cookies_json) {
        Ok(_) => {
            let c_string = CString::new("OK").unwrap();
            Box::into_raw(c_string.into_boxed_c_str()) as *const c_char
        }
        Err(e) => {
            let c_string = CString::new(e).unwrap();
            Box::into_raw(c_string.into_boxed_c_str()) as *const c_char
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn http_client_set_proxy(client: *mut HttpClient, proxy_url: *const c_char) {
    let client = unsafe { &mut *client };
    let proxy_url = unsafe { CStr::from_ptr(proxy_url).to_string_lossy().into_owned() };
    client.set_proxy(&proxy_url);
}

#[no_mangle]
pub unsafe extern "C" fn http_client_set_timeout(client: *mut HttpClient, timeout: i64) {
    let client = unsafe { &mut *client };
    client.set_timeout(timeout);
}

macro_rules! gen_http_method_without_body {
    ($($fn_name:ident => $method_str:expr),*) => {
        $(
            #[no_mangle]
            pub unsafe extern "C" fn $fn_name(
                client: *mut HttpClient,
                url: *const c_char,
            ) -> *mut HttpResponse {
                let client = unsafe { &*client };
                let _url = unsafe { CStr::from_ptr(url).to_string_lossy().into_owned() };
                match client.request_without_body($method_str, &_url) {
                    Ok(response) => Box::into_raw(Box::new(response)),
                    Err(e) => Box::into_raw(Box::new(e)),
                }
            }
        )*
    };
}

gen_http_method_without_body!(
    http_client_get    => "GET",
    http_client_delete => "DELETE"
);

macro_rules! gen_http_method_with_body {
    ($($fn_name:ident => $method_str:expr),*) => {
        $(
            #[no_mangle]
            pub unsafe extern "C" fn $fn_name(
                client: *mut HttpClient,
                url: *const c_char,
                body: *const c_char,
            ) -> *mut HttpResponse {
                let client = unsafe { &*client };
                let url = unsafe {
                    CStr::from_ptr(url)
                        .to_string_lossy()
                        .into_owned()
                };
                let body = unsafe {
                    CStr::from_ptr(body)
                        .to_string_lossy()
                        .into_owned()
                };

                match client.request_with_body($method_str, &url, &body) {
                    Ok(response) => Box::into_raw(Box::new(response)),
                    Err(e) => {
                        Box::into_raw(Box::new(e))
                    }
                }
            }
        )*
    };
}

gen_http_method_with_body!(
    http_client_post   => "POST",
    http_client_put    => "PUT",
    http_client_patch  => "PATCH"
);

#[no_mangle]
pub unsafe extern "C" fn http_response_get_status(response: *const HttpResponse) -> i64 {
    let response = unsafe { &*response };
    response.status as i64
}

#[no_mangle]
pub unsafe extern "C" fn http_response_get_body(response: *const HttpResponse) -> *const c_char {
    let response = unsafe { &*response };
    let c_string = CString::new(&response.body[..]).unwrap();
    Box::into_raw(c_string.into_boxed_c_str()) as *const c_char
}

#[no_mangle]
pub unsafe extern "C" fn http_response_get_headers(response: *const HttpResponse) -> *const c_char {
    let response = unsafe { &*response };
    let headers_json =
        serde_json::to_string(&response.headers).unwrap_or_else(|_| "{}".to_string());
    let c_string = CString::new(headers_json).unwrap();
    Box::into_raw(c_string.into_boxed_c_str()) as *const c_char
}

#[no_mangle]
pub unsafe extern "C" fn http_response_get_error(response: *const HttpResponse) -> *const c_char {
    let response = unsafe { &*response };
    let error = response.error.clone().unwrap_or_default();
    let c_string = CString::new(error).unwrap();
    Box::into_raw(c_string.into_boxed_c_str()) as *const c_char
}

#[no_mangle]
pub unsafe extern "C" fn http_response_get_header(
    response: *const HttpResponse,
    header_name: *const c_char,
) -> *const c_char {
    let response = unsafe { &*response };
    let header_name = unsafe { CStr::from_ptr(header_name).to_string_lossy().into_owned() };

    match response.headers.get(&header_name) {
        Some(value) => {
            let c_string = CString::new(value.clone()).unwrap();
            Box::into_raw(c_string.into_boxed_c_str()) as *const c_char
        }
        None => {
            let c_string = CString::new("").unwrap();
            Box::into_raw(c_string.into_boxed_c_str()) as *const c_char
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn http_response_get_cookies(response: *const HttpResponse) -> *const c_char {
    let response = unsafe { &*response };
    let mut cookies: HashMap<String, String> = HashMap::new();

    if let Some(set_cookie) = response.headers.get("set-cookie") {
        for cookie_str in set_cookie.split(',') {
            if let Some(cookie_part) = cookie_str.split(';').next() {
                let cookie_part = cookie_part.trim();
                if let Some(eq_pos) = cookie_part.find('=') {
                    let key = cookie_part[..eq_pos].to_string();
                    let value = cookie_part[eq_pos + 1..].to_string();
                    cookies.insert(key, value);
                }
            }
        }
    }

    let cookies_json = serde_json::to_string(&cookies).unwrap_or_else(|_| "{}".to_string());
    let c_string = CString::new(cookies_json).unwrap();
    Box::into_raw(c_string.into_boxed_c_str()) as *const c_char
}

#[no_mangle]
pub unsafe extern "C" fn http_response_free(response: *mut HttpResponse) {
    if !response.is_null() {
        unsafe {
            let _ = Box::from_raw(response);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn http_client_free(client: *mut HttpClient) {
    if !client.is_null() {
        unsafe {
            let _ = Box::from_raw(client);
        }
    }
}
