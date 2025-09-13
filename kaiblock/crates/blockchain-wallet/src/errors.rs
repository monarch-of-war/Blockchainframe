#[derive(thiserror::Error, Debug)]
pub enum WalletError{
    #[error(invalid key)]
    InvalidKey,
    #[error("invalid address")]
    InvalidAddress,
    #[error("signing failed")]
    SigningError,
    #[error("serialization error")]
    SerializationError,
}
