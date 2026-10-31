// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A single bounded draw pool. Arms within each draw remain sequential.
//! Delivery, errors, refusal rows and progress retain the original k order.

use super::{Options, probe_draw_execution::{self, Completed}, report::Domain};
use std::{collections::BTreeMap, sync::{Arc, Mutex, mpsc}, thread};

const MAX_WORKERS: usize = 64;

pub(super) fn requested(value: u64) -> Result<usize, String> {
    if value == 0 || value > MAX_WORKERS as u64 {
        return Err(format!("--workers must be between 1 and {MAX_WORKERS}"));
    }
    Ok(value as usize)
}

#[derive(serde::Serialize)]
pub(super) struct Execution {
    pub requested_workers: usize,
    pub workers: usize,
}

impl Execution {
    pub(super) fn new(requested_workers: usize, draws: u64) -> Self {
        // Draw zero, including its trace and sanity runs, precedes the pool.
        let remaining = usize::try_from(draws.saturating_sub(1)).unwrap_or(usize::MAX);
        Self { requested_workers, workers: requested_workers.min(remaining.max(1)) }
    }
}

pub(super) fn run(
    o: &Options, domain: &Domain, arms: &[usize],
    mut emit: impl FnMut(Completed) -> Result<(), String>,
) -> Result<(), String> {
    requested(o.workers as u64)?;
    if o.draws == 0 { return Ok(()); }
    let first = probe_draw_execution::execute(o, domain, arms, 0, None)?;
    let expected = probe_draw_execution::signature(&first.infos);
    emit(first)?;
    let workers = Execution::new(o.workers, o.draws).workers;
    if workers == 1 {
        for k in 1..o.draws {
            emit(probe_draw_execution::execute(o, domain, arms, k, Some(&expected))?)?;
        }
        return Ok(());
    }
    thread::scope(|scope| {
        // At most `workers` draws are submitted but not delivered, even when
        // the earliest one is slower. No complete world leaves its worker.
        let (jobs, queue) = mpsc::sync_channel::<u64>(workers);
        let queue = Arc::new(Mutex::new(queue));
        let (send, results) = mpsc::sync_channel(workers);
        let mut handles = Vec::new();
        for worker in 0..workers {
            let (queue, send, expected) = (queue.clone(), send.clone(), &expected);
            let handle = thread::Builder::new().name(format!("probe-draw-{worker}"))
                .spawn_scoped(scope, move || {
                    loop {
                        let job = match queue.lock() {
                            Ok(queue) => queue.recv(),
                            Err(_) => break,
                        };
                        let Ok(k) = job else { break; };
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(||
                            probe_draw_execution::execute(o, domain, arms, k, Some(expected))))
                            .unwrap_or_else(|_| Err(format!("draw {k}: probe worker panicked")));
                        if send.send((k, result)).is_err() { break; }
                    }
                }).map_err(|e| format!("could not start probe worker: {e}"))?;
            handles.push(handle);
        }
        drop(send);
        let mut submitted = 1;
        for _ in 0..workers {
            jobs.send(submitted).map_err(|_| "probe worker queue closed")?;
            submitted += 1;
        }
        let mut pending = BTreeMap::new();
        for wanted in 1..o.draws {
            while !pending.contains_key(&wanted) {
                let (k, completed) = results.recv().map_err(|_| "probe workers ended before all draws")?;
                pending.insert(k, completed);
            }
            emit(pending.remove(&wanted).expect("ordered result received")?)?;
            if submitted < o.draws {
                jobs.send(submitted).map_err(|_| "probe worker queue closed")?;
                submitted += 1;
            }
        }
        // Drop both channels before joining; every early return does the
        // same through scope-local destruction, waking blocked workers.
        drop(jobs);
        drop(results);
        for handle in handles {
            handle.join().map_err(|_| "probe worker panicked outside a draw")?;
        }
        Ok(())
    })
}

#[cfg(test)]
#[path = "probe_parallel_tests.rs"]
mod tests;
