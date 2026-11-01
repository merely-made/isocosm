//! The invite and join flow over two real transports in one process, with
//! in-memory dev identities (stack-lift D4): the owner hosts, a guest's
//! contact file is invited, the guest bootstraps, sees its seat's table,
//! acts, and its replica converges with the owner's world.

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
        for pump in pumps.iter() { table.pump_admitted_requests(pump, 8).unwrap(); }
        let turn = guest.tick();
        if let Some(status) = &turn.status { assert!(!status.contains("failed") && !status.contains("refused"), "{status}"); }
        if let Some(views) = turn.projections { *seen = views; }
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn an_invited_guest_joins_over_the_transport_acts_at_its_seat_and_converges() {
    let mut table = crate::campaign_store::bench_family_table();
    let handoff = std::env::temp_dir().join(format!("isocosm-handoff-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    // The owner keeps seats as grants first, so a guest has a seat action.
    let views = table.projections().unwrap();
    table.invoke(request(&views, "vtt.seats-configure")).unwrap();
    let owner: Arc<dyn graphshell_host::invitation::Signer + Send + Sync> = Arc::new(InMemoryProvider::from_seed([41; 32]));
    let mut host = table.host_peers(owner, handoff.clone(), Arc::new(|| {})).unwrap();
    let (mut guest, contact) = FamilyGuest::start(Arc::new(InMemoryProvider::from_seed([42; 32])),
        handoff.clone(), Arc::new(|| {})).unwrap();
    assert!(contact.exists() && handoff.join(CARD_FILE).exists());
    let (mut pumps, mut seen) = (Vec::new(), Vec::new());
    turns(&mut table, &mut host, &mut guest, &mut pumps, &mut seen, |table, guest, seen|
        guest.sequence() == Some(table.sequence()) && !seen.is_empty());
    assert_eq!(table.invitation_count(), 1, "the contact file was invited once");
    assert!(seen.iter().flat_map(|v| &v.cards).all(|c| !c.actions.iter().any(|a| a.id == "vtt.roster-toggle")),
        "the guest's page shows its player seat, not the owner's roster controls");

    // The guest claims the GM seat; the accepted claim reaches its replica.
    let before = table.sequence();
    guest.invoke(request(&seen, "vtt.seat-claim"));
    turns(&mut table, &mut host, &mut guest, &mut pumps, &mut seen, |table, guest, _|
        table.sequence() > before && guest.sequence() == Some(table.sequence()));
    let hosted = table.world_save().unwrap();
    assert_eq!(guest.replica().unwrap().snapshot().unwrap().save, hosted, "the guest's replica converges");
    guest.close();
    host.close();
    let _ = std::fs::remove_dir_all(handoff);
}
