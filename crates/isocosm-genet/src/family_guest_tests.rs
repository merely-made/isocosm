//! Real loopback sessions exercise seat projections and separately authorized full replicas.

use super::*;
use crate::family_table::FamilyTable;
use personae::InMemoryProvider;
use std::time::Instant;

fn request(views: &[VttProjectionView], action: &str) -> VttActionRequest {
    let (view, card) = views.iter().flat_map(|view| view.cards.iter().map(move |card| (view, card)))
        .find(|(_, card)| card.actions.iter().any(|a| a.id == action))
        .unwrap_or_else(|| panic!("no card offers {action}"));
    VttActionRequest { projection: view.id.clone(), card: card.id.clone(), action: action.into(),
        payload: Vec::new(), epoch: view.epoch, revision: view.revision }
}

/// Run the owner's and the guest's turns until `done` or a deadline.
fn turns(table: &mut FamilyTable, host: &mut crate::family_table::host_net::HostNet, guest: &mut FamilyGuest,
    pumps: &mut Vec<isonetry::family_network::AdmittedRequestPump>, seen: &mut Vec<VttProjectionView>,
    done: impl Fn(&FamilyTable, &FamilyGuest, &[VttProjectionView]) -> bool) {
    let wake: Arc<dyn Fn() + Send + Sync> = Arc::new(|| {});
    let deadline = Instant::now() + Duration::from_secs(60);
    while !done(table, guest, seen) {
        assert!(Instant::now() < deadline, "flow stalled: guest at {:?}, host at {}", guest.sequence(), table.sequence());
        let (new, notes) = host.tick(table, Arc::clone(&wake));
        pumps.extend(new);
        assert!(notes.iter().all(|n| !n.contains("refused") && !n.contains("failed")), "{notes:?}");
        pumps.retain(|pump| match table.pump_admitted_requests(pump, 8) {
            Ok(_) => true,
            Err(error) if error.contains("bridge is closed") => false,
            Err(error) => panic!("serving guest: {error}"),
        });
        let turn = guest.tick();
        if let Some(status) = &turn.status { assert!(!status.contains("failed") && !status.contains("refused"), "{status}"); }
        if let Some(views) = turn.projections { *seen = views; }
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn handoff() -> PathBuf {
    std::env::temp_dir().join(format!("isocosm-handoff-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()))
}

fn has_action(views: &[VttProjectionView], id: &str) -> bool {
    views.iter().flat_map(|v| &v.cards).flat_map(|c| &c.actions).any(|a| a.id == id)
}

#[test]
fn ordinary_guest_follows_seat_changes_and_acts_without_receiving_a_replica() {
    let mut table = crate::campaign_store::bench_family_table();
    let handoff = handoff();
    let owner = Arc::new(InMemoryProvider::from_seed([41; 32]));
    let player = InMemoryProvider::from_seed([42; 32]);
    let mut host = table.host_peers(owner.clone(), handoff.clone(), None, Arc::new(|| {})).unwrap();
    let (mut guest, contact) = FamilyGuest::start(Arc::new(player), handoff.clone(), false, Arc::new(|| {})).unwrap();
    assert!(contact.exists());
    let card: serde_json::Value = read_json(&handoff.join(CARD_FILE)).unwrap();
    assert!(card.get("checkpoint").is_none() && card.get("records").is_none());
    let (mut pumps, mut seen) = (Vec::new(), Vec::new());
    turns(&mut table, &mut host, &mut guest, &mut pumps, &mut seen, |_, _, views| !views.is_empty());
    assert_eq!(table.invitation_count(), 1);
    assert!(!has_action(&seen, "vtt.roster-toggle"));
    assert!(!has_action(&seen, "vtt.seat-claim"));
    // The owner changes the seat configuration while the guest is idle.
    let views = table.projections().unwrap();
    table.invoke(request(&views, "vtt.seats-configure")).unwrap();
    turns(&mut table, &mut host, &mut guest, &mut pumps, &mut seen, |_, _, views| has_action(views, "vtt.seat-claim"));
    let before = table.sequence();
    guest.invoke(request(&seen, "vtt.seat-claim"));
    turns(&mut table, &mut host, &mut guest, &mut pumps, &mut seen, |table, _, views|
        table.sequence() == before + 1 && has_action(views, "vtt.seat-withdraw"));
    assert!(guest.replica().is_none());
    // The guest worker remains alive while its owner rebinds the table to a new ticket.
    host.close();
    pumps.clear();
    seen.clear();
    let deadline = Instant::now() + Duration::from_secs(15);
    while !guest.tick().projections.is_some_and(|views| views.is_empty()) {
        assert!(Instant::now() < deadline, "the former seat view did not close");
        std::thread::sleep(Duration::from_millis(20));
    }
    let mut host = table.host_peers(owner, handoff.clone(), None, Arc::new(|| {})).unwrap();
    turns(&mut table, &mut host, &mut guest, &mut pumps, &mut seen, |_, _, views| has_action(views, "vtt.seat-withdraw"));
    assert_eq!(pumps.len(), 1, "the replacement host admitted a fresh connection");
    assert_eq!(table.sequence(), before + 1);
    assert_eq!(table.invitation_count(), 1);
    assert!(guest.replica().is_none());
    guest.close();
    host.close();
    std::fs::remove_dir_all(handoff).unwrap();
}

#[test]
fn explicitly_granted_replica_follows_the_signed_world_only_when_requested() {
    let mut table = crate::campaign_store::bench_family_table();
    let handoff = handoff();
    let owner = InMemoryProvider::from_seed([51; 32]);
    let player = Arc::new(InMemoryProvider::from_seed([52; 32]));
    let contact = Contact::create(&*player, &TERMS).unwrap();
    let mut host = table.host_peers(Arc::new(owner), handoff.clone(), Some(contact.clone()), Arc::new(|| {})).unwrap();
    let seat = table.invitation_count();
    let (mut guest, _) = FamilyGuest::start(player.clone(), handoff.clone(), false, Arc::new(|| {})).unwrap();
    let (mut pumps, mut seen) = (Vec::new(), Vec::new());
    turns(&mut table, &mut host, &mut guest, &mut pumps, &mut seen, |_, _, views| !views.is_empty());
    assert!(guest.replica().is_none(), "a local replica also requires the player's opt-in");
    guest.close();
    // An unselected contact can read its own projection, including on a replication-enabled host.
    let ordinary = InMemoryProvider::from_seed([53; 32]);
    let ordinary_contact = Contact::create(&ordinary, &TERMS).unwrap();
    let (mut guest, _) = FamilyGuest::start(Arc::new(ordinary), handoff.clone(), true, Arc::new(|| {})).unwrap();
    seen.clear();
    turns(&mut table, &mut host, &mut guest, &mut pumps, &mut seen, |_, _, views| !views.is_empty());
    assert!(guest.replica().is_none());
    let offer: ReplicaOffer = read_json(&handoff.join(replica_file(&contact))).unwrap();
    let call = isonetry::family_network::EndpointCall::Resource(offer.card.checkpoint.request());
    assert!(table.checkpoint_answer(ordinary_contact.peer(), &call).is_none());
    assert!(table.checkpoint_answer(contact.peer(), &call).is_some());
    guest.close();
    let (mut guest, _) = FamilyGuest::start(player, handoff.clone(), true, Arc::new(|| {})).unwrap();
    seen.clear();
    turns(&mut table, &mut host, &mut guest, &mut pumps, &mut seen, |table, guest, views|
        guest.sequence() == Some(table.sequence()) && !views.is_empty());
    let views = table.projections().unwrap();
    table.invoke(request(&views, "vtt.seats-configure")).unwrap();
    turns(&mut table, &mut host, &mut guest, &mut pumps, &mut seen, |table, guest, views|
        guest.sequence() == Some(table.sequence()) && has_action(views, "vtt.seat-claim"));
    assert_eq!(table.invitation_count(), seat + 1);
    assert_eq!(guest.replica().unwrap().snapshot().unwrap().save, table.world_save().unwrap());
    guest.close();
    host.close();
    std::fs::remove_dir_all(handoff).unwrap();
}

#[path = "family_guest_tests/encounter.rs"]
mod encounter;
