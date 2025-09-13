#[derive(thiserror::Error, Debug)]
pub enum RpcError {
    #[error("Block not found")]
    BlockNotFound,
    #[error("transaction not found")]
    TransactionNotFound,
    #[error("Internal server error")]
    InternalServerError,
}