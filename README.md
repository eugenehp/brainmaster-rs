# brainmaster

A Rust library and terminal UI for **BrainMaster Discovery 20/24E** EEG
amplifiers over USB serial (FTDI).

Dual backend: runtime-loaded `bmrcm.dll` FFI on Windows (18/18 functions) +
pure-Rust serial protocol on all OS. Frame decoding, sync detection, impedance
extraction, and conversion factors derived from the
[neuromore studio](https://github.com/neuromore/studio) open-source integration.

## Installation

```shell
cargo add brainmaster
```

## Supported hardware

| Device | Channels | Sampling Rate | Connection | Impedance |
|---|---|---|---|---|
| Discovery 20 | 20 EEG + 4 AUX (24 total) | 256 Hz | USB serial (FTDI, 460800 baud) | ✅ |
| Discovery 24E | 20 EEG + 4 AUX (24 total) | 256 Hz | USB serial (FTDI, 460800 baud) | ✅ |

### Channel layout (10-20 system)

```text
Ch01 Fp1    Ch07 T3     Ch13 P4     Ch19 Pz
Ch02 F3     Ch08 T5     Ch14 O2     Ch20 A2
Ch03 C3     Ch09 Fz     Ch15 F8     Ch21 Fpz
Ch04 P3     Ch10 Fp2    Ch16 T4     Ch22 Oz
Ch05 O1     Ch11 F4     Ch17 T6     Ch23 AUX23
Ch06 F7     Ch12 C4     Ch18 Cz     Ch24 AUX24
```

## Cross-platform

Works on **Windows**, **Linux**, and **macOS**.

| Platform | Backend | Requirements |
|---|---|---|
| **Windows** | `bmrcm.dll` via FFI (18 functions) **or** pure-Rust serial | FTDI drivers + optional `bmrcm.dll` |
| **Linux** | Pure-Rust serial protocol | FTDI drivers (`ftdi_sio` kernel module) |
| **macOS** | Pure-Rust serial protocol | FTDI drivers (built-in or VCP) |

The pure-Rust backend uses the `serialport` crate and requires no DLL.
It implements the complete Discovery serial frame protocol:

### Frame format (78 bytes at 256 Hz)

```text
[0]     Sync byte (cycles: 0x20 → 0x40 → 0x60 → 0x80 → 0xA0 → 0xC0 → 0xE0)
[1-2]   Unused
[3]     Steering (impedance channel selector, 1-28)
[4-5]   Special data (impedance raw, 16-bit signed LE)
[6-77]  24 channels × 3 bytes (24-bit signed LE)
```

### Conversion factors

- **EEG**: `raw_24bit × 0.01658` = µV
- **Impedance**: `raw_16bit × 0.005` = kΩ

### bmrcm.dll API (Windows, 18 functions)

| Function | Purpose |
|---|---|
| `AtlOpenPort` | Open COM port at baud rate |
| `AtlClosePort` | Close COM port |
| `AtlSetBaudRate` | Set baud rate (0x10=460800, 0x20=115200, 0x30=9600) |
| `AtlWriteSamplingRate` | Set device sampling rate |
| `AtlReadSamplingRate` | Read device sampling rate |
| `AtlClearSpecials` | Clear all special selections |
| `AtlFlush` | Flush serial buffers |
| `BmrLoginDevice` | Authenticate with codekey/serial/passkey |
| `DiscStartModule` | Start data acquisition |
| `DiscStopModule` | Stop data acquisition |
| `AtlGetBytesInQue` | Get bytes available in receive buffer |
| `AtlReadData` | Read raw bytes from buffer |
| `AtlSelectImpedanceChans` | Select impedance measurement channels |
| `AtlSelectSpecial` | Select special data mode |
| `AtlSetNotchFilters` | Configure notch filters |
| `AtlPeek` | Read device memory location |
| `AtlPoke` | Write device memory location |
| `AtlQueryFirmware` | Query firmware version |

## Quick start

```rust
use brainmaster::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ports = DiscoveryDevice::find()?;
    let mut device = DiscoveryDevice::open(&ports[0])?;
    device.start()?;

    for _ in 0..SAMPLING_RATE as usize * 4 {
        if let Some(_frame) = device.read_frame()? {
            let ch = device.channels();
            println!("Fp1={:.2}µV O1={:.2}µV O2={:.2}µV",
                ch.data[0], ch.data[4], ch.data[13]);
        }
    }

    device.stop()?;
    device.close();
    Ok(())
}
```

## Project layout

```
brainmaster-rs/
├── Cargo.toml
├── README.md
├── CHANGELOG.md
├── LICENSE
└── src/
    ├── lib.rs            # Crate root + prelude
    ├── main.rs           # CLI binary
    ├── bin/tui.rs        # ratatui TUI (8-channel EEG charts)
    ├── ffi.rs            # Runtime-loaded bmrcm.dll (Windows, 18 functions)
    ├── protocol.rs       # Pure-Rust serial protocol (all OS)
    ├── types.rs          # Frame format, channels, sync, conversion
    ├── device.rs         # High-level device API
    └── error.rs          # Error types
├── examples/
│   ├── scan.rs           # Port discovery
│   ├── read_eeg.rs       # 4-second EEG capture
│   └── read_impedance.rs # Impedance measurement
└── tests/
    └── protocol_tests.rs # Frame decode, sync, conversion tests
```

## Dependencies

| Crate | Purpose |
|---|---|
| [serialport](https://crates.io/crates/serialport) | Cross-platform serial I/O (FTDI USB) |
| [libloading](https://crates.io/crates/libloading) | Runtime DLL loading (Windows, optional) |
| [thiserror](https://crates.io/crates/thiserror) | Error types |
| [log](https://crates.io/crates/log) | Logging facade |
| [env_logger](https://crates.io/crates/env_logger) | Log output |
| [ratatui](https://ratatui.rs) | Terminal UI (optional) |
| [crossterm](https://github.com/crossterm-rs/crossterm) | Terminal backend (optional) |

## Running tests

```bash
cargo test
```

## License

[MIT](./LICENSE)
