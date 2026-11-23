use chirograph::{IntentInvocation, IntentResult};
use cosmograph::{Command, Execution, Session};
use cosmograph_host::{fixtures::fixture, world::World};
use cosmograph_overlay::ParticipantHandle;
use eponym_play::{
    contract::mode::ModeState,
    endpoint::{EponymEndpoint, MODE, REST, SESSION, STEP, TAKE},
    fixtures::{native_clock::cooperative_lives_in_rounds, native_severing::STRIKER},
};
use graphshell_endpoint::{IntentSink, ProjectionCatalog, ProjectionSource};
use std::collections::BTreeMap;

use super::harness::{ClientIntent, Harness};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn eponym_step_crosses_admission_as_a_time_only_record() {
    tokio::task::LocalSet::new()
        .run_until(async {
            let f = fixture();
            let participant = f.participants[0];
            let mut family = Harness::new(f.world, 41).await;
            let endpoint = EponymEndpoint::open(family.host.borrow().world(), participant)
                .expect("actual Eponym endpoint opens on family host world");
            let projection = endpoint.describe().projections[0].request.clone();
            assert_eq!(projection.session.0, SESSION);
            let before_tick = family.host.borrow().world().cut().unwrap().tick;

            assert!(
                !family.sync_pending().await.is_empty(),
                "the actual endpoint window is journaled"
            );
            family
                .serve(
                    endpoint,
                    vec![ClientIntent {
                        projection: projection.clone(),
                        target: "eponym".into(),
                        intent: STEP.into(),
                        payload: Vec::new(),
                    }],
                )
                .await;
            let accepted = family.sync_pending().await;
            assert_eq!(accepted.len(), 1);
            assert_eq!(accepted[0].final_tick, before_tick + 1);
            assert!(
                accepted[0].native_suffix.is_empty(),
                "the endpoint advances time without a command entry"
            );
            assert_eq!(
                accepted[0].section_updates,
                std::collections::BTreeMap::from([("eponym".into(), Some(Vec::new()))]),
                "the existing endpoint persists its absent mode as empty bytes"
            );
            family.assert_converged();
            family.shutdown().await;
        })
        .await;
}

#[test]
fn an_occupied_first_life_is_refused_before_joining() {
    use cosmograph_overlay::EntityHandle;
    use cosmograph_overlay::eponym::{
        EponymIntent, FirstLife, LifeCheckpoint, LifeChoice, StartTime,
    };
    use eponym_play::contract::Borrowed;

    let mut seed = cooperative_lives_in_rounds();
    let before = seed.native.save();
    let b = seed.binding();
    let mut endpoint = Borrowed::new(
        &mut seed.native,
        b.site,
        b.window.min,
        b.window.max,
        b.window.depth,
    )
    .unwrap();
    let first = EponymIntent::Checkpoint(LifeCheckpoint::First(FirstLife {
        start: StartTime::Society,
        begins_as: LifeChoice::Denizen(EntityHandle(seed.entity)),
    }));
    assert_eq!(
        endpoint.submit(None, &first),
        Err(eponym_play::WorldError::NotPlayed)
    );
    assert_eq!(seed.native.save(), before);
}

/// Address the body actually disclosed to this ordinary seat. A forged rest on
/// a visible neighbour must reach the ordinary intent admission check.
fn local_action(endpoint: &mut EponymEndpoint, body: u64, intent: &str) -> IntentResult {
    let req = endpoint.describe().projections.remove(0).request;
    let snap = endpoint.snapshot(req.clone()).unwrap();
    let id = format!("body:{body}");
    let target = snap
        .scene
        .active_items_in_order()
        .into_iter()
        .find_map(|(n, item)| {
            (snap.scene.tables.sources[item.source.0 as usize]
                .as_ref()?
                .id
                == id)
                .then_some(n)
        })
        .expect("body is visible to this participant");
    endpoint
        .invoke(IntentInvocation {
            session: req.session,
            target,
            intent: intent.into(),
            payload: Vec::new(),
            observed_epoch: snap.scene.epoch,
            observed_revision: snap.scene.revision,
        })
        .unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_lives_act_under_separate_seats_and_the_dead_life_takes_a_free_successor() {
    tokio::task::LocalSet::new()
        .run_until(play_through_succession())
        .await;
}

async fn play_through_succession() {
    let mut seed = cooperative_lives_in_rounds();
    let mode = seed.mode();
    let living = mode
        .bindings
        .iter()
        .find(|b| b.subject == STRIKER)
        .unwrap()
        .entity;
    let successor = mode
        .bindings
        .iter()
        .find(|b| b.entity != living && b.entity != seed.entity)
        .unwrap();
    let successor_body = successor.entity;
    let successor_subject = successor.subject;
    let keeper = seed.entity;
    let guest = ParticipantHandle(seed.participant);
    let joined = seed.native.command(Command::Join).unwrap();
    let owner = ParticipantHandle(
        joined
            .strip_prefix("participant:")
            .unwrap()
            .parse()
            .unwrap(),
    );
    assert_eq!(
        seed.native
            .command(Command::Take {
                participant: owner.0,
                critter: living
            })
            .unwrap(),
        "taken"
    );
    let world = World::with_sections(seed.native, BTreeMap::from([(MODE.into(), seed.game)]));
    let mut family = Harness::new(world, 85).await;
    const OWNER_IDENTITY: u8 = 89;
    const GUEST_IDENTITY: u8 = 90;
    family.use_ordinary_player(OWNER_IDENTITY).await;
    let authority = family.host.borrow().world().share();
    let mut local = EponymEndpoint::open(&authority, owner).unwrap();
    let remote = EponymEndpoint::open(&authority, guest).unwrap();
    let request = remote.describe().projections.remove(0).request;
    let act = |target: String, intent: &str| ClientIntent {
        projection: request.clone(),
        target,
        intent: intent.into(),
        payload: Vec::new(),
    };
    family.sync_pending().await;
    let before = family.host.borrow().sequence();
    assert!(matches!(
        local_action(&mut local, keeper, REST),
        IntentResult::Rejected { .. }
    ));
    assert_eq!(family.host.borrow().sequence(), before);
    family
        .serve(local, vec![act(format!("body:{living}"), REST)])
        .await;
    family.use_ordinary_player(GUEST_IDENTITY).await;
    family
        .serve(remote, vec![act(format!("body:{keeper}"), REST)])
        .await;
    let records = family.sync_pending().await;
    assert_eq!(records.len(), 2, "one accepted action per player");
    authority
        .read(|s| {
            let resting: Vec<_> = s
                .entries
                .iter()
                .filter_map(|entry| match &entry.command {
                    Command::ActOwned {
                        participant,
                        actor,
                        process,
                        ..
                    } if process == eponym_play::founding::RESTS => Some((*participant, *actor)),
                    _ => None,
                })
                .collect();
            assert!(
                resting.ends_with(&[(owner.0, living), (guest.0, keeper)]),
                "{resting:?}"
            );
        })
        .unwrap();

    // An authored scenario transition supplies a native death, separately
    // from the two players' ordinary actions.
    let death = authority
        .command(Command::Act {
            actor: keeper,
            target: None,
            process: eponym_play::founding::SPENT.into(),
            cause: None,
        })
        .unwrap();
    let death: cosmograph::simulation::Receipt = serde_json::from_str(&death).unwrap();
    assert!(death.accepted());
    let deceased = EponymEndpoint::open(&authority, guest).unwrap();
    let offers = family
        .serve_and_read(deceased, None, Vec::new(), vec![request.clone()])
        .await;
    let offered: Vec<_> = offers[0]
        .iter()
        .filter(|id| id.starts_with("life:"))
        .cloned()
        .collect();
    let next = format!("life:{}", successor_subject.0);
    assert_eq!(
        offered,
        [next.clone()],
        "the surviving player's life cannot be taken"
    );

    let resumed = EponymEndpoint::open(&authority, guest).unwrap();
    family
        .serve(
            resumed,
            vec![act(next, TAKE), act(format!("body:{successor_body}"), REST)],
        )
        .await;
    family.use_ordinary_player(OWNER_IDENTITY).await;
    let survivor = EponymEndpoint::open(&authority, owner).unwrap();
    family
        .serve(survivor, vec![act(format!("body:{living}"), REST)])
        .await;
    family.sync_pending().await;
    family.assert_converged();
    assert!(!family.has_bootstrapped_peer());
    let save = family.replica.snapshot().unwrap().save;
    let replay = Session::load(save.native.clone(), Execution::Individuals).unwrap();
    assert_eq!(replay.save(), save.native);
    assert!(!replay.sim.state().population.get(keeper).unwrap().alive);
    assert_eq!(replay.sim.plays(guest.0), Some(successor_body));
    assert_eq!(replay.sim.plays(owner.0), Some(living));
    let mode = ModeState::decode(&save.sections[MODE]).unwrap().unwrap();
    mode.validate_native(&replay).unwrap();
    let life = mode.life.last().unwrap();
    assert_eq!((life.participant, life.target), (guest.0, successor_body));
    eprintln!(
        "Eponym owner {} played {}, player {} played {} then took {}; native and signed family replay agree at sequence {}",
        owner.0,
        living,
        guest.0,
        keeper,
        successor_body,
        family.replica.sequence()
    );
    family.shutdown().await;
}
