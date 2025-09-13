use thiserror::Error;

#[derive(Debug, Error)]
pub enum MempoolError {
    #[error("transaction already exists in pool")]
    Duplicate,
    #[error("transaction invalid: {0}")]
    Invalid(String),
    #[error("transaction not found")]
    NotFound,
}
