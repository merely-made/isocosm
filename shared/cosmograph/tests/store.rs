// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! D42's first sparse store is a derived index, never a second native record.

use cosmograph::{
    Command, Execution, Founding, Session,
    schema::Id,
    store::{ReadStore, SparseStore, intersection},
};
use std::collections::BTreeMap;

#[test]
fn sparse_members_keep_native_order_gaps_and_full_width_identities() {
    let ids = [u64::MAX, 12, 0, u64::from(u32::MAX) + 7, 4];
    let mut store = SparseStore::new();
    for id in ids {
        store.set(id, format!("member {id}"));
    }
    let mut ordered = ids.to_vec();
    ordered.sort_unstable();
    assert_eq!(store.iter().map(|(id, _)| id).collect::<Vec<_>>(), ordered);
    assert_eq!(store.len(), 5);
    assert_eq!(store.get(5), None);
    assert_eq!(store.get(u64::MAX).unwrap(), "member 18446744073709551615");

    store.set(12, "replaced".into());
    assert_eq!(store.len(), 5);
    assert_eq!(store.get(12).unwrap(), "replaced");
    assert!(store.remove(12));
    assert!(!store.remove(12));
    assert_eq!(store.len(), 4);
    store.set(12, "restored".into());
    assert_eq!(store.iter().map(|(id, _)| id).collect::<Vec<_>>(), ordered);
}

#[test]
fn snapshots_share_unchanged_nonclone_payloads_and_isolate_all_writes() {
    struct Payload(String);
    let mut original = SparseStore::new();
    original.set(9, Payload("first".into()));
    original.set(27, Payload("unchanged".into()));
    let mut copy = original.clone();
    assert!(std::ptr::eq(original.get(9).unwrap(), copy.get(9).unwrap()));

    copy.set(9, Payload("copy".into()));
    copy.set(100, Payload("copy only".into()));
    assert_eq!(original.get(9).unwrap().0, "first");
    assert_eq!(copy.get(9).unwrap().0, "copy");
    assert!(original.get(100).is_none());
    assert!(std::ptr::eq(
        original.get(27).unwrap(),
        copy.get(27).unwrap()
    ));

    original.set(9, Payload("original".into()));
    assert!(original.remove(27));
    assert_eq!(copy.get(9).unwrap().0, "copy");
    assert_eq!(copy.get(27).unwrap().0, "unchanged");
    assert_eq!(original.len(), 1);
    assert_eq!(copy.len(), 3);
}

#[test]
fn ordered_joins_follow_a_canonical_reference_through_membership_changes() {
    let (mut left, mut right) = (SparseStore::new(), SparseStore::new());
    let (mut a, mut b) = (BTreeMap::new(), BTreeMap::new());
    for step in 0..160u64 {
        let id = (step * 17 + step / 7) % 23;
        match step % 6 {
            0 | 1 => {
                left.set(id, step as i64);
                a.insert(id, step as i64);
            },
            2 | 3 => {
                right.set(id, format!("right {step}"));
                b.insert(id, format!("right {step}"));
            },
            4 => {
                assert_eq!(left.remove(id), a.remove(&id).is_some());
            },
            _ => {
                assert_eq!(right.remove(id), b.remove(&id).is_some());
            },
        }
        let expected: Vec<_> = a
            .iter()
            .filter_map(|(&id, x)| b.get(&id).map(|y| (id, *x, y.clone())))
            .collect();
        let actual: Vec<_> = intersection(&left, &right)
            .map(|(id, x, y)| (id, *x, y.clone()))
            .collect();
        assert_eq!(actual, expected, "after update {step}");
        let reversed: Vec<_> = intersection(&right, &left)
            .map(|(id, y, x)| (id, *x, y.clone()))
            .collect();
        assert_eq!(reversed, expected, "reversed join after update {step}");
    }
    let empty = SparseStore::<i64>::new();
    assert!(empty.is_empty());
    assert_eq!(intersection(&left, &empty).count(), 0);
    assert_eq!(intersection(&empty, &left).count(), 0);
    assert_eq!(intersection(&left, &left).count(), left.len());
}

/// A different read layout in the test proves that joins use the read contract,
/// not sparse-store internals. This is a canonical ordered reference, not a
/// second production backend.
struct OrderedRows(Vec<(Id, i64)>);
impl ReadStore<i64> for OrderedRows {
    type Iter<'a> = Box<dyn Iterator<Item = (Id, &'a i64)> + 'a>;

    fn get(&self, id: Id) -> Option<&i64> {
        self.0
            .binary_search_by_key(&id, |(key, _)| *key)
            .ok()
            .map(|index| &self.0[index].1)
    }

    fn iter(&self) -> Self::Iter<'_> {
        Box::new(self.0.iter().map(|(id, value)| (*id, value)))
    }
}

#[test]
fn queries_join_different_read_layouts_without_allocating_foreign_identities() {
    let reference = OrderedRows(vec![(0, 7), (13, 8), (u64::MAX, 9)]);
    let mut sparse = SparseStore::new();
    sparse.set(u64::MAX, "last");
    sparse.set(6, "not shared");
    sparse.set(0, "first");
    assert_eq!(ReadStore::get(&reference, 13), Some(&8));
    assert_eq!(ReadStore::get(&sparse, 6), Some(&"not shared"));
    assert_eq!(
        intersection(&reference, &sparse)
            .map(|(id, a, b)| (id, *a, *b))
            .collect::<Vec<_>>(),
        vec![(0, 7, "first"), (u64::MAX, 9, "last")]
    );
}

fn native() -> cosmograph::simulation::Genesis {
    Founding {
        seed: 77,
        sites: 3,
        population: 24,
        cohort_size: 8,
        lineages: 2,
        ..Founding::default()
    }
    .generate()
    .unwrap()
}

#[test]
fn native_cohort_spans_do_not_turn_sparse_membership_into_an_interval() {
    let mut population = native().population;
    let (&first, cohort) = population
        .groups
        .iter()
        .find(|(_, group)| group.count > 2)
        .unwrap();
    let count = cohort.count;
    let mut rows = SparseStore::new();
    rows.set(first, count);
    assert_eq!(rows.get(first), Some(&count));
    assert!(population.get(first + 1).is_some());
    assert!(rows.get(first + 1).is_none());

    // Native splitting retains each identity. A derived index explicitly adopts
    // the new stored spans; the store never infers or allocates their members.
    population
        .lift(first + 1)
        .unwrap()
        .accounts
        .insert("world:soil".into(), 99);
    let mut split = SparseStore::new();
    for (&id, group) in &population.groups {
        split.set(id, group.count);
    }
    assert_eq!(split.get(first), Some(&1));
    assert_eq!(split.get(first + 1), Some(&1));
    assert_eq!(split.get(first + 2), Some(&(count - 2)));
    assert_eq!(
        rows.iter()
            .map(|(id, count)| (id, *count))
            .collect::<Vec<_>>(),
        vec![(first, count)]
    );
}

fn columns(session: &Session) -> (SparseStore<(String, bool, u64)>, SparseStore<Id>) {
    let (mut members, mut places) = (SparseStore::new(), SparseStore::new());
    for (&id, group) in &session.sim.state().population.groups {
        let body = &group.entity;
        members.set(id, (body.lineage.clone(), body.alive, group.count));
        if let Some(site) = body.resident_site() {
            places.set(id, site);
        }
    }
    (members, places)
}

#[test]
fn indexing_a_native_session_leaves_save_hash_history_and_replay_unchanged() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut indexed = Session::new(native(), mode).unwrap();
        indexed.command(Command::Inspect(5)).unwrap();
        indexed.advance(3).unwrap();
        let before = indexed.save();
        let before_bytes = serde_json::to_vec(&before).unwrap();
        let before_hash = indexed.sim.state_hash();
        let before_entries = indexed.entries.clone();
        let (members, places) = columns(&indexed);
        let joined: Vec<_> = intersection(&members, &places)
            .map(|(id, member, site)| (id, member.clone(), *site))
            .collect();
        assert!(!joined.is_empty());
        for (id, (lineage, alive, count), site) in &joined {
            let group = &indexed.sim.state().population.groups[id];
            assert_eq!(
                (lineage, *alive, *count, *site),
                (
                    &group.entity.lineage,
                    group.entity.alive,
                    group.count,
                    group.entity.place
                )
            );
        }
        let mut presentation = members.clone();
        let first = joined[0].0;
        assert!(presentation.remove(first));
        presentation.set(u64::MAX, ("presentation only".into(), false, 0));
        assert!(members.get(first).is_some());
        assert!(indexed.sim.state().population.get(u64::MAX).is_none());
        assert_eq!(indexed.sim.state_hash(), before_hash);
        assert_eq!(indexed.entries, before_entries);
        assert_eq!(serde_json::to_vec(&indexed.save()).unwrap(), before_bytes);

        let mut replayed = Session::load(before, mode).unwrap();
        let (replayed_members, replayed_places) = columns(&replayed);
        assert_eq!(
            intersection(&replayed_members, &replayed_places)
                .map(|(id, member, site)| (id, member.clone(), *site))
                .collect::<Vec<_>>(),
            joined
        );
        assert_eq!(replayed.sim.state_hash(), before_hash);
        let fork = indexed
            .fork_at(indexed.sim.state().tick, "branch:store-test".into())
            .unwrap();
        let (fork_members, fork_places) = columns(&fork);
        assert_eq!(
            intersection(&fork_members, &fork_places)
                .map(|(id, member, site)| (id, member.clone(), *site))
                .collect::<Vec<_>>(),
            joined
        );
        assert_eq!(serde_json::to_vec(&indexed.save()).unwrap(), before_bytes);
        indexed.advance(4).unwrap();
        replayed.advance(4).unwrap();
        assert_eq!(indexed.save(), replayed.save());
    }
}
