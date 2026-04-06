//! Unit tests for frame decoding, sync, and conversion (no hardware required).

use brainmaster::types::*;

// ── Constants ────────────────────────────────────────────────────────────────

#[test] fn test_num_channels() { assert_eq!(NUM_CHANNELS, 24); }
#[test] fn test_num_eeg_channels() { assert_eq!(NUM_EEG_CHANNELS, 20); }
#[test] fn test_sampling_rate() { assert_eq!(SAMPLING_RATE, 256); }
#[test] fn test_frame_size() { assert_eq!(FRAME_SIZE, 78); }
#[test] fn test_convert_uv() { assert!((CONVERT_UV - 0.01658).abs() < 0.0001); }
#[test] fn test_convert_kohm() { assert!((CONVERT_KOHM - 0.005).abs() < 0.0001); }

// ── Channel names ────────────────────────────────────────────────────────────

#[test]
fn test_channel_names() {
    assert_eq!(CHANNEL_NAMES.len(), 24);
    assert_eq!(CHANNEL_NAMES[0], "Fp1");
    assert_eq!(CHANNEL_NAMES[4], "O1");
    assert_eq!(CHANNEL_NAMES[13], "O2");
    assert_eq!(CHANNEL_NAMES[19], "A2");
    assert_eq!(CHANNEL_NAMES[20], "Fpz");
    assert_eq!(CHANNEL_NAMES[21], "Oz");
    assert_eq!(CHANNEL_NAMES[22], "AUX23");
    assert_eq!(CHANNEL_NAMES[23], "AUX24");
}

#[test]
fn test_eeg_channel_names() {
    assert_eq!(EEG_CHANNEL_NAMES.len(), 20);
    assert_eq!(EEG_CHANNEL_NAMES[0], "Fp1");
    assert_eq!(EEG_CHANNEL_NAMES[19], "A2");
}

// ── Sync bytes ───────────────────────────────────────────────────────────────

#[test]
fn test_sync_bytes() {
    assert_eq!(SYNC_BYTES, [0x20, 0x40, 0x60, 0x80, 0xA0, 0xC0, 0xE0]);
}

#[test]
fn test_next_sync() {
    assert_eq!(next_sync(0x20), 0x40);
    assert_eq!(next_sync(0x40), 0x60);
    assert_eq!(next_sync(0x60), 0x80);
    assert_eq!(next_sync(0x80), 0xA0);
    assert_eq!(next_sync(0xA0), 0xC0);
    assert_eq!(next_sync(0xC0), 0xE0);
    assert_eq!(next_sync(0xE0), 0x20); // wrap
}

#[test]
fn test_is_sync_pair() {
    assert!(is_sync_pair(0x20, 0x40));
    assert!(is_sync_pair(0xC0, 0xE0));
    assert!(is_sync_pair(0xE0, 0x20)); // wrap
    assert!(!is_sync_pair(0x20, 0x60)); // skip
    assert!(!is_sync_pair(0x00, 0x20)); // invalid start
    assert!(!is_sync_pair(0x40, 0x40)); // same
}

// ── 24-bit signed decode ─────────────────────────────────────────────────────

#[test]
fn test_decode_int24_positive() {
    // 0x000001 = 1
    assert_eq!(decode_int24(&[0x01, 0x00, 0x00]), 1);
    // 0x7FFFFF = 8388607 (max positive)
    assert_eq!(decode_int24(&[0xFF, 0xFF, 0x7F]), 8388607);
}

#[test]
fn test_decode_int24_negative() {
    // 0xFFFFFF = -1
    assert_eq!(decode_int24(&[0xFF, 0xFF, 0xFF]), -1);
    // 0x800000 = -8388608 (min negative)
    assert_eq!(decode_int24(&[0x00, 0x00, 0x80]), -8388608);
    // 0xFFFFFE = -2
    assert_eq!(decode_int24(&[0xFE, 0xFF, 0xFF]), -2);
}

#[test]
fn test_decode_int24_zero() {
    assert_eq!(decode_int24(&[0x00, 0x00, 0x00]), 0);
}

// ── Frame decode ─────────────────────────────────────────────────────────────

#[test]
fn test_frame_channels_zero() {
    let frame = Frame { raw: [0u8; FRAME_SIZE] };
    let ch = frame.channels_uv();
    for &v in ch.iter() {
        assert_eq!(v, 0.0);
    }
}

#[test]
fn test_frame_sync() {
    let mut raw = [0u8; FRAME_SIZE];
    raw[0] = 0x60;
    let frame = Frame { raw };
    assert_eq!(frame.sync(), 0x60);
}

#[test]
fn test_frame_steering() {
    let mut raw = [0u8; FRAME_SIZE];
    raw[3] = 5;
    let frame = Frame { raw };
    assert_eq!(frame.steering(), 5);
}

#[test]
fn test_frame_special_data() {
    let mut raw = [0u8; FRAME_SIZE];
    raw[4] = 0xD0; raw[5] = 0x07; // 2000 LE
    let frame = Frame { raw };
    assert_eq!(frame.special_data(), 2000);
}

#[test]
fn test_frame_channel_conversion() {
    let mut raw = [0u8; FRAME_SIZE];
    // Set ch1 (offset 6) to 0x000064 = 100
    raw[6] = 0x64; raw[7] = 0x00; raw[8] = 0x00;
    let frame = Frame { raw };
    let ch = frame.channels_uv();
    // 100 * 0.01658 = 1.658 µV
    assert!((ch[0] - 1.658).abs() < 0.001, "ch[0]={}", ch[0]);
}

#[test]
fn test_frame_negative_channel() {
    let mut raw = [0u8; FRAME_SIZE];
    // Set ch1 to 0xFFFFFF = -1
    raw[6] = 0xFF; raw[7] = 0xFF; raw[8] = 0xFF;
    let frame = Frame { raw };
    let ch = frame.channels_uv();
    // -1 * 0.01658 = -0.01658
    assert!((ch[0] - (-0.01658)).abs() < 0.001, "ch[0]={}", ch[0]);
}

// ── Channels ─────────────────────────────────────────────────────────────────

#[test]
fn test_channels_get_by_name() {
    let mut ch = Channels::zero();
    ch.data[0] = 42.0;
    ch.data[4] = 99.0;
    assert_eq!(ch.get("Fp1"), Some(42.0));
    assert_eq!(ch.get("O1"), Some(99.0));
    assert_eq!(ch.get("nonexistent"), None);
}

#[test]
fn test_channels_eeg_slice() {
    let ch = Channels::zero();
    assert_eq!(ch.eeg().len(), NUM_EEG_CHANNELS);
}

// ── Impedances ───────────────────────────────────────────────────────────────

#[test]
fn test_impedance_update_channel() {
    let mut imp = Impedances::zero();
    let mut raw = [0u8; FRAME_SIZE];
    raw[3] = 1; // steering = ch1
    raw[4] = 0xC8; raw[5] = 0x00; // special = 200
    let frame = Frame { raw };
    imp.update_from_frame(&frame);
    // 200 * 0.005 = 1.0 kΩ
    assert!((imp.active[0] - 1.0).abs() < 0.001);
}

#[test]
fn test_impedance_update_ref() {
    let mut imp = Impedances::zero();
    let mut raw = [0u8; FRAME_SIZE];
    raw[3] = 28; // steering = A1 ref for ch 1-22
    raw[4] = 0x90; raw[5] = 0x01; // special = 400
    let frame = Frame { raw };
    imp.update_from_frame(&frame);
    let expected = 400.0 * CONVERT_KOHM;
    for j in 0..22 {
        assert!((imp.reference[j] - expected).abs() < 0.001, "ref[{}]={}", j, imp.reference[j]);
    }
}

#[test]
fn test_impedance_steering_zero_ignored() {
    let mut imp = Impedances::zero();
    let frame = Frame { raw: [0u8; FRAME_SIZE] }; // steering=0
    imp.update_from_frame(&frame);
    assert_eq!(imp.active, [0.0; NUM_CHANNELS]);
}

// ── DeviceState ──────────────────────────────────────────────────────────────

#[test]
fn test_device_states() {
    assert_ne!(DeviceState::Disconnected, DeviceState::Connected);
    assert_ne!(DeviceState::Synced, DeviceState::Syncing);
}

// ── Baud rate codes ──────────────────────────────────────────────────────────

#[test] fn test_br_460800() { assert_eq!(BR_460800, 0x10); }
#[test] fn test_br_115200() { assert_eq!(BR_115200, 0x20); }
#[test] fn test_br_9600() { assert_eq!(BR_9600, 0x30); }

// ── Login codes ──────────────────────────────────────────────────────────────

#[test] fn test_login_early2e() { assert_eq!(LOGIN_EARLY2E, 11); }
#[test] fn test_login_ready2e() { assert_eq!(LOGIN_READY2E, 12); }
#[test] fn test_login_wideb2e() { assert_eq!(LOGIN_WIDEB2E, 13); }

// ── Protocol constants ───────────────────────────────────────────────────────

#[test] fn test_min_syncs() { assert_eq!(MIN_SYNCS, 4); }
#[test] fn test_data_timeout() { assert_eq!(DATA_TIMEOUT_MS, 3000); }
#[test] fn test_baud_rate() { assert_eq!(BAUD_RATE, 460800); }
#[test] fn test_initial_baud_rate() { assert_eq!(INITIAL_BAUD_RATE, 9600); }
#[test] fn test_serial_buffer_size() { assert_eq!(SERIAL_BUFFER_SIZE, 1024 * 1024); }

// ── Additional edge cases ────────────────────────────────────────────────────

#[test]
fn test_sync_pair_all_valid_pairs() {
    // All 7 valid consecutive pairs
    assert!(is_sync_pair(0x20, 0x40));
    assert!(is_sync_pair(0x40, 0x60));
    assert!(is_sync_pair(0x60, 0x80));
    assert!(is_sync_pair(0x80, 0xA0));
    assert!(is_sync_pair(0xA0, 0xC0));
    assert!(is_sync_pair(0xC0, 0xE0));
    assert!(is_sync_pair(0xE0, 0x20));
}

#[test]
fn test_sync_pair_invalid() {
    assert!(!is_sync_pair(0x00, 0x20));
    assert!(!is_sync_pair(0x10, 0x30));
    assert!(!is_sync_pair(0x20, 0x20));
    assert!(!is_sync_pair(0x20, 0x80));
    assert!(!is_sync_pair(0xFF, 0x20));
}

#[test]
fn test_decode_int24_boundary() {
    // 0x7FFFFF = max positive 24-bit
    assert_eq!(decode_int24(&[0xFF, 0xFF, 0x7F]), 8_388_607);
    // 0x800000 = min negative 24-bit
    assert_eq!(decode_int24(&[0x00, 0x00, 0x80]), -8_388_608);
}

#[test]
fn test_frame_all_24_channels() {
    let mut raw = [0u8; FRAME_SIZE];
    // Set each channel to its index value (1-24)
    for i in 0..NUM_CHANNELS {
        let offset = 6 + i * 3;
        let val = (i + 1) as u8;
        raw[offset] = val;
        raw[offset + 1] = 0;
        raw[offset + 2] = 0;
    }
    let frame = Frame { raw };
    let ch = frame.raw_channels();
    for i in 0..NUM_CHANNELS {
        assert_eq!(ch[i], (i + 1) as i32, "ch[{}]", i);
    }
    let uv = frame.channels_uv();
    for i in 0..NUM_CHANNELS {
        let expected = (i + 1) as f32 * CONVERT_UV;
        assert!((uv[i] - expected).abs() < 0.001, "uv[{}]={} expected {}", i, uv[i], expected);
    }
}

#[test]
fn test_impedance_all_steering_values() {
    // Test steering 1-22 sets active impedance
    for steering in 1..=22u8 {
        let mut imp = Impedances::zero();
        let mut raw = [0u8; FRAME_SIZE];
        raw[3] = steering;
        raw[4] = 0xC8; raw[5] = 0x00; // 200
        let frame = Frame { raw };
        imp.update_from_frame(&frame);
        assert!((imp.active[steering as usize - 1] - 1.0).abs() < 0.001,
            "steering {} → active[{}]={}", steering, steering - 1, imp.active[steering as usize - 1]);
    }
}

#[test]
fn test_impedance_steering_27_unused() {
    let mut imp = Impedances::zero();
    let mut raw = [0u8; FRAME_SIZE];
    raw[3] = 27;
    raw[4] = 0xFF; raw[5] = 0x7F;
    let frame = Frame { raw };
    imp.update_from_frame(&frame);
    // Should not change anything
    assert_eq!(imp.active, [0.0; NUM_CHANNELS]);
    assert_eq!(imp.reference, [0.0; NUM_CHANNELS]);
}

#[test]
fn test_impedance_steering_out_of_range() {
    let mut imp = Impedances::zero();
    for steering in [0u8, 29, 30, 255] {
        let mut raw = [0u8; FRAME_SIZE];
        raw[3] = steering;
        raw[4] = 0xFF; raw[5] = 0x7F;
        let frame = Frame { raw };
        imp.update_from_frame(&frame);
    }
    assert_eq!(imp.active, [0.0; NUM_CHANNELS]);
    assert_eq!(imp.reference, [0.0; NUM_CHANNELS]);
}

#[test]
fn test_channels_zero() {
    let ch = Channels::zero();
    assert_eq!(ch.data, [0.0; NUM_CHANNELS]);
}

#[test]
fn test_impedances_zero() {
    let imp = Impedances::zero();
    assert_eq!(imp.active, [0.0; NUM_CHANNELS]);
    assert_eq!(imp.reference, [0.0; NUM_CHANNELS]);
}

#[test]
fn test_frame_special_data_negative() {
    let mut raw = [0u8; FRAME_SIZE];
    raw[4] = 0x00; raw[5] = 0x80; // -32768 LE
    let frame = Frame { raw };
    assert_eq!(frame.special_data(), -32768);
}
