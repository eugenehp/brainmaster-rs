//! Runtime-loaded FFI bindings to `bmrcm.dll`.
//!
//! **Windows x86 (32-bit) only.** The `bmrcm.dll` from BrainMaster Technologies
//! is a 32-bit Windows DLL using the `cdecl` calling convention. It cannot be
//! loaded on 64-bit or ARM processes.
//!
//! **All 18 exported functions** are bound for 100% API parity.
//!
//! On Windows x64, Windows ARM64, Linux, and macOS, use the pure-Rust serial
//! protocol in [`crate::protocol`] instead — it provides identical functionality
//! without any DLL.
//!
//! # Platform availability
//!
//! | Platform | `bmrcm.dll` | Pure-Rust serial |
//! |---|---|---|
//! | Windows x86 | ✅ | ✅ |
//! | Windows x64 | ❌ (32-bit DLL) | ✅ |
//! | Windows ARM64 | ❌ (32-bit DLL) | ✅ |
//! | Linux x64 | ❌ | ✅ |
//! | Linux ARM64 | ❌ | ✅ |
//! | macOS x64 | ❌ | ✅ |
//! | macOS ARM64 | ❌ | ✅ |

use std::ffi::c_void;
use std::sync::OnceLock;

use crate::error::BrainMasterError;

// ── Function pointer types ───────────────────────────────────────────────────
//
// The bmrcm.dll uses __cdecl calling convention. On 32-bit Windows this
// matters (caller cleans stack). On 64-bit Windows there is only one
// calling convention so `extern "C"` works, but the DLL itself is 32-bit
// and cannot be loaded in a 64-bit process.
//
// We use `extern "C"` which maps to `cdecl` on x86 Windows and is correct
// everywhere else. The DLL simply won't load on non-x86 platforms.

type FnAtlOpenPort = unsafe extern "C" fn(i32, i32, *mut *mut c_void) -> i32;
type FnAtlClosePort = unsafe extern "C" fn(i32) -> i32;
type FnAtlSetBaudRate = unsafe extern "C" fn(i32) -> i32;
type FnAtlWriteSamplingRate = unsafe extern "C" fn(i32) -> i32;
type FnAtlReadSamplingRate = unsafe extern "C" fn() -> i32;
type FnAtlClearSpecials = unsafe extern "C" fn() -> i32;
type FnAtlFlush = unsafe extern "C" fn();
type FnBmrLoginDevice = unsafe extern "C" fn(*mut u8, *mut u8, *mut u8) -> i32;
type FnDiscStartModule = unsafe extern "C" fn() -> i32;
type FnDiscStopModule = unsafe extern "C" fn() -> i32;
type FnAtlGetBytesInQue = unsafe extern "C" fn() -> u32;
type FnAtlReadData = unsafe extern "C" fn(*mut u8, i32) -> i32;
type FnAtlSelectImpedanceChans = unsafe extern "C" fn(i32) -> i32;
type FnAtlSelectSpecial = unsafe extern "C" fn(i32) -> i32;
type FnAtlSetNotchFilters = unsafe extern "C" fn(i32) -> i32;
type FnAtlPeek = unsafe extern "C" fn(u16) -> i32;
type FnAtlPoke = unsafe extern "C" fn(u16, u16) -> i32;
type FnAtlQueryFirmware = unsafe extern "C" fn(i32) -> i32;

// ── Library wrapper ──────────────────────────────────────────────────────────

/// Dynamically-loaded `bmrcm.dll` with all 18 functions.
///
/// Only usable on **Windows x86 (32-bit)** targets. On all other platforms,
/// `sdk_lib()` will return an error directing you to use the pure-Rust
/// serial protocol instead.
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
        // bmrcm.dll is a 32-bit Windows DLL — guard against loading on wrong platform
        #[cfg(not(all(target_os = "windows", target_pointer_width = "32")))]
        {
            return Err(BrainMasterError::LibraryNotAvailable {
                reason: format!(
                    "bmrcm.dll is a Windows 32-bit DLL and cannot be loaded on this platform ({}/{}).\n\
                     Use the pure-Rust serial protocol instead: DiscoveryDevice::open(port)",
                    std::env::consts::OS,
                    std::env::consts::ARCH,
                ),
            });
        }

        #[cfg(all(target_os = "windows", target_pointer_width = "32"))]
        {
            let lib = unsafe { libloading::Library::new("bmrcm.dll") }.map_err(|e| {
                BrainMasterError::LibraryNotAvailable {
                    reason: format!(
                        "Could not load bmrcm.dll: {}\n\
                         Install BrainMaster device drivers, or use the pure-Rust serial protocol.",
                        e
                    ),
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
}

static SDK_LIB: OnceLock<Result<BmrcmLib, String>> = OnceLock::new();

/// Get the global `bmrcm.dll` handle.
///
/// Returns `Err` on non-Windows-x86 platforms. Use the pure-Rust serial
/// protocol via [`crate::protocol::SerialDevice`] instead.
pub fn sdk_lib() -> Result<&'static BmrcmLib, BrainMasterError> {
    SDK_LIB
        .get_or_init(|| BmrcmLib::load().map_err(|e| e.to_string()))
        .as_ref()
        .map_err(|e| BrainMasterError::LibraryNotAvailable { reason: e.clone() })
}
