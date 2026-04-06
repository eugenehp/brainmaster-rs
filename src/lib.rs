//! # brainmaster
//!
//! Rust library and terminal UI for **BrainMaster Discovery 20/24E**
//! EEG amplifiers.
//!
//! ## Cross-platform
//!
//! Works on **Windows**, **Linux**, and **macOS** via pure-Rust serial
//! protocol over FTDI USB. On Windows, the native `bmrcm.dll` is also
//! supported via runtime FFI (18/18 functions).
//!
//! ## Quick start
//!
//! ```rust,ignore
//! use brainmaster::prelude::*;
//!
//! let ports = DiscoveryDevice::find()?;
//! let mut device = DiscoveryDevice::open(&ports[0])?;
//! device.start()?;
//!
//! for _ in 0..SAMPLING_RATE as usize * 4 {
//!     if let Some(frame) = device.read_frame()? {
//!         let ch = device.channels();
//!         println!("Fp1={:.2}µV O1={:.2}µV", ch.data[0], ch.data[4]);
//!     }
//! }
//!
//! device.stop()?;
//! ```
//!
//! ## Module overview
//!
//! | Module | Purpose |
//! |---|---|
//! | [`ffi`] | Runtime-loaded `bmrcm.dll` FFI (Windows, 18/18 functions) |
//! | [`protocol`] | Pure-Rust serial protocol (all OS, no DLL needed) |
//! | [`types`] | Frame format, channel names, conversion factors |
//! | [`device`] | High-level device API |
//! | [`error`] | Error types |

pub mod ffi;
pub mod protocol;
pub mod types;
pub mod device;
pub mod error;

/// Convenience re-exports.
pub mod prelude {
    pub use crate::error::BrainMasterError;
    pub use crate::types::*;
    pub use crate::device::DiscoveryDevice;
    pub use crate::protocol::SerialDevice;
}
