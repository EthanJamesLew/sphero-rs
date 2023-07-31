/*!
 * Sphero Error
 */

/// Sphero API Error Codes
#[derive(Debug)]
pub enum Error {
    /// Packet is invalid
    InvalidPacket,
    /// Device ID is invalid (or is invisible with current permissions)
    BadDeviceId,
    /// Command ID is invalid (or is invisible with current permissions)
    BadCommandId,
    /// Command is not yet implemented or has a null handler
    NotImplemented,
    /// Command cannot be executed in the current state or mode
    CommandRestricted,
    /// Payload data length is invalid
    BadDataLength,
    /// Command failed to execute for a command-specific reason
    CommandFailed,
    /// At least one data parameter is invalid
    BadParameterValue,
    /// The operation is already in progress or the module is busy
    Busy,
    /// Target does not exist
    BadTargetId,
    /// Target exists but is unavailable (e.g., it is asleep or disconnected)
    TargetUnavailable,
    /// Currently unused
    Unused(u8),
}

impl From<u8> for Error {
    fn from(code: u8) -> Self {
        match code {
            0x01 => Error::BadDeviceId,
            0x02 => Error::BadCommandId,
            0x03 => Error::NotImplemented,
            0x04 => Error::CommandRestricted,
            0x05 => Error::BadDataLength,
            0x06 => Error::CommandFailed,
            0x07 => Error::BadParameterValue,
            0x08 => Error::Busy,
            0x09 => Error::BadTargetId,
            0x0A => Error::TargetUnavailable,
            _ => Error::Unused(code),
        }
    }
}
