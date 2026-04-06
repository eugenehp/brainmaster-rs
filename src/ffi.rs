//! Runtime-loaded FFI bindings to `bmrcm.dll` (Windows only).
//!
//! **All 18 exported functions** are bound for 100% API parity with
//! the BrainMaster Discovery SDK as documented in neuromore studio.
//!
//! On non-Windows platforms, `sdk_lib()` returns an error — use the
//! pure-Rust serial protocol in `protocol.rs` instead.

use std::ffi::c_void;
use std::sync::OnceLock;

use crate::error::BrainMasterError;

// ── Function pointer types ───────────────────────────────────────────────────

type FnAtlOpenPort = unsafe extern "cdecl" fn(i32, i32, *mut *mut c_void) -> i32;
type FnAtlClosePort = unsafe extern "cdecl" fn(i32) -> i32;
type FnAtlSetBaudRate = unsafe extern "cdecl" fn(i32) -> i32;
type FnAtlWriteSamplingRate = unsafe extern "cdecl" fn(i32) -> i32;
type FnAtlReadSamplingRate = unsafe extern "cdecl" fn() -> i32;
type FnAtlClearSpecials = unsafe extern "cdecl" fn() -> i32;
type FnAtlFlush = unsafe extern "cdecl" fn();
type FnBmrLoginDevice = unsafe extern "cdecl" fn(*mut u8, *mut u8, *mut u8) -> i32;
type FnDiscStartModule = unsafe extern "cdecl" fn() -> i32;
type FnDiscStopModule = unsafe extern "cdecl" fn() -> i32;
type FnAtlGetBytesInQue = unsafe extern "cdecl" fn() -> u32;
type FnAtlReadData = unsafe extern "cdecl" fn(*mut u8, i32) -> i32;
type FnAtlSelectImpedanceChans = unsafe extern "cdecl" fn(i32) -> i32;
type FnAtlSelectSpecial = unsafe extern "cdecl" fn(i32) -> i32;
type FnAtlSetNotchFilters = unsafe extern "cdecl" fn(i32) -> i32;
type FnAtlPeek = unsafe extern "cdecl" fn(u16) -> i32;
type FnAtlPoke = unsafe extern "cdecl" fn(u16, u16) -> i32;
type FnAtlQueryFirmware = unsafe extern "cdecl" fn(i32) -> i32;

// ── Library wrapper ──────────────────────────────────────────────────────────

/// Dynamically-loaded bmrcm.dll with all 18 functions.
pub struct BmrcmLib {
    _lib: libloading::Library,

    pub(crate) fn_atl_open_port: FnAtlOpenPort,
    pub(crate) fn_atl_close_port: FnAtlClosePort,
    pub(crate) fn_atl_set_baud_rate: FnAtlSetBaudRate,
    pub(crate) fn_atl_write_sampling_rate: FnAtlWriteSamplingRate,
    pub(crate) fn_atl_read_sampling_rate: FnAtlReadSamplingRate,
    pub(crate) fn_atl_clear_specials: FnAtlClearSpecials,
    pub(crate) fn_atl_flush: FnAtlFlush,
    pub(crate) fn_bmr_login_device: FnBmrLoginDevice,
    pub(crate) fn_disc_start_module: FnDiscStartModule,
    pub(crate) fn_disc_stop_module: FnDiscStopModule,
    pub(crate) fn_atl_get_bytes_in_que: FnAtlGetBytesInQue,
    pub(crate) fn_atl_read_data: FnAtlReadData,
    pub(crate) fn_atl_select_impedance_chans: FnAtlSelectImpedanceChans,
    pub(crate) fn_atl_select_special: FnAtlSelectSpecial,
    pub(crate) fn_atl_set_notch_filters: FnAtlSetNotchFilters,
    pub(crate) fn_atl_peek: FnAtlPeek,
    pub(crate) fn_atl_poke: FnAtlPoke,
    pub(crate) fn_atl_query_firmware: FnAtlQueryFirmware,
}

unsafe impl Send for BmrcmLib {}
unsafe impl Sync for BmrcmLib {}

macro_rules! load_fn {
    ($lib:expr, $name:literal, $ty:ty) => {
        *$lib.get::<$ty>($name).map_err(|e| BrainMasterError::LibraryNotAvailable {
            reason: format!("{}: {}", std::str::from_utf8($name).unwrap_or("?"), e),
        })?
    };
}

impl BmrcmLib {
    fn load() -> Result<Self, BrainMasterError> {
        let lib = unsafe { libloading::Library::new("bmrcm.dll") }.map_err(|e| {
            BrainMasterError::LibraryNotAvailable {
                reason: format!("Could not load bmrcm.dll: {}", e),
            }
        })?;

        unsafe {
            Ok(BmrcmLib {
                fn_atl_open_port: load_fn!(lib, b"AtlOpenPort\0", FnAtlOpenPort),
                fn_atl_close_port: load_fn!(lib, b"AtlClosePort\0", FnAtlClosePort),
                fn_atl_set_baud_rate: load_fn!(lib, b"AtlSetBaudRate\0", FnAtlSetBaudRate),
                fn_atl_write_sampling_rate: load_fn!(lib, b"AtlWriteSamplingRate\0", FnAtlWriteSamplingRate),
                fn_atl_read_sampling_rate: load_fn!(lib, b"AtlReadSamplingRate\0", FnAtlReadSamplingRate),
                fn_atl_clear_specials: load_fn!(lib, b"AtlClearSpecials\0", FnAtlClearSpecials),
                fn_atl_flush: load_fn!(lib, b"AtlFlush\0", FnAtlFlush),
                fn_bmr_login_device: load_fn!(lib, b"BmrLoginDevice\0", FnBmrLoginDevice),
                fn_disc_start_module: load_fn!(lib, b"DiscStartModule\0", FnDiscStartModule),
                fn_disc_stop_module: load_fn!(lib, b"DiscStopModule\0", FnDiscStopModule),
                fn_atl_get_bytes_in_que: load_fn!(lib, b"AtlGetBytesInQue\0", FnAtlGetBytesInQue),
                fn_atl_read_data: load_fn!(lib, b"AtlReadData\0", FnAtlReadData),
                fn_atl_select_impedance_chans: load_fn!(lib, b"AtlSelectImpedanceChans\0", FnAtlSelectImpedanceChans),
                fn_atl_select_special: load_fn!(lib, b"AtlSelectSpecial\0", FnAtlSelectSpecial),
                fn_atl_set_notch_filters: load_fn!(lib, b"AtlSetNotchFilters\0", FnAtlSetNotchFilters),
                fn_atl_peek: load_fn!(lib, b"AtlPeek\0", FnAtlPeek),
                fn_atl_poke: load_fn!(lib, b"AtlPoke\0", FnAtlPoke),
                fn_atl_query_firmware: load_fn!(lib, b"AtlQueryFirmware\0", FnAtlQueryFirmware),
                _lib: lib,
            })
        }
    }
}

static SDK_LIB: OnceLock<Result<BmrcmLib, String>> = OnceLock::new();

/// Get the global bmrcm.dll handle (Windows only).
pub fn sdk_lib() -> Result<&'static BmrcmLib, BrainMasterError> {
    SDK_LIB
        .get_or_init(|| BmrcmLib::load().map_err(|e| e.to_string()))
        .as_ref()
        .map_err(|e| BrainMasterError::LibraryNotAvailable { reason: e.clone() })
}
