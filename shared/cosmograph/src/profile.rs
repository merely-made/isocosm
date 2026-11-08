// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Bounded opt-in wall timings. Nested spans overlap; never saved authority.
//! `COSMOGRAPH_PROFILE_ACTIVE=1` logs each span; `=sum` totals them by label
//! in memory instead, read back by [`totals`].
use std::{
    collections::BTreeMap,
    sync::{
        Mutex, OnceLock, PoisonError,
        atomic::{AtomicUsize, Ordering},
    },
    time::Instant,
};

pub(crate) struct Span<'a> {
    label: &'a str,
    started: Option<Instant>,
}
/// Aggregate short operations without emitting a log line per posture.
#[derive(Clone, Copy)]
pub(crate) struct Clock(Option<Instant>);
impl Clock {
    pub(crate) fn new() -> Self {
        Self(enabled().then(Instant::now))
    }
    pub(crate) fn ns(self) -> usize {
        self.0.map_or(0, |t| {
            t.elapsed().as_nanos().try_into().unwrap_or(usize::MAX)
        })
    }
}
pub(crate) fn counts(label: &str, values: &[(&str, usize)]) {
    if enabled() && COUNT.fetch_add(1, Ordering::Relaxed) < limit() {
        eprint!("[cosmograph-count] {label}");
        for (key, value) in values {
            eprint!(" {key}={value}");
        }
        eprintln!();
    }
}
static COUNT: AtomicUsize = AtomicUsize::new(0);
/// Per label: nanoseconds spent, spans closed, and counts added by [`tally`].
static TOTALS: Mutex<BTreeMap<String, [u128; 3]>> = Mutex::new(BTreeMap::new());
fn mode() -> u8 {
    static MODE: OnceLock<u8> = OnceLock::new();
    *MODE.get_or_init(|| match std::env::var("COSMOGRAPH_PROFILE_ACTIVE").as_deref() {
        Ok("1") => 1,
        Ok("sum") => 2,
        _ => 0,
    })
}
fn enabled() -> bool {
    mode() != 0
}
fn add(label: &str, slot: usize, value: u128) {
    let mut totals = TOTALS.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(t) = totals.get_mut(label) {
        t[slot] += value;
    } else {
        let mut t = [0; 3];
        t[slot] = value;
        totals.insert(label.to_owned(), t);
    }
}
/// Whether spans are summed, so a caller builds a label only then.
pub(crate) fn summing() -> bool {
    mode() == 2
}
/// Adds `n` to a label's count where spans are summed.
pub(crate) fn tally(label: &str, n: u64) {
    if mode() == 2 {
        add(label, 2, u128::from(n));
    }
}
/// The summed spans so far, by label: nanoseconds, spans closed and the
/// count tallied; empty unless `COSMOGRAPH_PROFILE_ACTIVE=sum`.
#[must_use]
pub fn totals() -> Vec<(String, [u128; 3])> {
    let totals = TOTALS.lock().unwrap_or_else(PoisonError::into_inner);
    totals.iter().map(|(k, v)| (k.clone(), *v)).collect()
}
impl<'a> Span<'a> {
    pub(crate) fn new(label: &'a str) -> Self {
        Self {
            label,
            started: enabled().then(Instant::now),
        }
    }
}
impl Drop for Span<'_> {
    fn drop(&mut self) {
        if let Some(started) = self.started {
            if mode() == 2 {
                add(self.label, 0, started.elapsed().as_nanos());
                add(self.label, 1, 1);
            } else if COUNT.fetch_add(1, Ordering::Relaxed) < limit() {
                eprintln!(
                    "[cosmograph-active] {} {}us",
                    self.label,
                    started.elapsed().as_micros()
                );
            }
        }
    }
}

/// Log cap; `COSMOGRAPH_PROFILE_LIMIT` raises it for whole multi-tick runs.
fn limit() -> usize {
    static LIMIT: OnceLock<usize> = OnceLock::new();
    *LIMIT.get_or_init(|| {
        std::env::var("COSMOGRAPH_PROFILE_LIMIT")
            .ok()
            .and_then(|n| n.parse().ok())
            .unwrap_or(4096)
    })
}
