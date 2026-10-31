// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Heap accounting for this measurement binary only. `GlobalAlloc` is an
//! unsafe trait, so implementing it needs `unsafe`; the wrapper forwards every
//! call unchanged to `System` and only counts bytes. It is the one portable way
//! to read live and peak heap without a new dependency, and the core library
//! itself stays free of `unsafe`.

use serde::Serialize;
use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

struct Counting;
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

fn grew(bytes: usize) {
    let now = LIVE.fetch_add(bytes, Relaxed) + bytes;
    PEAK.fetch_max(now, Relaxed);
}

// SAFETY: each method forwards its arguments unchanged to `System`, which
// upholds the `GlobalAlloc` contract; the counters never touch the memory.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            grew(layout.size());
        }
        p
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc_zeroed(layout) };
        if !p.is_null() {
            grew(layout.size());
        }
        p
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        LIVE.fetch_sub(layout.size(), Relaxed);
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let p = unsafe { System.realloc(ptr, layout, size) };
        if !p.is_null() {
            LIVE.fetch_sub(layout.size(), Relaxed);
            grew(size);
        }
        p
    }
}

#[global_allocator]
static HEAP: Counting = Counting;

pub(super) fn live() -> usize {
    LIVE.load(Relaxed)
}
pub(super) fn peak() -> usize {
    PEAK.load(Relaxed)
}
/// Starts a new high-water mark at the current live heap and returns it.
pub(super) fn reset_peak() -> usize {
    let now = live();
    PEAK.store(now, Relaxed);
    now
}

#[derive(Serialize)]
pub(super) struct Control {
    allocated_bytes: usize,
    live_rise_bytes: usize,
    live_fall_bytes: usize,
    pub(super) passed: bool,
}

/// Positive control: the instrument must see a known allocation come and go.
pub(super) fn control() -> Control {
    let size = 64 << 20;
    let before = live();
    let block = black_box(vec![1u8; size]);
    let during = live();
    drop(block);
    let after = live();
    let rise = during.saturating_sub(before);
    let fall = during.saturating_sub(after);
    Control {
        allocated_bytes: size,
        live_rise_bytes: rise,
        live_fall_bytes: fall,
        passed: rise >= size && fall >= size,
    }
}
