use serde::{Deserialize, Serialize};
use ts_rs::TS;
pub type AppResult<T> = Result<T, AppError>;
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct FieldError {
    pub field: String,
    pub code: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: String,
    pub fields: Vec<FieldError>,
    pub context: Option<String>,
}
impl AppError {
    pub fn new(code: &str) -> Self {
        Self {
            code: code.into(),
            fields: vec![],
            context: None,
        }
    }
    pub fn field(field: &str, code: &str) -> Self {
        Self {
            code: "validation".into(),
            fields: vec![FieldError {
                field: field.into(),
                code: code.into(),
            }],
            context: None,
        }
    }
}
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code)
    }
}
impl std::error::Error for AppError {}
impl From<rusqlite::Error> for AppError {
    fn from(_: rusqlite::Error) -> Self {
        Self::new("storage")
    }
}
impl From<serde_json::Error> for AppError {
    fn from(_: serde_json::Error) -> Self {
        Self::new("invalidData")
    }
}
impl From<std::io::Error> for AppError {
    fn from(_: std::io::Error) -> Self {
        Self::new("fileAccess")
    }
}
