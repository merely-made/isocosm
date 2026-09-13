// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Serial shipping-roster world-tick receipt. No rendering.
//!
//! The timer encloses only `World::apply`. Per-channel reconciliation, flow
//! reduction and snapshot work intentionally happen after each timed tick.
use std::{collections::BTreeMap, hint::black_box, time::Instant};

use mesocosm_core::{
    Intent, OrganismId, World,
    flow::{Account, Conversion, RecordedFlow},
    matter::{Material, Stock},
    snapshot,
    world::FOUNDERS,
};
use serde_json::json;

type Key = (Account, Option<OrganismId>);

#[derive(Default)]
struct ConversionCounts {
    synthesis_events: u64,
    synthesis_mg: u64,
    digestion_events: u64,
    digestion_mg: u64,
    mineralization_events: u64,
    mineralization_mg: u64,
}

impl ConversionCounts {
    fn record(&mut self, flows: &[RecordedFlow]) {
        for flow in flows {
            let event = flow.record;
            match event
                .composition
                .and_then(|composition| composition.conversion)
            {
                Some(Conversion::Synthesis) => {
                    self.synthesis_events += 1;
                    self.synthesis_mg += event.amount_mg;
                },
                Some(Conversion::Digestion) => {
                    self.digestion_events += 1;
                    self.digestion_mg += event.amount_mg;
                },
                Some(Conversion::Mineralization) => {
                    self.mineralization_events += 1;
                    self.mineralization_mg += event.amount_mg;
                },
                None => {},
            }
        }
    }

    fn json(&self) -> serde_json::Value {
        json!({
            "synthesis": {"events": self.synthesis_events, "mg": self.synthesis_mg},
            "digestion": {"events": self.digestion_events, "mg": self.digestion_mg},
            "mineralization": {"events": self.mineralization_events, "mg": self.mineralization_mg},
        })
    }
}

fn add(book: &mut BTreeMap<Key, Stock>, key: Key, stock: Stock) {
    let held = book.get(&key).copied().unwrap_or(Stock::EMPTY);
    book.insert(key, held.checked_add(stock).expect("finite accounts add"));
}

fn accounts(world: &World) -> BTreeMap<Key, Stock> {
    let mut book = BTreeMap::new();
    add(&mut book, (Account::Soil, None), world.soil().total_stock());
    for organism in &world.organisms {
        add(
            &mut book,
            (Account::Substance, Some(organism.id)),
            organism
                .phenotype
                .total_stock()
                .expect("body stock is valid"),
        );
        add(
            &mut book,
            (Account::Reserve, Some(organism.id)),
            Stock::single(Material::Untyped, organism.energy_mg),
        );
    }
    book
}

fn side(account: Account, subject: Option<mesocosm_core::flow::Subject>) -> Key {
    if account.is_body() {
        (
            account,
            Some(subject.expect("body flow names its subject").organism),
        )
    } else {
        (account, None)
    }
}

fn reconcile(mut before: BTreeMap<Key, Stock>, flows: &[RecordedFlow]) -> BTreeMap<Key, Stock> {
    for flow in flows {
        let event = flow.record;
        let composition = event.composition.expect("live flow carries composition");
        let source = side(event.source, event.from);
        let destination = side(event.destination, event.to);
        let remainder = before
            .get(&source)
            .copied()
            .unwrap_or(Stock::EMPTY)
            .checked_sub(composition.input)
            .expect("flow input is present in its source account");
        before.insert(source, remainder);
        add(&mut before, destination, composition.output);
    }
    before.retain(|_, stock| *stock != Stock::EMPTY);
    before
}

fn main() {
    const WARMUP_TICKS: usize = 20;
    const MEASURED_TICKS: usize = 200;
    const TOTAL_TICKS: usize = WARMUP_TICKS + MEASURED_TICKS;

    let mut runs = Vec::new();
    for seed in [1, 4, 7] {
        let mut world = World::new(seed, FOUNDERS);
        let rules = world.rules();
        let initial = world.total_matter_mg();
        let mut times = Vec::new();
        let mut conversions = ConversionCounts::default();
        for tick in 0..TOTAL_TICKS {
            let before = accounts(&world);
            let start = Instant::now();
            black_box(&mut world).apply(Intent::Idle);
            let ms = start.elapsed().as_secs_f64() * 1_000.0;
            let flows = world.drain_flows();
            let expected = reconcile(before, &flows);
            let mut actual = accounts(&world);
            actual.retain(|_, stock| *stock != Stock::EMPTY);
            assert_eq!(
                expected, actual,
                "per-channel accounts reconcile on tick {tick}"
            );
            conversions.record(&flows);
            if tick >= WARMUP_TICKS {
                times.push(ms);
            }
            assert_eq!(world.total_matter_mg(), initial);
        }
        let bytes = snapshot::snapshot(&world).unwrap();
        let restored = snapshot::restore_under(&bytes, world.admitted()).unwrap();
        assert_eq!(
            snapshot::state_hash(&world),
            snapshot::state_hash(&restored)
        );
        times.sort_by(f64::total_cmp);
        runs.push(json!({
            "seed": seed,
            "founders_including_played": FOUNDERS + 1,
            "living_at_end": world.living().count(),
            "median_tick_ms": times[MEASURED_TICKS / 2],
            "p95_tick_ms": times[MEASURED_TICKS * 95 / 100 - 1],
            "max_tick_ms": times[MEASURED_TICKS - 1],
            "snapshot_bytes": bytes.len(),
            "state_hash": format!("{:016x}", snapshot::state_hash(&world)),
            "total_mg": initial,
            "snapshot_roundtrip_exact": true,
            "rules": rules,
            "typed_conversion_counts": conversions.json(),
        }));
    }
    let output = json!({
        "scope": "Shipping-roster World::apply ticks with per-channel flow reconciliation; not a full-lifetime ecology receipt",
        "optimized": !cfg!(debug_assertions),
        "grammar_revision": mesocosm_core::TROPHIC_GRAMMAR_REVISION,
        "warmup_ticks": WARMUP_TICKS,
        "measured_ticks_per_seed": MEASURED_TICKS,
        "total_ticks_per_seed": TOTAL_TICKS,
        "timing_scope": "World::apply only; reconciliation, flow reduction, snapshot and rendering are excluded",
        "runs": runs,
    });
    let rendered = serde_json::to_string_pretty(&output).unwrap();
    if let Some(path) = std::env::args().nth(1) {
        std::fs::write(path, &rendered).unwrap();
    }
    println!("{rendered}");
}
