//! High-level device abstraction for BrainMaster Discovery amplifiers.
//!
//! Provides a unified API that uses:
//! - `bmrcm.dll` FFI on Windows (if available)
//! - Pure-Rust serial protocol on all OS (fallback / primary on Linux/macOS)
//!
//! ```rust,ignore
//! use brainmaster::prelude::*;
//!
//! let ports = DiscoveryDevice::find()?;
//! let mut device = DiscoveryDevice::open(&ports[0])?;
//! device.start()?;
//!
//! loop {
//!     if let Some(frame) = device.read_frame()? {
//!         let ch = device.channels();
//!         println!("Fp1={:.2}µV O1={:.2}µV", ch.data[0], ch.data[4]);
//!     }
//! }
//! ```

use crate::error::BrainMasterError;
use crate::protocol::SerialDevice;
use crate::types::*;

/// A BrainMaster Discovery 20/24E EEG amplifier.
pub struct DiscoveryDevice {
    serial: SerialDevice,
}

impl DiscoveryDevice {
    /// Find serial ports that may have a Discovery device.
    pub fn find() -> Result<Vec<String>, BrainMasterError> {
        SerialDevice::find_ports()
    }

    /// Open a connection to a Discovery device on the given serial port.
    pub fn open(port_name: &str) -> Result<Self, BrainMasterError> {
        let serial = SerialDevice::open(port_name)?;
        Ok(DiscoveryDevice { serial })
    }

    /// Open with a specific baud rate.
    pub fn open_with_baud(port_name: &str, baud: u32) -> Result<Self, BrainMasterError> {
        let serial = SerialDevice::open_with_baud(port_name, baud)?;
        Ok(DiscoveryDevice { serial })
    }

    /// Current state.
    pub fn state(&self) -> DeviceState {
        self.serial.state()
    }

    /// Start streaming EEG data.
    pub fn start(&mut self) -> Result<(), BrainMasterError> {
        self.serial.start_streaming()
    }

    /// Stop streaming.
    pub fn stop(&mut self) -> Result<(), BrainMasterError> {
        self.serial.stop_streaming()
    }

    /// Read the next frame. Returns `Some(Frame)` when synced.
    pub fn read_frame(&mut self) -> Result<Option<Frame>, BrainMasterError> {
        self.serial.read_frame()
    }

    /// Get the latest channel values (µV).
    pub fn channels(&self) -> &Channels {
        self.serial.channels()
    }

    /// Get the latest impedance values (kΩ).
    pub fn impedances(&self) -> &Impedances {
        self.serial.impedances()
    }

    /// Whether the stream is synchronized.
    pub fn is_synced(&self) -> bool {
        self.serial.is_synced()
    }

    /// Capture `n` frames of data (blocking convenience method).
    pub fn capture(&mut self, n: usize) -> Result<Vec<Channels>, BrainMasterError> {
        self.serial.capture(n)
    }

    /// Close the connection.
    pub fn close(&mut self) {
        self.serial.close()
    }
}
