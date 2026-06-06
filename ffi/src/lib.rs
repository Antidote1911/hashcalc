use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::OnceLock;

use hashcalc_core::Algorithm;
use strum::IntoEnumIterator;

static ALGO_NAMES: OnceLock<Vec<CString>> = OnceLock::new();

fn algo_names() -> &'static [CString] {
    ALGO_NAMES.get_or_init(|| {
        Algorithm::iter()
            .map(|a| CString::new(a.to_string()).unwrap())
            .collect()
    })
}

fn all_algos() -> Vec<Algorithm> {
    Algorithm::iter().collect()
}

#[no_mangle]
pub extern "C" fn hashcalc_algorithm_count() -> c_int {
    algo_names().len() as c_int
}

/// Returns the algorithm name as a static C string. Do not free.
#[no_mangle]
pub extern "C" fn hashcalc_algorithm_name(index: c_int) -> *const c_char {
    let names = algo_names();
    if index < 0 || (index as usize) >= names.len() {
        return std::ptr::null();
    }
    names[index as usize].as_ptr()
}

/// Hash a file without progress tracking.
#[no_mangle]
pub unsafe extern "C" fn hashcalc_hash_file(
    path: *const c_char,
    algo_index: c_int,
) -> *mut c_char {
    let compute = || -> Option<*mut c_char> {
        let path_str = CStr::from_ptr(path).to_str().ok()?;
        let algo = *all_algos().get(algo_index as usize)?;
        let hash = algo.into_hasher()(Path::new(path_str)).ok()?;
        Some(CString::new(hash).ok()?.into_raw())
    };
    compute().unwrap_or(std::ptr::null_mut())
}

/// Hash a file with byte-level progress and cooperative cancellation.
///
/// `bytes_done` — pointer to a caller-owned zero-initialised `uint64_t`
///   (use `std::atomic<uint64_t>` on the C++ side).  Updated with relaxed
///   atomics; safe to poll from another thread.
///
/// `file_size`  — written once before hashing starts.  May be NULL.
///
/// `cancelled`  — pointer to a caller-owned `uint8_t`
///   (use `std::atomic<uint8_t>` on the C++ side).  The hasher checks this
///   flag at every 64 KiB chunk boundary; writing 1 stops the computation
///   and causes the function to return NULL.  May be NULL to disable.
///
/// Returns a heap string on success; free with `hashcalc_free_string`.
/// Returns NULL on error or cancellation.
#[no_mangle]
pub unsafe extern "C" fn hashcalc_hash_file_progress(
    path: *const c_char,
    algo_index: c_int,
    bytes_done: *mut u64,
    file_size: *mut u64,
    cancelled: *const u8,
) -> *mut c_char {
    let compute = || -> Option<*mut c_char> {
        let path_str = CStr::from_ptr(path).to_str().ok()?;
        let algo = *all_algos().get(algo_index as usize)?;
        let p = Path::new(path_str);

        if !file_size.is_null() {
            *file_size = std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
        }
        if !bytes_done.is_null() {
            std::ptr::write_volatile(bytes_done, 0);
        }

        let hash = if !bytes_done.is_null() && !cancelled.is_null() {
            // SAFETY: both pointers come from std::atomic<T> on the C++ side,
            // which has the same size and alignment as the underlying integer.
            let progress = &*(bytes_done as *const AtomicU64);
            let cancel   = &*(cancelled  as *const AtomicBool);
            algo.into_hasher_cancellable()(p, progress, cancel).ok()?
        } else if !bytes_done.is_null() {
            let progress = &*(bytes_done as *const AtomicU64);
            algo.into_hasher_with_progress()(p, progress).ok()?
        } else {
            algo.into_hasher()(p).ok()?
        };

        Some(CString::new(hash).ok()?.into_raw())
    };
    compute().unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn hashcalc_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}
