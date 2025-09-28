use serde_json::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReportError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Business error: {0}")]
    Business(#[from] SimpleError),
    #[error("Json error: {0}")]
    Err(#[from] Error)
}

#[derive(Error, Debug)]
#[error("{0}")]
pub struct SimpleError(String);

// From<String> 可以手动实现，或直接用
impl From<String> for SimpleError {
    fn from(msg: String) -> Self {
        SimpleError(msg)
    }
}
