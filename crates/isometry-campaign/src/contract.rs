// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The VTT's assertions through its contract (ruling 795): the table's
//! vocabulary, `isocosm_overlay::vtt::Assertion`, lowered onto native
//! `isocosm::asserted` entries in this one place. The campaign folds the
//! entries (768), and a native session replays them when the sim switches
//! on (248). Handles name things at the table; entries carry authored keys.

use std::collections::BTreeSet;

use isocosm::Simulation;
use isocosm::asserted as native;
use crate::{CampaignWorld, WorldEvent};
use isocosm_overlay::Pointable;
use isocosm_overlay::vtt::Assertion;

/// Why a contract assertion did not reach the campaign.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// A handle names nothing asserted or drawn.
    Unnamed(Pointable),
    /// More than one distinct authored key has this handle.
    Ambiguous(Pointable),
    /// The campaign's fold refused the entry.
    Fold(String),
}

/// What a handle names, by authored key.
pub trait Keys {
    fn key(&self, of: Pointable) -> Option<String>;

    fn candidates(&self, of: Pointable) -> BTreeSet<String> {
        self.key(of).into_iter().collect()
    }
}

/// A sim-off campaign's handle for an authored key: its FNV-1a hash, so a
/// table needs no registry. With the sim on, entity, place and faction
/// handles are native ids; event handles always derive from native keys (807).
pub fn handle(key: &str) -> u64 {
    isocosm_overlay::key_handle(key)
}

/// A sim-off campaign names what its entries assert.
impl Keys for native::Asserted {
    fn key(&self, of: Pointable) -> Option<String> {
        let candidates = self.candidates(of);
        (candidates.len() == 1).then(|| candidates.into_iter().next().expect("one candidate"))
    }

    fn candidates(&self, of: Pointable) -> BTreeSet<String> {
        fn matching<'a>(keys: impl Iterator<Item = &'a String>, h: u64) -> BTreeSet<String> {
            keys.filter(|key| handle(key) == h).cloned().collect()
        }
        match of {
            Pointable::Entity(h) => matching(self.characters.keys(), h.0),
            Pointable::Place(h) => matching(self.places.keys(), h.0),
            Pointable::Faction(h) => matching(self.factions.keys(), h.0),
            Pointable::Event(h) => self
                .history
                .iter()
                .map(native::HistoryLine::event_key)
                .filter(|key| handle(key) == h.0)
                .collect(),
            _ => BTreeSet::new(),
        }
    }
}

/// With the sim on, a handle is a native id: its authored key where it
/// has one, its drawn name otherwise.
impl Keys for Simulation {
    fn key(&self, of: Pointable) -> Option<String> {
        let s = self.state();
        match of {
            Pointable::Entity(h) => s.population.get(h.0).map(|e| match &e.authored {
                Some(c) => c.key.clone(),
                None => format!("entity:{}", h.0),
            }),
            Pointable::Place(h) => s.sites.get(&h.0).map(|site| match &site.authored {
                Some(p) => p.key.clone(),
                None => format!("site:{}", h.0),
            }),
            Pointable::Faction(h) => s.polities.get(&h.0).map(|p| match &p.authored {
                Some(a) => a.key.clone(),
                None => format!("polity:{}", h.0),
            }),
            Pointable::Event(h) => {
                isocosm_overlay::key_for_handle(s.events.keys().map(String::as_str), h.0)
                    .map(str::to_owned)
            },
            _ => None,
        }
    }

    fn candidates(&self, of: Pointable) -> BTreeSet<String> {
        if let Pointable::Event(h) = of {
            return self
                .state()
                .events
                .keys()
                .filter(|key| handle(key) == h.0)
                .cloned()
                .collect();
        }
        self.key(of).into_iter().collect()
    }
}

fn named(keys: &dyn Keys, of: Pointable) -> Result<String, Refusal> {
    let candidates = keys.candidates(of);
    match candidates.len() {
        0 => Err(Refusal::Unnamed(of)),
        1 => Ok(candidates.into_iter().next().expect("one candidate")),
        _ => Err(Refusal::Ambiguous(of)),
    }
}

/// Lowers one contract assertion onto its native entry. Character cell and
/// owner are retained as table metadata (809); native placement and control
/// remain V2's work (799, 152).
pub fn lower(a: &Assertion, keys: &dyn Keys) -> Result<native::Assertion, Refusal> {
    Ok(match a {
        Assertion::Fact(f) => native::Assertion::Fact(native::Fact {
            about: f
                .about
                .map(|p| named(keys, p))
                .transpose()?
                .unwrap_or_default(),
            key: f.key.clone(),
            kind: f.kind.clone(),
            text: f.text.clone(),
            tags: f.tags.iter().cloned().collect::<BTreeSet<_>>(),
        }),
        Assertion::Edit(m) => native::Assertion::Edit(native::MapEdit {
            key: m.key.clone(),
            place: named(keys, Pointable::Place(m.site))?,
            cells: m
                .cells
                .iter()
                .map(|c| native::CellEdit {
                    at: c.at.0,
                    kind: c.kind.clone(),
                    height: c.height,
                })
                .collect(),
        }),
        Assertion::Character(c) => native::Assertion::Character(native::Character {
            key: c.key.clone(),
            name: c.name.clone(),
            tags: BTreeSet::new(),
            faction: c
                .faction
                .map(|f| named(keys, Pointable::Faction(f)))
                .transpose()?,
            place: Some(named(keys, Pointable::Place(c.at.site))?),
            table: Some(native::CharacterTable {
                cell: c.at.at.0,
                owner: c.owner.map(|owner| owner.0),
            }),
        }),
        Assertion::Storylet {
            key,
            storylet,
            asserts,
        } => native::Assertion::Storylet(group(key, storylet, asserts, keys)?),
        Assertion::PackForced { key, pack, asserts } => {
            native::Assertion::PackForced(group(key, pack, asserts, keys)?)
        },
    })
}

fn group(
    key: &str,
    source: &str,
    asserts: &[Assertion],
    keys: &dyn Keys,
) -> Result<native::Applied, Refusal> {
    struct Staged<'a> {
        base: &'a dyn Keys,
        characters: BTreeSet<String>,
    }
    impl Keys for Staged<'_> {
        fn key(&self, of: Pointable) -> Option<String> {
            let candidates = self.candidates(of);
            (candidates.len() == 1).then(|| candidates.into_iter().next().expect("one candidate"))
        }

        fn candidates(&self, of: Pointable) -> BTreeSet<String> {
            let mut candidates = self.base.candidates(of);
            if let Pointable::Entity(h) = of {
                candidates.extend(
                    self.characters
                        .iter()
                        .filter(|key| handle(key) == h.0)
                        .cloned(),
                );
            }
            candidates
        }
    }
    fn remember(a: &native::Assertion, characters: &mut BTreeSet<String>) {
        match a {
            native::Assertion::Character(c) => {
                characters.insert(c.key.clone());
            },
            native::Assertion::Storylet(a) | native::Assertion::PackForced(a) => {
                for a in &a.asserts {
                    remember(a, characters);
                }
            },
            _ => {},
        }
    }
    let mut staged = Staged {
        base: keys,
        characters: BTreeSet::new(),
    };
    let mut entries = Vec::with_capacity(asserts.len());
    for a in asserts {
        let entry = lower(a, &staged)?;
        remember(&entry, &mut staged.characters);
        entries.push(entry);
    }
    Ok(native::Applied {
        key: key.into(),
        source: source.into(),
        asserts: entries,
    })
}

/// Lowers `a` against the campaign's own fold and checks it folds, without
/// changing `world`: the event to commit, which every peer folds (768).
pub fn translate(world: &CampaignWorld, a: &Assertion) -> Result<WorldEvent, Refusal> {
    let entry = lower(a, world.asserted())?;
    let mut preview = world.clone();
    preview
        .assert(entry.clone())
        .map_err(|e| Refusal::Fold(format!("{e:?}")))?;
    Ok(WorldEvent::Assert(entry))
}

#[cfg(test)]
mod tests {
    use super::*;
    use isocosm_overlay::vtt::{Cell, Fact, NewCharacter};
    use isocosm_overlay::{EntityHandle, ParticipantHandle, PlaceHandle, WorldPoint};

    fn world() -> CampaignWorld {
        let mut world = CampaignWorld::default();
        let ford = native::Place {
            key: "ford".into(),
            name: "Ford".into(),
            ..Default::default()
        };
        world.assert(native::Assertion::Place(ford)).unwrap();
        world
    }

    fn mira(site: u64) -> Assertion {
        Assertion::Character(NewCharacter {
            key: "mira".into(),
            name: "Mira".into(),
            faction: None,
            at: Cell {
                site: PlaceHandle(site),
                at: WorldPoint([-7, 2, 13]),
            },
            owner: Some(ParticipantHandle(77)),
        })
    }

    #[test]
    fn a_contract_character_folds_by_its_key() {
        let mut world = world();
        let event = translate(&world, &mira(handle("ford"))).unwrap();
        world.apply(&event).unwrap();
        assert_eq!(world.characters()["mira"].place.as_deref(), Some("ford"));
        assert_eq!(
            world.characters()["mira"].table,
            Some(native::CharacterTable {
                cell: [-7, 2, 13],
                owner: Some(77)
            })
        );
        let json = serde_json::to_vec(&world).unwrap();
        let saved: CampaignWorld = serde_json::from_slice(&json).unwrap();
        assert_eq!(saved, world, "authored metadata survives campaign save");
        let bytes = postcard::to_allocvec(&world).unwrap();
        let wired: CampaignWorld = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(wired, world, "the wire retains the same assertion");
        let before = world.clone();
        world.apply(&event).unwrap();
        assert_eq!(world, before, "the same assertion replays once");
        for changed_owner in [false, true] {
            let mut changed = mira(handle("ford"));
            let Assertion::Character(c) = &mut changed else {
                unreachable!()
            };
            if changed_owner {
                c.owner = Some(ParticipantHandle(78));
            } else {
                c.at.at = WorldPoint([8, 2, 13]);
            }
            assert!(matches!(translate(&world, &changed), Err(Refusal::Fold(_))));
            assert_eq!(world, before, "metadata conflicts cannot alter the fold");
        }
        let unnamed = Pointable::Place(PlaceHandle(7));
        assert_eq!(translate(&world, &mira(7)), Err(Refusal::Unnamed(unnamed)));
    }

    #[test]
    fn a_storylet_folds_whole_through_the_contract() {
        let mut world = world();
        let storylet = Assertion::Storylet {
            key: "arrival-1".into(),
            storylet: "arrival".into(),
            asserts: vec![mira(handle("ford"))],
        };
        world.apply(&translate(&world, &storylet).unwrap()).unwrap();
        assert!(world.asserted().records.storylets.contains_key("arrival-1"));
        let fact = Assertion::Fact(Fact {
            key: "met-mira".into(),
            kind: "narration".into(),
            about: Some(Pointable::Entity(EntityHandle(handle("mira")))),
            text: "Mira waits at the ford.".into(),
            tags: vec![],
        });
        world.apply(&translate(&world, &fact).unwrap()).unwrap();
        assert_eq!(world.asserted().facts["met-mira"].about, "mira");
    }

    #[test]
    fn nested_groups_name_earlier_characters_and_replay_once() {
        let mut world = world();
        let fact = Assertion::Fact(Fact {
            key: "met-mira".into(),
            kind: "narration".into(),
            about: Some(Pointable::Entity(EntityHandle(handle("mira")))),
            text: "Mira waits.".into(),
            tags: vec![],
        });
        let pack = Assertion::PackForced {
            key: "p".into(),
            pack: "arrival-pack".into(),
            asserts: vec![Assertion::Storylet {
                key: "s".into(),
                storylet: "arrival".into(),
                asserts: vec![mira(handle("ford")), fact],
            }],
        };
        let event = translate(&world, &pack).unwrap();
        world.apply(&event).unwrap();
        assert_eq!(world.asserted().facts["met-mira"].about, "mira");
        let before = world.clone();
        world.apply(&event).unwrap();
        assert_eq!(world, before);
    }

    #[test]
    fn a_group_conflict_never_commits_its_earlier_children() {
        let world = world();
        let mut changed = mira(handle("ford"));
        if let Assertion::Character(c) = &mut changed {
            c.name = "Another Mira".into();
        }
        let group = Assertion::Storylet {
            key: "bad".into(),
            storylet: "conflict".into(),
            asserts: vec![mira(handle("ford")), changed],
        };
        let before = world.clone();
        assert!(matches!(translate(&world, &group), Err(Refusal::Fold(_))));
        assert_eq!(world, before);
    }

    #[test]
    fn a_map_and_a_fact_keep_the_tables_values() {
        use isocosm_overlay::vtt::{CellEdit, MapEdit};
        let mut world = world();
        let map = Assertion::Edit(MapEdit {
            key: "map:ford".into(),
            site: PlaceHandle(handle("ford")),
            cells: vec![CellEdit {
                at: WorldPoint([1, 2, 3]),
                kind: "stone".into(),
                height: 4,
            }],
        });
        world.apply(&translate(&world, &map).unwrap()).unwrap();
        let held = &world.asserted().records.maps["map:ford"];
        assert_eq!(held.place, "ford");
        assert_eq!(
            held.cells[0],
            native::CellEdit {
                at: [1, 2, 3],
                kind: "stone".into(),
                height: 4
            }
        );
        let fact = Assertion::Fact(Fact {
            key: "weather".into(),
            kind: "narration".into(),
            about: None,
            text: "Raining.".into(),
            tags: vec!["wet".into()],
        });
        world.apply(&translate(&world, &fact).unwrap()).unwrap();
        assert_eq!(
            world.asserted().facts["weather"].tags,
            BTreeSet::from(["wet".into()])
        );
    }

    #[test]
    fn native_event_facts_use_the_shared_event_key_mapping() {
        let mut session = isocosm::Session::new(
            isocosm::Founding::default().generate().unwrap(),
            isocosm::Execution::Individuals,
        )
        .unwrap();
        let history = native::Assertion::History(native::HistoryLine {
            key: "arrival".into(),
            ..Default::default()
        });
        session
            .command(isocosm::Command::Assert(history.clone()))
            .unwrap();
        let fact = Assertion::Fact(Fact {
            key: "tale".into(),
            kind: "narration".into(),
            text: "Someone arrived.".into(),
            tags: vec![],
            about: Some(Pointable::Event(isocosm_overlay::EventHandle(
                isocosm_overlay::key_handle("line:arrival"),
            ))),
        });
        let native::Assertion::Fact(lowered) = lower(&fact, &session.sim).unwrap() else {
            panic!("a fact")
        };
        assert_eq!(lowered.about, "line:arrival");
        let asserted = native::Asserted::fold([&history]).unwrap();
        assert_eq!(lower(&fact, &asserted), lower(&fact, &session.sim));
    }

    #[test]
    fn an_ambiguous_handle_never_selects_the_first_key() {
        struct Collision;
        impl Keys for Collision {
            fn key(&self, _: Pointable) -> Option<String> {
                None
            }
            fn candidates(&self, _: Pointable) -> BTreeSet<String> {
                BTreeSet::from(["line:one".into(), "line:two".into()])
            }
        }
        let of = Pointable::Event(isocosm_overlay::EventHandle(7));
        let fact = Assertion::Fact(Fact {
            key: "f".into(),
            kind: "narration".into(),
            text: "One event.".into(),
            tags: vec![],
            about: Some(of),
        });
        assert_eq!(lower(&fact, &Collision), Err(Refusal::Ambiguous(of)));
    }
}
