#![cfg(feature = "endpoint")]
//! A timeskip is a span of the wait ladder proposed as consented downtime and
//! stepped on the one clock (case 1075).
use chirograph::{IntentInvocation, IntentResult};
use cosmograph_overlay::Seat;
use graphshell_endpoint::{IntentSink, ProjectionCatalog, ProjectionSource};
use isocosm_native::{TABLE, VttEndpoint};
#[path = "support/table.rs"]
mod table;

fn click(endpoint: &mut VttEndpoint, card: impl Fn(&str) -> bool, intent: &str) -> IntentResult {
    let offer = endpoint.describe().projections.into_iter().find(|o| o.request.session.0 == TABLE).unwrap();
    let snap = endpoint.snapshot(offer.request).unwrap();
    let target = snap.scene.active_items_in_order().into_iter()
        .find(|(_, item)| card(&snap.scene.tables.sources[item.source.0 as usize].as_ref().unwrap().id))
        .unwrap_or_else(|| panic!("no card for {intent}")).0;
    endpoint.invoke(IntentInvocation { session: snap.session, target, observed_epoch: snap.scene.epoch,
        observed_revision: snap.scene.revision, intent: intent.into(), payload: vec![] }).unwrap()
}

#[test]
fn a_gm_skips_a_day_as_consented_downtime() {
    let t = table::table_with_sites(1);
    let mut gm = VttEndpoint::open(&t.world, t.dm, Seat::Dm).unwrap();
    let ok = IntentResult::Accepted;
    assert_eq!(click(&mut gm, |c| c == "roster", "vtt.roster-empty"), ok);
    let before = t.world.read(|session| session.sim.state().tick).unwrap();
    assert_eq!(click(&mut gm, |c| c == "time", "vtt.propose-day"), ok);
    assert_eq!(click(&mut gm, |c| c.starts_with("proposal:"), "vtt.advance"), ok);
    let after = t.world.read(|session| session.sim.state().tick).unwrap();
    assert_eq!(after - before, 1_440, "a day at the minute every game keeps");
}
