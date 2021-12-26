use std::{error::Error, fmt::Display};

/// TODO: there is a wasm_bindgen::JsError coming up. Once it lands, this class should no longer tbe required.
#[derive(Debug, Clone)]
pub struct JsError {
    pub description: String,
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
            description: val.as_string().unwrap(),
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
