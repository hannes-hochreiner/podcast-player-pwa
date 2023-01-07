use std::{error::Error, fmt::Display, num::ParseFloatError};

use serde::{Deserialize, Serialize};

/// TODO: there is a wasm_bindgen::JsError coming up. Once it lands, this class should no longer tbe required.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsError {
    pub description: String,
}

impl JsError {
    pub fn from_str(s: &str) -> Self {
        Self {
            description: s.into(),
        }
    }
}

impl Error for JsError {}

impl Display for JsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.description))
    }
}

impl From<wasm_bindgen::JsValue> for JsError {
    fn from(val: wasm_bindgen::JsValue) -> Self {
        Self {
            description: format!("{:?}", val),
        }
    }
}

impl From<web_sys::DomException> for JsError {
    fn from(val: web_sys::DomException) -> Self {
        Self {
            description: format!("{:?}", val),
        }
    }
}

impl From<serde_wasm_bindgen::Error> for JsError {
    fn from(err: serde_wasm_bindgen::Error) -> Self {
        Self {
            description: err.to_string(),
        }
    }
}

impl From<&str> for JsError {
    fn from(str: &str) -> Self {
        Self {
            description: String::from(str),
        }
    }
}

impl From<url::ParseError> for JsError {
    fn from(e: url::ParseError) -> Self {
        Self {
            description: e.to_string(),
        }
    }
}

impl From<ParseFloatError> for JsError {
    fn from(e: ParseFloatError) -> Self {
        Self {
            description: e.to_string(),
        }
    }
}

impl From<serde_json::Error> for JsError {
    fn from(e: serde_json::Error) -> Self {
        Self {
            description: e.to_string(),
        }
    }
}
