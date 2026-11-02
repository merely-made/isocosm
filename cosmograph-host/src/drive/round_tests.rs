// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use std::time::{Duration, Instant};

#[test]
fn each_speed_steps_sixty_a_second_and_ticks_every_sixty_over_speed_seconds() {
    for speed in Speed::ALL {
        let mut pacer = Pacer::new(Pace::Round { speed });
        let mut steps = 0;
        let mut ticked_at = Vec::new();
        // Two real minutes, a second at a time.
        for second in 1..=120u64 {
            let due = pacer.advance(SECOND_US);
            steps += due.steps;
            ticked_at.extend(std::iter::repeat_n(second, due.ticks as usize));
        }
        // The k-th minute falls due at k * 60 / speed real seconds, so in the
        // first whole second at or after it (7.5 s at eight times).
        let f = speed.factor();
        let expected: Vec<u64> = (1..=2 * f).map(|k| (k * 60).div_ceil(f)).collect();
        assert_eq!(steps, 120 * 60, "{speed:?}: sixty steps a real second");
        assert_eq!(ticked_at, expected, "{speed:?}: a minute tick every 60/{f} real seconds");
        assert_eq!(pacer.step_world_us(), Some((SECOND_US * speed.factor(), 60)), "a sixtieth of a second times the speed");
    }
}

#[test]
fn ragged_delivery_authorises_the_same_steps_and_ticks() {
    let chunks = [1u64, 29_999_999, 333, 100_000, 99_667, 0, 30_000];
    let mut ragged = Pacer::new(Pace::Round { speed: Speed::X4 });
    let mut total = Due::default();
    for chunk in chunks {
        let d = ragged.advance(chunk);
        total.steps += d.steps;
        total.ticks += d.ticks;
    }
    let mut whole = Pacer::new(Pace::Round { speed: Speed::X4 });
    assert_eq!(whole.advance(chunks.iter().sum()), total, "only the time elapsed counts");
    assert_eq!(total.ticks, 2, "about thirty real seconds at four times is two minutes");
}

#[test]
fn simming_keeps_the_dial_and_a_change_of_pace_keeps_time_counted() {
    let mut pacer = Pacer::new(Pace::Simming { ticks_per_second: 10 });
    assert_eq!(pacer.advance(SECOND_US), Due { steps: 0, ticks: 10 });
    pacer.set_pace(Pace::Round { speed: Speed::X2 });
    let due = pacer.advance(15 * SECOND_US);
    assert_eq!(due, Due { steps: 900, ticks: 0 }, "thirty world seconds, no minute yet");
    pacer.set_pace(Pace::Round { speed: Speed::X8 });
    assert_eq!(pacer.advance(4 * SECOND_US).ticks, 1, "the half minute counted at two times carries over");
    assert_eq!(pacer.ticks(), 11);
}

/// A frame loop: real time arrives each frame, due steps run, each tick's
/// job carries the steps since the last boundary as its handback, and
/// answers are taken as they arrive.
fn play(mode: TickMode, frames: u32, frame_us: u64, sleep: Duration) -> (u64, Duration, usize) {
    let mut ticker = Ticker::new(mode, move |(index, handback): (u64, u64)| {
        std::thread::sleep(sleep);
        index.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ handback
    });
    let mut pacer = Pacer::new(Pace::Round { speed: Speed::X8 });
    let (mut stepped, mut index, mut fold, mut slowest, mut most_pending) = (0u64, 0u64, 0u64, Duration::ZERO, 0);
    for _ in 0..frames {
        let started = Instant::now();
        let due = pacer.advance(frame_us);
        stepped += due.steps;
        for _ in 0..due.ticks {
            ticker.submit((index, stepped)).unwrap();
            index += 1;
            stepped = 0;
        }
        most_pending = most_pending.max(ticker.pending());
        for out in ticker.poll().unwrap() {
            fold = fold.rotate_left(7) ^ out;
        }
        slowest = slowest.max(started.elapsed());
    }
    for out in ticker.drain().unwrap() {
        fold = fold.rotate_left(7) ^ out;
    }
    (fold, slowest, most_pending)
}

#[test]
fn a_worker_tick_never_blocks_the_frame_thread() {
    // Eight times real time, a second a frame: a minute every 7.5 frames,
    // each tick sleeping 200 ms. The frames carry on while ticks run.
    let (_, slowest, pending) = play(TickMode::Worker, 60, SECOND_US, Duration::from_millis(200));
    assert!(slowest < Duration::from_millis(50), "the slowest frame took {slowest:?}");
    assert!(pending > 0, "ticks were in flight while frames ran");
}

#[test]
fn a_tick_in_step_ends_where_the_worker_ends() {
    let (worker, _, _) = play(TickMode::Worker, 60, SECOND_US, Duration::from_millis(5));
    let (in_step, slowest, _) = play(TickMode::InStep, 60, SECOND_US, Duration::from_millis(5));
    assert_eq!(worker, in_step, "the same inputs end in the same state either way");
    assert!(slowest >= Duration::from_millis(5), "in step, a frame waits on its tick");
}
