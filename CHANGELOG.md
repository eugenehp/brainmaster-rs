# Changelog

## [0.0.1] - 2026-04-06

### Added
- Initial release
- Runtime-loaded FFI bindings to bmrcm.dll (Windows, 18/18 functions)
- Pure-Rust serial protocol for all OS (no DLL needed)
- 78-byte frame decode with 24-bit signed LE channels
- 24 channel support (20 EEG + 4 AUX) at 256 Hz
- Impedance extraction from steering/special data
- Sync byte detection and validation
- CLI binary + ratatui TUI with 8-channel EEG charts
- Examples: scan, read_eeg, read_impedance
- 26 unit tests (no hardware required)
- Cross-platform: Windows, Linux, macOS
