//! Error types for the BrainMaster API wrapper.

/// All errors that can occur within the BrainMaster API.
#[derive(Debug, thiserror::Error)]
pub enum BrainMasterError {
    /// The bmrcm.dll could not be loaded (Windows only).
    #[error("bmrcm.dll not available: {reason}")]
    LibraryNotAvailable { reason: String },

    /// Serial port error.
    #[error("Serial port error: {0}")]
    SerialPort(String),

    /// No device found.
    #[error("No BrainMaster device found")]
    NoDeviceFound,

    /// Connection failed.
    #[error("Connect failed (code {code}): {message}")]
    ConnectFailed { code: i32, message: String },

    /// Device not connected.
    #[error("Device not connected")]
    NotConnected,

    /// Sync lost during streaming.
    #[error("Sync lost")]
    SyncLost,

    /// Timeout waiting for data.
    #[error("Timeout")]
    Timeout,

    /// Login credentials rejected.
    #[error("Login failed: invalid credentials")]
    LoginFailed,

    /// Not supported on this platform.
    #[error("Not supported: {0}")]
    NotSupported(String),
}

impl From<serialport::Error> for BrainMasterError {
    fn from(e: serialport::Error) -> Self {
        BrainMasterError::SerialPort(e.to_string())
    }
}
