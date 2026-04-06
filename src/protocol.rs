//! Pure-Rust serial protocol for BrainMaster Discovery 20/24E.
//!
//! Works on **all OS** (Windows, Linux, macOS) via the `serialport` crate.
//! No bmrcm.dll required.
//!
//! # Protocol
//!
//! The Discovery communicates over FTDI USB-serial at 460800 baud.
//! Data frames are 78 bytes, streamed at 256 Hz.
//!
//! ## Frame format (78 bytes)
//!
//! ```text
//! [0]    sync byte (cycles: 0x20 → 0x40 → 0x60 → 0x80 → 0xA0 → 0xC0 → 0xE0)
//! [1]    unused
//! [2]    unused
//! [3]    steering (impedance channel, 1-28)
//! [4-5]  special data (impedance raw, 16-bit signed LE)
//! [6-77] 24 channels × 3 bytes (24-bit signed LE)
//! ```
//!
//! ## Conversion
//!
//! - EEG: `raw_24bit × 0.01658` = µV
//! - Impedance: `raw_16bit × 0.005` = kΩ

use std::io::{Read, Write};
use std::time::{Duration, Instant};

use crate::error::BrainMasterError;
use crate::types::*;

/// A serial connection to a BrainMaster Discovery device.
pub struct SerialDevice {
    port: Box<dyn serialport::SerialPort>,
    state: DeviceState,
    next_sync: u8,
    num_syncs: u32,
    channels: Channels,
    impedances: Impedances,
}

impl SerialDevice {
    /// Find available serial ports that might be a Discovery device.
    ///
    /// Looks for FTDI-based ports with "Discovery" in the description.
    pub fn find_ports() -> Result<Vec<String>, BrainMasterError> {
        let ports = serialport::available_ports()?;
        let mut result = Vec::new();
        for p in &ports {
            let dominated = match &p.port_type {
                serialport::SerialPortType::UsbPort(info) => {
                    // FTDI VID=0x0403
                    info.vid == 0x0403
                        || info
                            .product
                            .as_ref()
                            .map(|s| s.contains("Discovery") || s.contains("FTDI"))
                            .unwrap_or(false)
                }
                _ => false,
            };
            if dominated {
                result.push(p.port_name.clone());
            }
        }
        // If no FTDI ports found, return all ports as candidates
        if result.is_empty() {
            result = ports.iter().map(|p| p.port_name.clone()).collect();
        }
        Ok(result)
    }

    /// Open a serial connection to a Discovery device.
    ///
    /// This opens at `INITIAL_BAUD_RATE` (9600), then switches to
    /// `BAUD_RATE` (460800) for streaming.
    pub fn open(port_name: &str) -> Result<Self, BrainMasterError> {
        let port = serialport::new(port_name, BAUD_RATE)
            .timeout(Duration::from_millis(DATA_TIMEOUT_MS))
            .open()?;

        Ok(SerialDevice {
            port,
            state: DeviceState::Connected,
            next_sync: 0,
            num_syncs: 0,
            channels: Channels::zero(),
            impedances: Impedances::zero(),
        })
    }

    /// Open at a specific baud rate.
    pub fn open_with_baud(port_name: &str, baud: u32) -> Result<Self, BrainMasterError> {
        let port = serialport::new(port_name, baud)
            .timeout(Duration::from_millis(DATA_TIMEOUT_MS))
            .open()?;

        Ok(SerialDevice {
            port,
            state: DeviceState::Connected,
            next_sync: 0,
            num_syncs: 0,
            channels: Channels::zero(),
            impedances: Impedances::zero(),
        })
    }

    /// Current device state.
    pub fn state(&self) -> DeviceState {
        self.state
    }

    /// Whether the device is streaming and synced.
    pub fn is_synced(&self) -> bool {
        self.state == DeviceState::Synced
    }

    /// Get the latest decoded channel values (µV).
    pub fn channels(&self) -> &Channels {
        &self.channels
    }

    /// Get the latest impedance values (kΩ).
    pub fn impedances(&self) -> &Impedances {
        &self.impedances
    }

    /// Start streaming. Transitions to `Unsynced` state.
    pub fn start_streaming(&mut self) -> Result<(), BrainMasterError> {
        // Flush any stale data
        let _ = self.port.clear(serialport::ClearBuffer::All);
        self.state = DeviceState::Unsynced;
        self.num_syncs = 0;
        self.next_sync = 0;
        Ok(())
    }

    /// Stop streaming.
    pub fn stop_streaming(&mut self) -> Result<(), BrainMasterError> {
        self.state = DeviceState::Connected;
        Ok(())
    }

    /// Read and process the next frame from the serial port.
    ///
    /// Returns `Some(frame)` if a valid synced frame was decoded,
    /// `None` if still syncing or no data available.
    pub fn read_frame(&mut self) -> Result<Option<Frame>, BrainMasterError> {
        match self.state {
            DeviceState::Disconnected => Err(BrainMasterError::NotConnected),
            DeviceState::Connected => Ok(None),
            DeviceState::Unsynced => {
                self.try_sync()?;
                Ok(None)
            }
            DeviceState::Syncing => {
                self.try_sync()?;
                Ok(None)
            }
            DeviceState::Synced => {
                self.read_synced_frame()
            }
        }
    }

    /// Attempt to synchronize with the data stream.
    fn try_sync(&mut self) -> Result<(), BrainMasterError> {
        // Read two frame's worth of data to find sync pair
        let mut buf = [0u8; FRAME_SIZE * 2];
        self.read_exact(&mut buf)?;

        for i in 0..FRAME_SIZE {
            let c1 = buf[i];
            let c2 = buf[i + FRAME_SIZE];
            if is_sync_pair(c1, c2) {
                self.next_sync = c2;
                self.num_syncs = 0;
                self.state = DeviceState::Syncing;
                log::debug!("Sync start: 0x{:02X} → 0x{:02X}", c1, c2);

                // Now read frames and count successful syncs
                return self.count_syncs();
            }
        }

        Ok(())
    }

    /// Count consecutive sync-valid frames until MIN_SYNCS.
    fn count_syncs(&mut self) -> Result<(), BrainMasterError> {
        while self.num_syncs < MIN_SYNCS {
            let mut frame_buf = [0u8; FRAME_SIZE];
            self.read_exact(&mut frame_buf)?;

            if frame_buf[0] != self.next_sync {
                log::warn!("Sync fail: expected 0x{:02X}, got 0x{:02X}", self.next_sync, frame_buf[0]);
                self.state = DeviceState::Unsynced;
                self.num_syncs = 0;
                return Ok(());
            }

            self.num_syncs += 1;
            self.next_sync = next_sync(self.next_sync);
        }

        log::info!("Sync achieved after {} iterations", self.num_syncs);
        self.state = DeviceState::Synced;
        Ok(())
    }

    /// Read a single synced frame.
    fn read_synced_frame(&mut self) -> Result<Option<Frame>, BrainMasterError> {
        let mut raw = [0u8; FRAME_SIZE];
        self.read_exact(&mut raw)?;

        if raw[0] != self.next_sync {
            log::warn!("Sync lost: expected 0x{:02X}, got 0x{:02X}", self.next_sync, raw[0]);
            self.state = DeviceState::Unsynced;
            self.num_syncs = 0;
            return Err(BrainMasterError::SyncLost);
        }

        self.next_sync = next_sync(self.next_sync);

        let frame = Frame { raw };

        // Decode channels
        self.channels.data = frame.channels_uv();

        // Update impedances
        self.impedances.update_from_frame(&frame);

        Ok(Some(frame))
    }

    /// Read exact number of bytes with timeout.
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), BrainMasterError> {
        let start = Instant::now();
        let mut pos = 0;
        while pos < buf.len() {
            if start.elapsed() > Duration::from_millis(DATA_TIMEOUT_MS) {
                return Err(BrainMasterError::Timeout);
            }
            match self.port.read(&mut buf[pos..]) {
                Ok(n) => pos += n,
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
                Err(e) => return Err(BrainMasterError::SerialPort(e.to_string())),
            }
        }
        Ok(())
    }

    /// Write raw bytes to the serial port.
    pub fn write_raw(&mut self, data: &[u8]) -> Result<(), BrainMasterError> {
        self.port.write_all(data).map_err(|e| BrainMasterError::SerialPort(e.to_string()))
    }

    /// Close the serial connection.
    pub fn close(&mut self) {
        self.state = DeviceState::Disconnected;
    }

    /// Set the baud rate on the serial port.
    pub fn set_baud_rate(&mut self, baud: u32) -> Result<(), BrainMasterError> {
        self.port.set_baud_rate(baud).map_err(|e| BrainMasterError::SerialPort(e.to_string()))
    }

    /// Capture `n` frames of EEG data (blocking).
    pub fn capture(&mut self, n: usize) -> Result<Vec<Channels>, BrainMasterError> {
        self.start_streaming()?;

        let mut result = Vec::with_capacity(n);
        let mut attempts = 0;
        let max_attempts = n * 10; // allow sync overhead

        while result.len() < n && attempts < max_attempts {
            attempts += 1;
            match self.read_frame()? {
                Some(_) => result.push(self.channels),
                None => continue,
            }
        }

        self.stop_streaming()?;
        Ok(result)
    }
}
