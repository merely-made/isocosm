// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Checkpoint 8, step 8d: births spend the provision (rulings 447, 448,
//! 518, 521, 524 and 530). A brood develops the whole recipe, a clutch
//! lays eggs that are the recipe's root alone, and a bud fills from the
//! provision at the reproducing part and severs once full; each child
//! draws its soma by its own seed, and every move reconciles in the flow
//! record, to the child's parts.

use isocosm::{
    Command, Execution, Session, anatomy,
    development::{Soma, develop, soma},
    flows::{Flow, Holder},
    probe::BodyFounding,
    rules::*,
    schema::*,
    simulation::Genesis,
};
use std::collections::{BTreeMap, BTreeSet};

const SOIL: &str = "world:soil";
const BUD: &str = "part:bud";

fn template(half_extent: [i32; 3], cells: &[(&str, u32)]) -> Template {
    Template {
        half_extent,
        cells: cells
            .iter()
            .map(|(f, n)| (format!("function:{f}"), *n))
            .collect(),
        shape: String::new(),
    }
}

fn development(clutch: u32) -> Development {
    let recipe = Recipe {
        tagmata: vec![Tagma {
            segments: 1,
            segment: "kind:lump".into(),
            bears: Some("kind:limb".into()),
            per_segment: 1,
            parent: None,
            anchor: Anchor::Tip,
            facing: Facing::Back,
            socket: Facing::Right,
            variance: None,
        }],
        variance: 0,
        absence: [0, 1],
    };
    Development {
        lexicon: recipe.kinds(),
        recipe,
        policy: Policy::default(),
        domain: 0,
        clutch,
        anamorphic: false,
    }
}

fn act(id: &str, effects: Vec<Effect>) -> Process {
    Process {
        id: id.into(),
        causation: Causation::Choice,
        requires: vec![Query::Alive(Binding::Actor)],
        commitments: vec![],
        effects,
        risk: None,
        target: None,
        period: None,
        priority: 0,
        need_account: None,
        need_below: 0,
        glyphs: BTreeSet::new(),
        invariants: BTreeSet::new(),
        note: false,
    }
}

/// The probe's world, its first grazer replaced by a lump bearing a pair
/// of limbs, its parts full and its provision `provision`.
fn world(clutch: u32, provision: u64) -> (Genesis, Id) {
    let mut g = BodyFounding::default().generate().unwrap().genesis;
    g.rules.kinds = BTreeMap::from([
        (
            "kind:lump".into(),
            template([2, 2, 2], &[("intake", 5), ("store", 1), ("reproduce", 1)]),
        ),
        ("kind:limb".into(), template([3, 1, 1], &[("contract", 2)])),
    ]);
    g.rules.accounts.insert(
        "provision:1".into(),
        AccountKind::Matter {
            lineage: "lineage:1".into(),
            reserve: false,
            provision: true,
        },
    );
    let d = development(clutch);
    g.lineages.get_mut("lineage:1").unwrap().development = Some(d.clone());
    let feed = vec![
        Effect::Transfer {
            from: Binding::Place,
            to: Binding::Actor,
            account: SOIL.into(),
            amount: Amount::Fixed(12),
        },
        Effect::Convert {
            who: Binding::Actor,
            from: vec![SOIL.into()],
            to: "provision:1".into(),
            amount: Amount::Fixed(12),
            conversion: Conversion::Synthesis,
        },
    ];
    for p in [
        act("test:brood", vec![Effect::Bear { clutch: false }]),
        act("test:lay", vec![Effect::Bear { clutch: true }]),
        act(
            "test:bud",
            vec![Effect::Bud {
                mark: BUD.into(),
                once: false,
            }],
        ),
        act(
            "test:bud-once",
            vec![Effect::Bud {
                mark: BUD.into(),
                once: true,
            }],
        ),
        act("test:feed", feed),
    ] {
        g.rules.processes.insert(p.id.clone(), p);
    }
    g.rules.traits.insert(BUD.into());
    // Room for the children; the probe's world holds its founders alone.
    g.rules.limits.entities += 64;
    let groups = g.population.groups.iter();
    let id = *groups
        .filter(|(_, c)| c.entity.lineage == "lineage:1")
        .map(|(f, _)| f)
        .next()
        .unwrap();
    let e = g.population.lift(id).unwrap();
    let soma = Soma {
        segments: vec![1],
        absent: vec![],
    };
    e.parts = develop(&g.rules, &d, &soma).unwrap();
    e.soma = vec![1];
    e.accounts.clear();
    let rules = g.rules.clone();
    let room = anatomy::room(e, &rules, "tissue:1");
    anatomy::give(e, &rules, "tissue:1", room).unwrap().unwrap();
    anatomy::give(e, &rules, "provision:1", provision)
        .unwrap()
        .unwrap();
    g.sites
        .get_mut(&0)
        .unwrap()
        .accounts
        .insert(SOIL.into(), 10_000);
    (g, id)
}

type Books = BTreeMap<(Holder, Key), i128>;

/// Every matter account of every member, its parts, and every site.
fn books(s: &Session) -> Books {
    let (rules, state) = (&s.sim.genesis().rules, s.sim.state());
    let matter = |k: &Key| matches!(rules.accounts.get(k), Some(AccountKind::Matter { .. }));
    let mut books = Books::new();
    let mut hold = |h: Holder, l: &Ledger| {
        for (k, v) in l.iter().filter(|(k, v)| matter(k) && **v > 0) {
            *books.entry((h, k.clone())).or_default() += i128::from(*v);
        }
    };
    for (&first, c) in &state.population.groups {
        for id in first..first + c.count {
            hold(Holder::Entity(id), &c.entity.accounts);
            for (&part, p) in &c.entity.parts {
                hold(Holder::Part(id, part), &p.matter);
            }
        }
    }
    for (&id, site) in &state.sites {
        hold(Holder::Site(id), &site.accounts);
    }
    books
}

/// Runs `process` for `actor` and checks the record accounts for every
/// ledger's change.
fn run(s: &mut Session, actor: Id, process: &str) -> Vec<Flow> {
    let before = books(s);
    let command = Command::Act {
        actor,
        target: None,
        process: process.into(),
        cause: None,
    };
    let record = s.command_with_flows(command).unwrap();
    let mut claimed = Books::new();
    for f in &record.flows {
        for (holder, sign) in [(f.from.0, -1), (f.to.0, 1)] {
            let amount = sign * i128::from(f.amount);
            let key = |h| {
                (
                    h,
                    if sign < 0 {
                        f.from.1.clone()
                    } else {
                        f.to.1.clone()
                    },
                )
            };
            match holder {
                Holder::Entity(first) => (first..first + f.count)
                    .for_each(|id| *claimed.entry(key(Holder::Entity(id))).or_default() += amount),
                Holder::Part(first, part) => (first..first + f.count).for_each(|id| {
                    *claimed.entry(key(Holder::Part(id, part))).or_default() += amount
                }),
                Holder::Site(_) => {
                    *claimed.entry(key(holder)).or_default() += amount * i128::from(f.count)
                },
                Holder::Dev => {},
            }
        }
    }
    let after = books(s);
    let keys: BTreeSet<_> = before
        .keys()
        .chain(after.keys())
        .chain(claimed.keys())
        .collect();
    for k in keys {
        let delta = after.get(k).copied().unwrap_or(0) - before.get(k).copied().unwrap_or(0);
        assert_eq!(
            delta,
            claimed.get(k).copied().unwrap_or(0),
            "{process}: {k:?}"
        );
    }
    record.flows
}

fn session(g: Genesis) -> Session {
    Session::new(g, Execution::Individuals).unwrap()
}

fn children(s: &Session, of: Id) -> Vec<(Id, Entity)> {
    let state = s.sim.state();
    let born = state
        .relations
        .iter()
        .filter(|r| r.kind == "sim:parent" && r.object == of);
    born.map(|r| (r.subject, state.population.get(r.subject).unwrap().clone()))
        .collect()
}

fn held(s: &Session, e: &Entity, key: &str) -> u64 {
    anatomy::held(e, &s.sim.genesis().rules, key)
}

#[test]
fn a_brood_develops_the_whole_recipe_from_the_whole_provision() {
    let (g, id) = world(3, 12);
    let mut s = session(g);
    let before = s.sim.matter();
    let flows = run(&mut s, id, "test:brood");
    let kids = children(&s, id);
    assert_eq!(kids.len(), 1);
    let (kid, child) = &kids[0];
    assert_eq!(child.parts.len(), 3, "a lump and its pair of limbs");
    assert_eq!(held(&s, child, "tissue:1"), 12);
    let parent = s.sim.state().population.get(id).unwrap();
    assert_eq!(held(&s, parent, "provision:1"), 0);
    assert_eq!(s.sim.matter(), before);
    // The record names the parts the provision left and reached.
    let reached = |h: &Holder| matches!(h, Holder::Part(c, _) if c == kid);
    assert!(flows.iter().any(|f| reached(&f.to.0)));
    assert!(
        flows
            .iter()
            .any(|f| matches!(f.from.0, Holder::Part(p, _) if p == id))
    );
}

#[test]
fn a_clutch_splits_the_provision_among_eggs_each_the_root_alone() {
    let (g, id) = world(5, 12);
    let mut s = session(g);
    run(&mut s, id, "test:lay");
    let kids = children(&s, id);
    let shares: Vec<u64> = kids.iter().map(|(_, e)| held(&s, e, "tissue:1")).collect();
    assert_eq!(shares, [3, 3, 2, 2, 2]);
    for (_, egg) in &kids {
        assert_eq!(egg.parts.len(), 1);
        assert_eq!(egg.parts[&0].situs, Some([0, 0, 0]));
        assert_eq!(egg.soma, vec![1], "it keeps the soma it drew");
    }
}

#[test]
fn each_child_draws_its_soma_by_its_own_seed() {
    let (g, id) = world(2, 12);
    let rules = g.rules.clone();
    let dynamics = g.dynamics_seed();
    let mut s = session(g);
    run(&mut s, id, "test:lay");
    for (kid, egg) in children(&s, id) {
        let seed = isocosm::draw(dynamics, "soma", &[kid]);
        assert_eq!(
            egg.soma,
            soma(&rules, &development(2).recipe, seed).segments
        );
    }
    // The control: the same world bears the same children.
    let (g, id) = world(2, 12);
    let mut again = session(g);
    run(&mut again, id, "test:lay");
    assert_eq!(children(&again, id), children(&s, id));
}

#[test]
fn nothing_provisioned_bears_nothing() {
    let (g, id) = world(3, 0);
    let mut s = session(g);
    for p in ["test:brood", "test:lay", "test:bud"] {
        run(&mut s, id, p);
        assert!(children(&s, id).is_empty(), "{p}");
    }
}

#[test]
fn a_bud_fills_from_the_provision_and_severs_once_full() {
    let (g, id) = world(1, 12);
    let mut s = session(g);
    let before = s.sim.matter();
    run(&mut s, id, "test:bud");
    let parent = s.sim.state().population.get(id).unwrap();
    let bud = parent
        .parts
        .values()
        .find(|p| p.traits.contains(BUD))
        .unwrap();
    assert_eq!(bud.matter["tissue:1"], 12, "the provision poured into it");
    assert_eq!(parent.parts.len(), 4);
    // A lump of 100 mg fills at 12 mg a provision: severed on the ninth.
    let mut rounds = 1;
    while children(&s, id).is_empty() {
        assert!(rounds < 20, "the bud never severed");
        run(&mut s, id, "test:feed");
        run(&mut s, id, "test:bud");
        rounds += 1;
    }
    // The ninth pours the 4 mg the bud has room for and keeps 8.
    assert_eq!(rounds, 9);
    let (_, child) = &children(&s, id)[0];
    assert_eq!(child.parts.len(), 1);
    let root = &child.parts[&0];
    assert!(!root.traits.contains(BUD));
    assert_eq!(
        (root.parent, root.offset, root.situs),
        (None, [0; 3], Some([0, 0, 0]))
    );
    assert!(held(&s, child, "tissue:1") >= 100);
    let parent = s.sim.state().population.get(id).unwrap();
    assert_eq!(parent.parts.len(), 3, "the bud left");
    assert!(parent.alive);
    // Fed from the site, so the world's matter is the site's to account.
    assert_eq!(s.sim.matter(), before);
}

#[test]
fn a_semelparous_bud_dies_as_it_severs() {
    let (g, id) = world(1, 12);
    let mut s = session(g);
    for round in 0.. {
        assert!(round < 20, "the bud never severed");
        run(&mut s, id, "test:bud-once");
        let parent = s.sim.state().population.get(id).unwrap();
        assert_eq!(parent.alive, children(&s, id).is_empty());
        if !parent.alive {
            break;
        }
        run(&mut s, id, "test:feed");
    }
}
