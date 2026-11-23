//! Two independent transports play a prepared Pathfinder encounter and reopen its ledger.

use super::*;
use crate::family_table::invite::now_ms;
use isocosm_core::encounter_board::{BADGE, EncounterBoard, VALUE};
use isocosm_native::{TABLE, native::GameBindings};

fn board(
    views: &[VttProjectionView],
) -> Option<(
    &VttProjectionView,
    &isocosm_views::VttCardView,
    EncounterBoard,
)> {
    let view = views.iter().find(|v| v.id == TABLE)?;
    let card = view
        .cards
        .iter()
        .find(|c| c.badges.iter().any(|b| b == BADGE))?;
    let encoded = card.values.iter().find(|value| value.label == VALUE)?;
    Some((view, card, serde_json::from_str(&encoded.value).unwrap()))
}

fn feedback(views: &[VttProjectionView]) -> (String, Option<String>) {
    let (_, card, _) = board(views).expect("the player's encounter board");
    let field = |label: &str| card.values.iter().find(|v| v.label == label).map(|v| v.value.clone());
    (field("Actions remaining").expect("visible turn budget"), field("Last Strike"))
}

fn gm_click(table: &mut FamilyTable, intent: &str) {
    let views = table.projections().unwrap();
    assert_eq!(
        table.invoke(request(&views, intent)).unwrap(),
        IntentResult::Accepted
    );
}

#[test]
#[ignore = "requires a fresh pf2e_table --combat ledger at ISOCOSM_PF2E_TRIAL_DIR/seed.family"]
fn invited_pathfinder_player_strides_strikes_and_rejoins_the_reopened_table() {
    let dir = PathBuf::from(
        std::env::var_os("ISOCOSM_PF2E_TRIAL_DIR").expect("supply the trial directory"),
    );
    let ledger = dir.join("played.family");
    let mut input = std::fs::File::open(dir.join("seed.family")).unwrap();
    let mut output = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&ledger)
        .unwrap();
    std::io::copy(&mut input, &mut output).unwrap();
    drop(output);
    let handoff = dir.join("handoff");
    let owner = Arc::new(InMemoryProvider::from_seed([21; 32]));
    let player = Arc::new(InMemoryProvider::from_seed([22; 32]));
    let wake: Arc<dyn Fn() + Send + Sync> = Arc::new(|| {});
    let mut table = FamilyTable::open(None, None, Some(&ledger)).unwrap();
    let game = GameBindings::decode(&table.vtt_section().unwrap()).unwrap();
    let actor = game.characters()["arden"].entity;
    let target = game.characters()["brisa"].entity;
    let seat = game.controls()[&actor].participant;
    let mut host = table
        .host_peers(owner.clone(), handoff.clone(), None, wake.clone())
        .unwrap();
    let (mut guest, contact) =
        FamilyGuest::start(player.clone(), handoff.clone(), false, wake.clone()).unwrap();
    let contact: Contact = read_json(&contact).unwrap();
    let now = now_ms();
    let (_, bound) = table
        .invite_player(&*owner, contact.clone(), Some(seat), now, now + 3_600_000)
        .unwrap();
    assert_eq!(bound, seat);
    gm_click(&mut table, "vtt.pf2e-start-encounter");
    let turn = table.edition_state().unwrap().unwrap().encounter.unwrap();
    if u64::from(turn.current().unwrap().token) != actor {
        gm_click(&mut table, "vtt.pf2e-end-turn");
    }
    let (mut pumps, mut seen) = (Vec::new(), Vec::new());
    turns(
        &mut table,
        &mut host,
        &mut guest,
        &mut pumps,
        &mut seen,
        |_, _, views| {
            board(views).is_some_and(|(_, _, b)| b.cells.iter().any(|c| c.stride.is_some()))
        },
    );
    assert!(guest.replica().is_none());
    let (view, card, offered) = board(&seen).unwrap();
    assert_eq!(feedback(&seen), ("3".to_owned(), None));
    let origin = offered
        .tokens
        .iter()
        .find(|token| token.body == actor)
        .unwrap()
        .at;
    let cell = offered
        .cells
        .iter()
        .filter(|c| c.stride.is_some())
        .min_by_key(|c| {
            (0..3)
                .map(|axis| origin[axis].abs_diff(c.at[axis]))
                .max()
                .unwrap()
        })
        .unwrap();
    let destination = cell.at;
    let stride = cell.stride.clone().unwrap();
    let before = table.sequence();
    guest.invoke(VttActionRequest {
        projection: view.id.clone(),
        card: card.id.clone(),
        action: stride,
        payload: vec![],
        epoch: view.epoch,
        revision: view.revision,
    });
    turns(
        &mut table,
        &mut host,
        &mut guest,
        &mut pumps,
        &mut seen,
        |table, _, views| {
            table.sequence() == before + 1
                && board(views).is_some_and(|(_, _, b)| {
                    b.tokens
                        .iter()
                        .any(|t| t.body == actor && t.at == destination)
                })
        },
    );
    assert_eq!(feedback(&seen), ("2".to_owned(), None));
    let strike = format!("vtt.pf2e-strike-{target}");
    let (view, card, _) = board(&seen).unwrap();
    assert!(
        card.actions.iter().any(|a| a.id == strike),
        "the rogue can see and reach the fighter"
    );
    let stride_revision = view.revision;
    guest.invoke(VttActionRequest {
        projection: view.id.clone(),
        card: card.id.clone(),
        action: strike,
        payload: vec![],
        epoch: view.epoch,
        revision: view.revision,
    });
    turns(
        &mut table,
        &mut host,
        &mut guest,
        &mut pumps,
        &mut seen,
        |table, _, views| {
            table.sequence() == before + 2
                && views
                    .iter()
                    .find(|v| v.id == TABLE)
                    .is_some_and(|v| v.revision > stride_revision)
        },
    );
    let state = table.edition_state().unwrap().unwrap();
    let turn = state.encounter.as_ref().unwrap().current().unwrap();
    assert_eq!(
        (u64::from(turn.token), turn.actions, turn.attacks),
        (actor, 1, 1)
    );
    let logged = state.log.last().unwrap();
    let (actions, result) = feedback(&seen);
    assert_eq!(actions, "1");
    let result = result.expect("the player sees the accepted Strike on the board");
    assert!(result.contains("Arden") && result.contains("Brisa"), "{result}");
    assert!(result.contains(&format!("roll {}", logged.rolls[0].total)), "{result}");
    assert!(result.contains("failure") || result.contains("success"), "{result}");
    guest.close();
    drop(pumps);
    host.close();
    drop(table);

    let mut table = FamilyTable::open(None, None, Some(&ledger)).unwrap();
    assert_eq!(table.sequence(), before + 2);
    assert_eq!(table.edition_state().unwrap().unwrap(), state);
    assert_eq!(table.participant_for_peer(contact.peer()), Some(seat));
    let mut host = table
        .host_peers(owner, handoff.clone(), None, wake.clone())
        .unwrap();
    let (mut guest, _) = FamilyGuest::start(player, handoff, false, wake).unwrap();
    let (mut pumps, mut seen) = (Vec::new(), Vec::new());
    turns(
        &mut table,
        &mut host,
        &mut guest,
        &mut pumps,
        &mut seen,
        |_, _, views| {
            board(views).is_some_and(|(_, _, b)| {
                b.tokens
                    .iter()
                    .any(|t| t.body == actor && t.at == destination)
            })
        },
    );
    assert!(guest.replica().is_none());
    assert_eq!(feedback(&seen), ("1".to_owned(), Some(result)));
    assert_eq!(
        table.sequence(),
        before + 2,
        "rejoin must not replay either action"
    );
    println!(
        "seat={seat} actor={actor} stride={destination:?} strike_target={target} actions={} attacks={} reopened_sequence={}",
        turn.actions,
        turn.attacks,
        table.sequence()
    );
    guest.close();
    drop(pumps);
    host.close();
}
