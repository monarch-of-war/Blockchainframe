#[derive(thiserror::Error, Debug)]
pub enum NetworkError{
    #[error("Serialization Error: {0}")]
    SerializationError(String),
    #[error("Deserialization Error: {0}")]
    DeserializationError(String),
    #[error("Network IO Error: {0}")]
    IoError(String),
    #[error("Peer Not Found")]
    PeerNotFound,
}