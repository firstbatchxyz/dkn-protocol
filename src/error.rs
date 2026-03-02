#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("serialize: {0}")]
    Serialize(String),
    #[error("deserialize: {0}")]
    Deserialize(String),
    #[error("network: {0}")]
    Network(String),
}
