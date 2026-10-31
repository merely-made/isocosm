//! Test-only requested-heap meter. Counts are per thread, so parallel tests do
//! not disturb a measurement that allocates and frees on its own thread.
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
};
thread_local! {
    static LIVE: Cell<isize> = const { Cell::new(0) };
}
struct Meter;
#[global_allocator]
static METER: Meter = Meter;
fn add(bytes: isize) {
    // Ignored during thread-local teardown, after any measurement has ended.
    let _ = LIVE.try_with(|live| live.set(live.get() + bytes));
}
// SAFETY: every call forwards to the system allocator with the caller's layout.
unsafe impl GlobalAlloc for Meter {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            add(layout.size() as isize);
        }
        p
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc_zeroed(layout) };
        if !p.is_null() {
            add(layout.size() as isize);
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, layout: Layout) {
        unsafe { System.dealloc(p, layout) };
        add(-(layout.size() as isize));
    }
    unsafe fn realloc(&self, p: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let q = unsafe { System.realloc(p, layout, size) };
        if !q.is_null() {
            add(size as isize - layout.size() as isize);
        }
        q
    }
}
/// Bytes requested and not yet freed by this thread.
pub(super) fn live() -> isize {
    LIVE.with(Cell::get)
}
