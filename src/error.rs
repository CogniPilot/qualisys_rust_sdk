use thiserror::Error;

use crate::protocol::PacketType;

pub type Result<T, E = QtmError> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum QtmError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("utf-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("xml parse error: {0}")]
    Xml(#[from] roxmltree::Error),
    #[error("invalid packet: {0}")]
    InvalidPacket(String),
    #[error("invalid QTM parameters XML: {0}")]
    InvalidParametersXml(String),
    #[error("unexpected packet type: expected {expected:?}, got {actual:?}")]
    UnexpectedPacketType {
        expected: PacketType,
        actual: PacketType,
    },
    #[error("unsupported packet type value {0}")]
    UnsupportedPacketType(u32),
    #[error("unsupported event value {0}")]
    UnsupportedEvent(u8),
    #[error("qtm returned error: {0}")]
    CommandFailed(String),
    #[error("timeout waiting for response")]
    Timeout,
    #[error("UDP stream is not active")]
    UdpStreamNotActive,
}

impl QtmError {
    pub fn invalid_packet(message: impl Into<String>) -> Self {
        Self::InvalidPacket(message.into())
    }

    pub fn invalid_parameters_xml(message: impl Into<String>) -> Self {
        Self::InvalidParametersXml(message.into())
    }
}
