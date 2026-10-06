// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Optional current-thread CPU accounting for the receipt.
//!
//! Windows reports cumulative user and kernel FILETIMEs in 100 ns units. They
//! are scheduler-accounted counters, so a short delta may be quantized rather
//! than precisely separating running and waiting within one tick.

#[cfg(windows)]
mod windows {
    use std::ffi::c_void;

    type Bool = i32;
    type Handle = *mut c_void;

    #[repr(C)]
    #[derive(Default)]
    struct FileTime {
        low: u32,
        high: u32,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        #[link_name = "GetCurrentThread"]
        fn get_current_thread() -> Handle;
        #[link_name = "GetThreadTimes"]
        fn get_thread_times(
            thread: Handle,
            creation: *mut FileTime,
            exit: *mut FileTime,
            kernel: *mut FileTime,
            user: *mut FileTime,
        ) -> Bool;
    }

    fn ticks(time: FileTime) -> u64 {
        (u64::from(time.high) << 32) | u64::from(time.low)
    }

    pub(super) fn current_thread_cpu_100ns() -> Option<u64> {
        let mut creation = FileTime::default();
        let mut exit = FileTime::default();
        let mut kernel = FileTime::default();
        let mut user = FileTime::default();
        // SAFETY: FileTime has the SDK FILETIME C layout and all four local
        // buffers are initialized and valid for writable pointers. The current
        // thread pseudo-handle has the documented GetThreadTimes ABI and must
        // not be passed to CloseHandle.
        let success = unsafe {
            get_thread_times(
                get_current_thread(),
                &mut creation,
                &mut exit,
                &mut kernel,
                &mut user,
            )
        };
        (success != 0)
            .then(|| ticks(kernel).checked_add(ticks(user)))
            .flatten()
    }
}

/// Cumulative current-thread user plus kernel time in 100 ns units.
///
/// Returns `None` off Windows or when the operating-system call fails.
pub fn current_thread_cpu_100ns() -> Option<u64> {
    #[cfg(windows)]
    {
        windows::current_thread_cpu_100ns()
    }
    #[cfg(not(windows))]
    {
        None
    }
}
