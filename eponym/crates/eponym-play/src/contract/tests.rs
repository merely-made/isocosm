// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

mod borrowed;
use crate::{WorldConfig, founding, vocabulary};
use cosmograph::arrival::Arrival;
use cosmograph::schema::Method;
use cosmograph_overlay::ActKey;
use cosmograph_overlay::eponym::{
    Blow, Deed, DeedKey, FirstLife, PlayerAct, PlayerActKind, StartTime, TimedAct,
};

fn peer(world: &mut World) -> Id {
    let arrival = Arrival {
        lineage: founding::SOPHONT.into(),
        site: world.site(),
        method: Method::Normative,
        accounts: [
            (founding::TISSUE.into(), 16),
            (founding::RESERVE.into(), founding::FULL),
            (founding::ENERGY.into(), founding::FULL),
        ]
        .into(),
        traits: Default::default(),
        skills: [(vocabulary::SCOUTING.into(), 5)].into(),
        disposition: [0; 5],
        character: None,
    };
    let outcome = world.found(Command::Arrive(arrival)).unwrap();
    let entity = outcome.strip_prefix("entity:").unwrap().parse().unwrap();
    let body = isometer_core::BodyDocument::new(isometer_core::VolumeRef::from_tag(0), [2, 4, 2]);
    world.found(Command::Embody { entity, body }).unwrap();
    entity
}

fn world() -> (World, Id, Id) {
    let mut world = World::generate(7, WorldConfig { side: 2, extent: 8 }).unwrap();
    let a = peer(&mut world);
    let b = peer(&mut world);
    (world, a, b)
}

#[test]
fn a_participant_drives_only_the_life_it_took() {
    let (mut world, a, b) = world();
    let hash = world.state_hash().unwrap();
    let earlier = FirstLife {
        start: StartTime::Habitability,
        begins_as: LifeChoice::Denizen(EntityHandle(a)),
    };
    assert!(matches!(
        world.submit(
            None,
            &EponymIntent::Checkpoint(LifeCheckpoint::First(earlier))
        ),
        Err(WorldError::Unbuilt(_))
    ));
    assert_eq!(world.state_hash().unwrap(), hash);
    let first = FirstLife {
        start: StartTime::Society,
        begins_as: LifeChoice::Denizen(EntityHandle(a)),
    };
    let lowered = world
        .submit(
            None,
            &EponymIntent::Checkpoint(LifeCheckpoint::First(first)),
        )
        .unwrap();
    let participant = lowered.joined.unwrap();
    assert_eq!(world.sim().plays(participant), Some(a));
    let drive = |body| {
        EponymIntent::Drive(Actuation {
            body: EntityHandle(body),
            motion: None,
            acts: vec![TimedAct {
                act: ActKey(founding::FATIGUE.into()),
                target: None,
            }],
        })
    };
    let before = world.sim().state().population.get(a).unwrap().accounts[founding::ENERGY];
    world.submit(Some(participant), &drive(a)).unwrap();
    assert_eq!(
        world.sim().state().population.get(a).unwrap().accounts[founding::ENERGY],
        before - 1
    );
    let hash = world.state_hash().unwrap();
    assert_eq!(
        world.submit(Some(participant), &drive(b)),
        Err(WorldError::NotPlayed)
    );
    assert_eq!(
        world.submit(Some(u64::MAX), &drive(a)),
        Err(WorldError::NotPlayed)
    );
    assert_eq!(world.state_hash().unwrap(), hash);
}

#[test]
fn game_naming_uses_the_played_doer_after_first_naming() {
    use crate::identity::SubjectId;
    use crate::{GameIntent, GameState, Name};
    let world = World::generate(7, WorldConfig { side: 2, extent: 8 }).unwrap();
    let top = world.ground().surface(0, 0).unwrap();
    let mut game = GameState::new(world);
    let (a, b) = (SubjectId(0), SubjectId(1));
    for subject in [a, b] {
        game.apply(GameIntent::Generate {
            tick: game.next_tick(),
            subject,
            body_seed: 7,
            at: [0, top + 1, 0],
        })
        .unwrap();
    }
    let name = |game: &mut GameState, subject, text| {
        game.apply(GameIntent::Name {
            tick: game.next_tick(),
            subject,
            name: Name::new(text).unwrap(),
        })
        .unwrap();
    };
    name(&mut game, a, "Mira");
    game.take_up(a).unwrap();
    name(&mut game, b, "Rowan");
    let (by, of) = (game.entity(a).unwrap(), game.entity(b).unwrap());
    assert!(matches!(
        &game.world().session().entries.last().unwrap().command,
        Command::Name { by: actor, of: named, .. } if *actor == by && *named == of
    ));
}

#[test]
fn a_refused_actuation_rolls_back_its_earlier_acts() {
    let (mut world, a, _) = world();
    let hash = world.state_hash().unwrap();
    let first = EponymIntent::Drive(Actuation {
        body: EntityHandle(a),
        motion: None,
        acts: vec![TimedAct {
            act: ActKey(founding::FATIGUE.into()),
            target: None,
        }],
    });
    let mut accepted = world.clone();
    accepted.submit(None, &first).unwrap();
    assert_ne!(accepted.state_hash().unwrap(), hash);
    let drive = EponymIntent::Drive(Actuation {
        body: EntityHandle(a),
        motion: None,
        acts: [founding::FATIGUE, "not-a-process"]
            .map(|key| TimedAct {
                act: ActKey(key.into()),
                target: None,
            })
            .into(),
    });
    assert!(world.submit(None, &drive).is_err());
    assert_eq!(world.state_hash().unwrap(), hash);
}

#[test]
fn anatomical_spending_keeps_the_native_outcome_through_the_adapter() {
    let (mut world, a, _) = world();
    let mut native = world.sim().clone();
    let receipt = native.execute(a, None, founding::HUNGER, None);
    let hash = world.state_hash().unwrap();
    let drive = EponymIntent::Drive(Actuation {
        body: EntityHandle(a),
        motion: None,
        acts: vec![TimedAct {
            act: ActKey(founding::HUNGER.into()),
            target: None,
        }],
    });
    if receipt.accepted() {
        let lowered = world.submit(None, &drive).unwrap();
        let adapted: cosmograph::simulation::Receipt =
            serde_json::from_str(&lowered.outcomes[0]).unwrap();
        assert_eq!(adapted, receipt);
        assert_eq!(world.sim().state_hash(), native.state_hash());
    } else {
        assert_eq!(world.submit(None, &drive), Err(WorldError::Refused));
        assert_eq!(world.state_hash().unwrap(), hash);
    }
}

#[test]
fn accepted_motion_names_a_native_patch() {
    use cosmograph_overlay::eponym::Motion;
    use cosmograph_overlay::{PlaceHandle, WorldPoint};
    let (mut world, a, _) = world();
    let top = world.ground().surface(0, 0).unwrap();
    let cell =
        crate::walking::to_site(world.volume(), [0, top + 1, 0]).map(|v| i32::try_from(v).unwrap());
    let motion = Motion {
        at: PlaceHandle(world.site()),
        position: WorldPoint(cell),
        fell: 0,
    };
    let drive = EponymIntent::Drive(Actuation {
        body: EntityHandle(a),
        motion: Some(motion),
        acts: vec![],
    });
    world.submit(None, &drive).unwrap();
    let patch = world
        .sim()
        .state()
        .population
        .get(a)
        .unwrap()
        .patch
        .unwrap();
    assert_eq!(patch.site, world.site());
    assert!(
        matches!(world.session().entries.last().unwrap().command, Command::Patch { entity, .. } if entity == a)
    );
}

#[test]
fn names_deeds_and_asks_reach_the_native_owners() {
    let (mut world, a, b) = world();
    let name = PlayerAct {
        subject: EntityHandle(a),
        kind: PlayerActKind::Name {
            of: Pointable::Entity(EntityHandle(a)),
            name: "Mira".into(),
        },
    };
    world.submit(None, &EponymIntent::Act(name)).unwrap();
    assert_eq!(world.sim().name_of(a), Some("Mira"));
    let deed = Deed {
        doer: EntityHandle(a),
        toward: Some(EntityHandle(b)),
        kind: DeedKey(vocabulary::STOOD_BY.into()),
        under: None,
    };
    world.submit(None, &EponymIntent::Deed(deed)).unwrap();
    assert_eq!(world.sim().deeds().last().unwrap().doer, a);
    let offer = Offer::new(
        a,
        b,
        social::Work::new(vocabulary::SCOUTING, 1, 0),
        social::Terms::new(4, 5),
    );
    let out = world
        .submit(None, &EponymIntent::Ask(proposal(&offer, None)))
        .unwrap();
    assert!(matches!(
        serde_json::from_str::<social::Answer>(&out.outcomes[0]).unwrap(),
        social::Answer::Response(_)
    ));
    world
        .submit(
            None,
            &EponymIntent::Agree(AgreementAct::Form(proposal(&offer, None))),
        )
        .unwrap();
    let agreement = *world.sim().state().agreements.keys().next().unwrap();
    world
        .submit(None, &EponymIntent::Ask(proposal(&offer, Some(agreement))))
        .unwrap();
    assert_eq!(world.sim().state().agreements[&agreement].exercises(), 1);
    let mut impostor = proposal(&offer, Some(agreement));
    impostor.from = EntityHandle(b);
    let hash = world.state_hash().unwrap();
    assert_eq!(
        world.submit(None, &EponymIntent::Ask(impostor)),
        Err(WorldError::Refused)
    );
    assert_eq!(world.state_hash().unwrap(), hash);
    let changed = social::Terms::new(5, 5);
    world
        .submit(
            None,
            &EponymIntent::Agree(AgreementAct::Renegotiate {
                agreement: AgreementHandle(agreement),
                by: EntityHandle(b),
                terms: agreement_terms(&changed),
            }),
        )
        .unwrap();
    assert_eq!(world.sim().state().agreements[&agreement].terms, changed);
    world
        .submit(
            None,
            &EponymIntent::Agree(AgreementAct::End {
                agreement: AgreementHandle(agreement),
                by: EntityHandle(a),
                why: EndReason::WorkDone,
            }),
        )
        .unwrap();
    assert!(!world.sim().state().agreements[&agreement].standing());
    for deed in world
        .sim()
        .deeds()
        .iter()
        .filter(|d| d.kind.agreement.is_some())
    {
        assert_eq!(deed.kind.agreement, Some(agreement));
        let event = &world.sim().state().events[&deed.id];
        assert_eq!(event.agreement, Some(agreement));
        assert_eq!(event.cause, None);
    }
    assert_eq!(world.sim().state().agreements[&agreement].history.len(), 4);
    let restored =
        cosmograph::Session::load(world.session().save(), cosmograph::Execution::Individuals).unwrap();
    assert_eq!(restored.sim.state_hash(), world.sim().state_hash());
    assert_eq!(restored.sim.deeds(), world.sim().deeds());

    let mut native = world.sim().clone();
    assert_eq!(
        native
            .execute(
                a,
                Some(b),
                vocabulary::STOOD_BY,
                Some(format!("agreement:{agreement}"))
            )
            .outcome,
        cosmograph::simulation::Outcome::Refused("unknown causal event".into())
    );
    let before = world.state_hash().unwrap();
    let unattached = Deed {
        doer: EntityHandle(a),
        toward: Some(EntityHandle(b)),
        kind: DeedKey(vocabulary::STOOD_BY.into()),
        under: Some(AgreementHandle(u64::MAX)),
    };
    assert_eq!(
        world.submit(None, &EponymIntent::Deed(unattached)),
        Err(WorldError::Refused)
    );
    assert_eq!(world.state_hash().unwrap(), before);
}

#[test]
fn proposal_terms_cannot_be_silently_dropped_or_clamped() {
    let (mut world, a, b) = world();
    let offer = Offer::new(
        a,
        b,
        social::Work::new(vocabulary::SCOUTING, 1, 0),
        social::Terms::new(4, 5),
    );
    let native = proposal(&offer, None);
    let hash = world.state_hash().unwrap();
    let mut custom = native.clone();
    custom.terms.push(Term {
        side: TermSide::Offered,
        what: "iron".into(),
        count: 1,
    });
    assert!(matches!(
        world.submit(None, &EponymIntent::Ask(custom)),
        Err(WorldError::Unbuilt(_))
    ));
    let mut overflow = native.clone();
    overflow.terms[0].count = 256;
    assert_eq!(
        world.submit(None, &EponymIntent::Ask(overflow)),
        Err(WorldError::Refused)
    );
    let mut duplicate = native;
    duplicate.terms.push(duplicate.terms[0].clone());
    assert_eq!(
        world.submit(None, &EponymIntent::Ask(duplicate)),
        Err(WorldError::Refused)
    );
    assert_eq!(world.state_hash().unwrap(), hash);
}

#[test]
fn a_resolved_blow_becomes_a_native_wound() {
    let (mut world, a, b) = world();
    let blow = EponymHandoff::Blow(Blow {
        by: EntityHandle(a),
        target: EntityHandle(b),
        act: ActKey("strike".into()),
        harm: Harm {
            vigour_drained: 1,
            wounds: vec![],
        },
    });
    let hash = world.state_hash().unwrap();
    assert_eq!(world.hand_back(a, &blow), Err(WorldError::Refused));
    assert_eq!(world.state_hash().unwrap(), hash);
    let out = world.hand_back(b, &blow).unwrap();
    assert_eq!(out.wounded.len(), 1);
    assert!(
        matches!(world.session().entries.last().unwrap().command, Command::Wound { entity, cells: 1, .. } if entity == b)
    );
}

#[test]
fn knowing_and_tellings_keep_the_native_claim_chain() {
    use cosmograph::knowing::Reading;
    use cosmograph_overlay::EventHandle;
    use cosmograph_overlay::eponym::{Claim, KnowingAct, Telling};
    let (mut world, a, b) = world();
    let deed = Deed {
        doer: EntityHandle(a),
        toward: Some(EntityHandle(b)),
        kind: DeedKey(vocabulary::STOOD_BY.into()),
        under: None,
    };
    let event = serde_json::from_str::<social::Answer>(
        &world
            .submit(None, &EponymIntent::Deed(deed))
            .unwrap()
            .outcomes[0],
    )
    .unwrap();
    let social::Answer::Deed(event) = event else {
        panic!("a deed")
    };
    let submit = |world: &mut World, subject, record| {
        let act = KnowingAct {
            subject: EntityHandle(subject),
            record: serde_json::to_vec(&record).unwrap(),
        };
        world.submit(None, &EponymIntent::Know(act))
    };
    let observe = Knowing::Observe {
        observer: a,
        event: event.clone(),
    };
    let observed = submit(&mut world, a, observe).unwrap().outcomes[0].clone();
    let proposition = Proposition {
        event,
        reading: Reading::Helped,
    };
    let record = Knowing::Claim {
        claimant: a,
        proposition: proposition.clone(),
        supports: vec![observed],
    };
    let claim = submit(&mut world, a, record).unwrap().outcomes[0].clone();
    let bytes =
        serde_json::to_vec(&serde_json::json!({ "claim": claim, "proposition": proposition }))
            .unwrap();
    let telling = Telling {
        from: EntityHandle(a),
        to: EntityHandle(b),
        claim: Claim {
            about: Pointable::Event(EventHandle(cosmograph_overlay::key_handle(&proposition.event))),
            as_told: bytes,
        },
        manner: Manner::Plain,
    };
    let mut mismatched = telling.clone();
    mismatched.claim.about = Pointable::Event(EventHandle(0));
    let before = world.state_hash().unwrap();
    assert_eq!(
        world.submit(None, &EponymIntent::Tell(mismatched)),
        Err(WorldError::Refused)
    );
    assert_eq!(world.state_hash().unwrap(), before);
    world.submit(None, &EponymIntent::Tell(telling)).unwrap();
    assert_eq!(world.sim().received_by(b).len(), 1);
    let hash = world.state_hash().unwrap();
    assert_eq!(
        submit(
            &mut world,
            b,
            Knowing::Observe {
                observer: a,
                event: proposition.event
            }
        ),
        Err(WorldError::NotPlayed)
    );
    assert_eq!(world.state_hash().unwrap(), hash);
}
