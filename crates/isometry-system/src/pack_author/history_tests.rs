//! History modes: text-only retention, the document node cache and the byte budget.
use super::*;
use history::{Mode, Policy, Retained};
use std::{collections::HashSet, time::Instant};
use taplo::rowan::{GreenNodeData, NodeOrToken};

const ANNOTATED: &str = include_str!("../../tests/fixtures/annotated-pack.toml");
fn open(f: &Fixture, mode: Mode, limit: usize, budget: usize) -> Session {
    Session::open_with(
        &f.0,
        Policy {
            mode,
            limit,
            budget,
        },
    )
    .unwrap()
}
fn rename(s: &mut Session, i: usize) {
    let from = match i {
        0 => "Demo Generator Pack".to_string(),
        _ => format!("Revision {}", i - 1),
    };
    replace(
        s,
        &format!("name = \"{from}\""),
        &format!("name = \"Revision {i}\""),
    );
}
/// Pointer identities of every green node and token in a snapshot's tree.
fn green(snapshot: &Snapshot) -> (Vec<usize>, Vec<usize>) {
    let root = snapshot.syntax.as_ref().unwrap().clone().into_syntax();
    let (mut nodes, mut tokens) = (vec![], vec![]);
    for element in root.descendants_with_tokens() {
        match element {
            NodeOrToken::Node(n) => nodes.push(n.green() as *const GreenNodeData as usize),
            NodeOrToken::Token(t) => tokens.push(t.green() as *const _ as usize),
        }
    }
    (nodes, tokens)
}
fn shared(new: &[usize], old: &[usize]) -> usize {
    let old = old.iter().collect::<HashSet<_>>();
    new.iter().filter(|p| old.contains(p)).count()
}

#[test]
fn text_history_keeps_trees_only_for_current_and_pinned_revisions() {
    let f = Fixture::new();
    f.write(ANNOTATED);
    let mut s = open(&f, Mode::Text, 8, usize::MAX);
    let first = Arc::downgrade(&s.current);
    rename(&mut s, 0);
    let pinned = s.current.clone();
    rename(&mut s, 1);
    rename(&mut s, 2);
    assert!(first.upgrade().is_none(), "history must not keep old trees");
    let kinds = s
        .history
        .entries
        .values()
        .map(|r| matches!(r, Retained::Tree(_)));
    assert_eq!(kinds.collect::<Vec<_>>(), [false, false, false, true]);
    // A reader's snapshot is returned as is; an unpinned one is reparsed exactly.
    assert!(Arc::ptr_eq(&s.snapshot(2).unwrap(), &pinned));
    let rebuilt = s.snapshot(1).unwrap();
    assert_eq!(&*rebuilt.text, ANNOTATED);
    assert_eq!(rebuilt.fingerprint, fingerprint(ANNOTATED.as_bytes()));
    assert_eq!(
        rebuilt
            .syntax
            .as_ref()
            .unwrap()
            .clone()
            .into_syntax()
            .to_string(),
        ANNOTATED
    );
    s.undo(4, 1).unwrap();
    assert_eq!(&*s.current.text, ANNOTATED);
    assert!(s.history.retained_bytes() < 8 * ANNOTATED.len() + s.current.bytes);
}
#[test]
fn cached_history_shares_unchanged_elements_and_resets_its_cache() {
    let f = Fixture::new();
    f.write(ANNOTATED);
    let mut s = open(&f, Mode::Cached, 4, usize::MAX);
    let original = s.current.clone();
    rename(&mut s, 0);
    let (nodes, tokens) = green(&s.current);
    let (old_nodes, old_tokens) = green(&original);
    assert!(shared(&nodes, &old_nodes) > 0 && shared(&tokens, &old_tokens) > 0);
    // Cached trees equal a fresh parse of the same text.
    let fresh = taplo::parser::parse(&s.current.text);
    assert_eq!(
        s.current.syntax.as_ref().unwrap().green_node,
        fresh.green_node
    );
    // After `limit` parses a fresh cache starts; its first tree shares nothing.
    // Parses so far: open and one rename. The fifth parse starts a new cache.
    for i in 1..3 {
        rename(&mut s, i);
    }
    let before = s.current.clone();
    assert!(shared(&green(&before).1, &tokens) > 0);
    rename(&mut s, 3);
    assert_eq!(shared(&green(&s.current).1, &green(&before).1), 0);
    s.release_history();
    assert_eq!(s.history.len(), 1);
}
#[test]
fn the_byte_budget_evicts_before_the_count_cap_but_keeps_current() {
    let f = Fixture::new();
    f.write(ANNOTATED);
    for mode in [Mode::Full, Mode::Text, Mode::Cached] {
        let mut s = open(&f, mode, 32, 0);
        for i in 0..5 {
            rename(&mut s, i);
        }
        assert_eq!(s.history.len(), 1, "{mode:?}");
        assert!(s.undo(s.current.revision, 1).is_err());
        let one = s.current.bytes;
        let mut s = open(&f, mode, 32, 3 * one + one / 2);
        for i in 0..8 {
            rename(&mut s, i);
        }
        let expected = if mode == Mode::Text { 32 } else { 3 };
        assert!(
            s.history.len() >= 3.min(expected) && s.history.len() <= expected,
            "{mode:?}"
        );
        assert!(s.history.retained_bytes() <= 3 * one + one / 2);
    }
}

/// Requested-heap measurement in the method of `Code/testing/retained-document-20261016`:
/// the annotated pack, 1/16/64 renames, history release, then document drop.
#[test]
#[ignore = "measurement; run in release with --ignored and PACK_AUTHOR_HISTORY_REPORT"]
fn measure_history_modes() {
    let Some(report) = std::env::var_os("PACK_AUTHOR_HISTORY_REPORT") else {
        return;
    };
    let f = Fixture::new();
    f.write(ANNOTATED);
    let modes = [Mode::Full, Mode::Text, Mode::Cached];
    let mut memory = vec![];
    for mode in modes {
        // Warm lazy library state (first open and first edit) outside the measurement.
        let mut warm = open(&f, mode, 4, usize::MAX);
        rename(&mut warm, 0);
        drop(warm);
        for edits in [1usize, 16, 64] {
            let base = meter::live();
            let mut s = open(&f, mode, edits + 1, usize::MAX);
            let original = Arc::downgrade(&s.current);
            for i in 0..edits {
                rename(&mut s, i);
            }
            let retained = meter::live() - base;
            let (revisions, estimated) = (s.history.len(), s.history.retained_bytes());
            s.release_history();
            assert!(original.upgrade().is_none());
            drop(original);
            let released = meter::live() - base;
            drop(s);
            let dropped = meter::live() - base;
            assert_eq!(dropped, 0, "{mode:?} document drop");
            memory.push(
                serde_json::json!({"mode":format!("{mode:?}").to_lowercase(),
                "retained_revisions":revisions,"retained_requested_bytes":retained,
                "estimated_budget_bytes":estimated,"after_history_release_bytes":released,
                "after_document_drop_bytes":dropped}),
            );
        }
    }
    let mut sharing = vec![];
    for mode in modes {
        let mut s = open(&f, mode, 65, usize::MAX);
        let mut pins = vec![s.current.clone()];
        for i in 0..64 {
            rename(&mut s, i);
            pins.push(s.current.clone());
        }
        let ids = pins.iter().map(|p| green(p)).collect::<Vec<_>>();
        let (last, previous, first) = (&ids[64], &ids[63], &ids[0]);
        sharing.push(
            serde_json::json!({"mode":format!("{mode:?}").to_lowercase(),
            "final_nodes":last.0.len(),"final_tokens":last.1.len(),
            "nodes_shared_with_previous":shared(&last.0,&previous.0),
            "tokens_shared_with_previous":shared(&last.1,&previous.1),
            "nodes_shared_with_original":shared(&last.0,&first.0),
            "tokens_shared_with_original":shared(&last.1,&first.1)}),
        );
    }
    let wide = format!(
        "{ANNOTATED}\n[vendor.wide]\n{}",
        (0..4096)
            .map(|i| format!("key_{i} = {i}\n"))
            .collect::<String>()
    );
    let widest = format!(
        "{ANNOTATED}\n[vendor.wide]\n{}",
        (0..65536)
            .map(|i| format!("key_{i} = {i}\n"))
            .collect::<String>()
    );
    let demo = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/packs/demo/isometry-pack.toml"),
    )
    .unwrap();
    let mut timing = vec![];
    for (fixture, source, target, count) in [
        ("real", demo.as_str(), "Demo Generator Pack", 500),
        ("annotated", ANNOTATED, "Demo Generator Pack", 500),
        ("wide4096", wide.as_str(), "key_2048 = 2048", 33),
        ("wide65536", widest.as_str(), "key_32768 = 32768", 33),
    ] {
        f.write(source);
        for mode in modes {
            let mut s = open(&f, mode, Policy::DEFAULT_LIMIT, Policy::DEFAULT_BUDGET);
            let start = Instant::now();
            for i in 0..count {
                let from = if i == 0 {
                    target.to_string()
                } else {
                    format!("{target}{}", i - 1)
                };
                replace(&mut s, &from, &format!("{target}{i}"));
            }
            let micros = start.elapsed().as_secs_f64() * 1e6 / count as f64;
            let start = Instant::now();
            for _ in 0..count {
                std::hint::black_box(history::estimate(
                    &s.current.text,
                    s.current.syntax.as_ref(),
                ));
            }
            let estimate = start.elapsed().as_secs_f64() * 1e6 / count as f64;
            timing.push(serde_json::json!({"fixture":fixture,"mode":format!("{mode:?}").to_lowercase(),
                "edits":count,"mean_edit_publication_us":(micros * 100.0).round() / 100.0,
                "of_which_budget_estimate_us":(estimate * 100.0).round() / 100.0,
                "retained_revisions":s.history.len(),"estimated_budget_bytes":s.history.retained_bytes(),
                "current_estimated_bytes":s.current.bytes}));
        }
    }
    let out = serde_json::json!({"fixture_bytes":ANNOTATED.len(),"memory":memory,
        "sharing":sharing,"timing":timing});
    fs::write(report, serde_json::to_string_pretty(&out).unwrap() + "\n").unwrap();
}
