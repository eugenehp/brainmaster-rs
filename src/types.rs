//! Types and constants for the BrainMaster Discovery protocol.
//!
//! Derived from the neuromore studio open-source integration
//! (`Discovery20.h`) and the bmrcm.dll API.

use std::fmt;

// ── Constants ────────────────────────────────────────────────────────────────

/// Number of EEG channels (20 standard + 4 auxiliary).
pub const NUM_CHANNELS: usize = 24;

/// Number of standard 10-20 EEG channels.
pub const NUM_EEG_CHANNELS: usize = 20;

/// Device sampling rate (Hz).
pub const SAMPLING_RATE: u32 = 256;

/// Serial frame size in bytes.
pub const FRAME_SIZE: usize = 78;

/// EEG value conversion factor: raw_24bit × CONVERT_UV = µV.
pub const CONVERT_UV: f32 = 0.01658;

/// Impedance conversion factor: raw_16bit × CONVERT_KOHM = kΩ.
pub const CONVERT_KOHM: f32 = 0.005;

/// Minimum sync iterations before frames are considered valid.
pub const MIN_SYNCS: u32 = 4;

/// Data timeout in milliseconds.
pub const DATA_TIMEOUT_MS: u64 = 3000;

/// Default serial baud rate for streaming.
pub const BAUD_RATE: u32 = 460800;

/// Initial serial baud rate for connection.
pub const INITIAL_BAUD_RATE: u32 = 9600;

/// Serial port buffer size in bytes.
pub const SERIAL_BUFFER_SIZE: u32 = 1024 * 1024;

// ── Baud rate codes (bmrcm.dll) ──────────────────────────────────────────────

/// Baud rate code for 460800.
pub const BR_460800: i32 = 0x10;
/// Baud rate code for 115200.
pub const BR_115200: i32 = 0x20;
/// Baud rate code for 9600.
pub const BR_9600: i32 = 0x30;

// ── Login codes ──────────────────────────────────────────────────────────────

/// Login result: early 2E firmware.
pub const LOGIN_EARLY2E: i32 = 11;
/// Login result: ready 2E firmware.
pub const LOGIN_READY2E: i32 = 12;
/// Login result: wideband 2E firmware.
pub const LOGIN_WIDEB2E: i32 = 13;

// ── Channel names ────────────────────────────────────────────────────────────

/// All 24 channel names in order (10-20 system + auxiliary).
pub const CHANNEL_NAMES: [&str; NUM_CHANNELS] = [
    "Fp1", "F3",  "C3",  "P3",    // ch01-04
    "O1",  "F7",  "T3",  "T5",    // ch05-08
    "Fz",  "Fp2", "F4",  "C4",    // ch09-12
    "P4",  "O2",  "F8",  "T4",    // ch13-16
    "T6",  "Cz",  "Pz",  "A2",    // ch17-20
    "Fpz", "Oz",  "AUX23", "AUX24", // ch21-24
];

/// Standard 10-20 EEG channel names (first 20).
pub const EEG_CHANNEL_NAMES: [&str; NUM_EEG_CHANNELS] = [
    "Fp1", "F3",  "C3",  "P3",
    "O1",  "F7",  "T3",  "T5",
    "Fz",  "Fp2", "F4",  "C4",
    "P4",  "O2",  "F8",  "T4",
    "T6",  "Cz",  "Pz",  "A2",
];

// ── Sync bytes ───────────────────────────────────────────────────────────────

/// Valid sync bytes cycle: 0x20 → 0x40 → 0x60 → 0x80 → 0xA0 → 0xC0 → 0xE0 → 0x20.
pub const SYNC_BYTES: [u8; 7] = [0x20, 0x40, 0x60, 0x80, 0xA0, 0xC0, 0xE0];

/// Get the next expected sync byte.
///
/// Only valid for inputs that are in `SYNC_BYTES`. For other inputs,
/// the result is meaningless (but won't panic).
pub fn next_sync(s: u8) -> u8 {
    if s == 0xE0 { 0x20 } else { s.wrapping_add(0x20) }
}

/// Check if two bytes form a valid consecutive sync pair.
pub fn is_sync_pair(a: u8, b: u8) -> bool {
    SYNC_BYTES.contains(&a) && next_sync(a) == b
}

// ── 24-bit signed integer ────────────────────────────────────────────────────

/// Decode a 24-bit signed little-endian value from 3 bytes.
///
/// The Discovery frame stores each channel as 3 bytes: `[low16_lo, low16_hi, high8]`.
/// This is sign-extended to 32-bit.
pub fn decode_int24(bytes: &[u8; 3]) -> i32 {
    let raw = (bytes[0] as u32) | ((bytes[1] as u32) << 8) | ((bytes[2] as u32) << 16);
    // Sign extend from 24-bit
    let m1: u32 = 1 << 23;
    let m2: u32 = (1 << 24) - 1;
    ((raw & m2) ^ m1).wrapping_sub(m1) as i32
}

// ── Frame ────────────────────────────────────────────────────────────────────

/// A decoded Discovery 20 data frame (78 bytes → 24 channels).
///
/// ```text
/// Byte  Field
/// [0]   sync byte (0x20/0x40/0x60/0x80/0xA0/0xC0/0xE0)
/// [1]   unused
/// [2]   unused
/// [3]   steering (impedance channel selector)
/// [4-5] special data (impedance value, 16-bit signed LE)
/// [6-77] 24 channels × 3 bytes (24-bit signed LE)
/// ```
#[derive(Clone)]
pub struct Frame {
    /// Raw 78-byte frame data.
    pub raw: [u8; FRAME_SIZE],
}

impl Frame {
    /// The sync byte.
    pub fn sync(&self) -> u8 {
        self.raw[0]
    }

    /// Steering byte (impedance channel selector, 1-28).
    pub fn steering(&self) -> u8 {
        self.raw[3]
    }

    /// Special data (impedance raw value, 16-bit signed LE).
    pub fn special_data(&self) -> i16 {
        i16::from_le_bytes([self.raw[4], self.raw[5]])
    }

    /// Decode all 24 channels to raw i32 values.
    pub fn raw_channels(&self) -> [i32; NUM_CHANNELS] {
        let mut out = [0i32; NUM_CHANNELS];
        for i in 0..NUM_CHANNELS {
            let offset = 6 + i * 3;
            out[i] = decode_int24(&[self.raw[offset], self.raw[offset + 1], self.raw[offset + 2]]);
        }
        out
    }

    /// Decode all 24 channels to µV.
    pub fn channels_uv(&self) -> [f32; NUM_CHANNELS] {
        let raw = self.raw_channels();
        let mut out = [0.0f32; NUM_CHANNELS];
        for i in 0..NUM_CHANNELS {
            out[i] = raw[i] as f32 * CONVERT_UV;
        }
        out
    }
}

impl fmt::Debug for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ch = self.channels_uv();
        f.debug_struct("Frame")
            .field("sync", &format_args!("0x{:02X}", self.sync()))
            .field("steering", &self.steering())
            .field("Fp1", &format_args!("{:.2}µV", ch[0]))
            .field("O1", &format_args!("{:.2}µV", ch[4]))
            .field("O2", &format_args!("{:.2}µV", ch[13]))
            .finish()
    }
}

// ── Channel values ───────────────────────────────────────────────────────────

/// Decoded channel values in µV.
#[derive(Debug, Clone, Copy)]
pub struct Channels {
    /// All 24 channel values in µV.
    pub data: [f32; NUM_CHANNELS],
}

impl Channels {
    /// Create zeroed channels.
    pub fn zero() -> Self {
        Channels { data: [0.0; NUM_CHANNELS] }
    }

    /// Get a channel by name.
    pub fn get(&self, name: &str) -> Option<f32> {
        CHANNEL_NAMES.iter().position(|&n| n == name).map(|i| self.data[i])
    }

    /// Standard 10-20 EEG channels (first 20).
    pub fn eeg(&self) -> &[f32] {
        &self.data[..NUM_EEG_CHANNELS]
    }
}

/// Impedance values in kΩ.
#[derive(Debug, Clone, Copy)]
pub struct Impedances {
    /// Active (electrode) impedances in kΩ.
    pub active: [f32; NUM_CHANNELS],
    /// Reference impedances in kΩ.
    pub reference: [f32; NUM_CHANNELS],
}

impl Impedances {
    /// Create zeroed impedances.
    pub fn zero() -> Self {
        Impedances {
            active: [0.0; NUM_CHANNELS],
            reference: [0.0; NUM_CHANNELS],
        }
    }

    /// Update impedances from a frame's steering/special data.
    pub fn update_from_frame(&mut self, frame: &Frame) {
        let steering = frame.steering() as usize;
        if steering == 0 || steering > 28 {
            return;
        }
        let value = frame.special_data() as f32 * CONVERT_KOHM;
        match steering {
            23 => self.active[22] = value,  // AUX23 active
            24 => self.active[23] = value,  // AUX24 active
            25 => self.reference[22] = value, // AUX23 ref
            26 => self.reference[23] = value, // AUX24 ref
            27 => {}                         // unused
            28 => {                          // A1 ref for ch 1-22
                for j in 0..22 {
                    self.reference[j] = value;
                }
            }
            _ => {                           // ch 1-22 active
                self.active[steering - 1] = value;
            }
        }
    }
}

// ── Connection state ─────────────────────────────────────────────────────────

/// Device connection/streaming state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceState {
    /// Not connected.
    Disconnected,
    /// Connected but not streaming.
    Connected,
    /// Streaming but not yet synchronized.
    Unsynced,
    /// Synchronizing (counting sync iterations).
    Syncing,
    /// Fully synchronized and receiving valid frames.
    Synced,
}
