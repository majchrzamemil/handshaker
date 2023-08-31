use crate::error::Error;

pub mod message;
pub mod verack;
pub mod version;

/// A trait for types that can be converted into a network message payload.
pub trait ToNetworkMessage {
    /// Converts the implementing type into a network message payload.
    ///
    /// Returns a `Result` containing the binary representation of the message payload
    /// if the conversion is successful. If an error occurs during the conversion,
    /// an `Error` is returned.
    fn to_network_message(self) -> Result<Vec<u8>, Error>;
}
