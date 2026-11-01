// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Seats as grants (case 1059). Each participant may hold the GM grant, any
//! number or none; the player grant is the saved roster. A window's seat
//! chooses only its knowledge view and must be held: a GM window needs the
//! grant. Taking the GM grant needs the yes of every GM seated, or of every
//! bound player when none is. Absent seats are a table from before grants,
//! where the trusted host's DM window stands as before.

use super::GameBindings;
use cosmograph::{Session, schema::Id};
use cosmograph_overlay::{OrdinaryAccess, Seat};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const KIND: &str = "vtt:seat";

/// The saved grants. `revision` moves with every change of `gms`; a claim
/// binds the revision and roster it was made against.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seats {
    pub gms: BTreeSet<Id>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub claims: BTreeMap<Id, Claim>,
    pub revision: u64,
}

/// An open claim on the GM grant and the yeses it needs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim {
    /// Whose yes it needs: the GMs seated, or the bound players when none is.
    pub consenters: BTreeSet<Id>,
    pub yeses: BTreeSet<Id>,
    pub seats_revision: u64,
    pub roster_revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeatInput {
    pub kind: String,
    pub version: u32,
    pub expected_revision: u64,
    pub action: SeatAction,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeatAction {
    /// The trusted host's DM window records itself as the first GM.
    Configure,
    Claim,
    Withdraw,
    Consent { claimant: Id },
    Release,
}

impl SeatInput {
    pub fn new(expected_revision: u64, action: SeatAction) -> Self {
        Self { kind: KIND.into(), version: 1, expected_revision, action }
    }
}

impl Seats {
    pub fn holds_gm(&self, participant: Id) -> bool { self.gms.contains(&participant) }

    /// A claim is current while the grants and roster it was made against stand.
    pub fn current(&self, claim: &Claim, roster_revision: u64) -> bool {
        claim.seats_revision == self.revision && claim.roster_revision == roster_revision
    }

    pub fn validate(&self, session: &Session) -> Result<(), String> {
        let native = |id: &Id| session.sim.state().population.get(*id).is_some_and(cosmograph::directing::is_participant);
        if !self.gms.iter().all(native) || !self.claims.keys().all(native) {
            return Err("saved seats name no actual native participant".into());
        }
        if self.claims.iter().any(|(claimant, claim)| self.gms.contains(claimant)
            || !claim.yeses.is_subset(&claim.consenters) || claim.consenters.contains(claimant)) {
            return Err("saved GM claim is held, self-consented or has a stray yes".into());
        }
        Ok(())
    }
}

/// A window's seat must be held: a GM window needs the GM grant once the
/// table keeps grants. Player and observer windows need none to read.
pub fn require_held(game: &[u8], access: OrdinaryAccess) -> Result<(), String> {
    if access.seat != Seat::Dm { return Ok(()); }
    match GameBindings::decode(game)?.seats() {
        Some(seats) if !seats.holds_gm(access.participant.0) => Err("this participant holds no GM grant".into()),
        _ => Ok(()),
    }
}

pub(super) fn submit(session: &Session, game: Vec<u8>, access: OrdinaryAccess, input: &SeatInput)
    -> Result<(String, Vec<u8>), String> {
    super::movement::current_access(session, access)?;
    if input.kind != KIND || input.version != 1 { return Err("seat input has another kind or version".into()); }
    let mut bindings = GameBindings::decode(&game)?;
    bindings.validate(session)?;
    let me = access.participant.0;
    let roster_revision = bindings.roster_revision();
    let players = bindings.shared_players().cloned().unwrap_or_default();
    let Some(mut seats) = bindings.seats().cloned() else {
        if input.action != SeatAction::Configure || access.seat != Seat::Dm || input.expected_revision != 0 {
            return Err("this table keeps no seats yet; its DM window configures them first".into());
        }
        bindings.record_seats(Seats { gms: BTreeSet::from([me]), claims: BTreeMap::new(), revision: 1 });
        bindings.validate(session)?;
        return Ok(("seats configured; this participant holds the GM grant".into(), bindings.encode()?));
    };
    if input.expected_revision != seats.revision { return Err("seat input names a stale grant revision".into()); }
    let note = match &input.action {
        SeatAction::Configure => return Err("this table already keeps seats".into()),
        SeatAction::Claim => {
            if seats.holds_gm(me) { return Err("this participant already holds the GM grant".into()); }
            let consenters: BTreeSet<Id> = if seats.gms.is_empty() { &players - &BTreeSet::from([me]) }
                else { seats.gms.clone() };
            if consenters.is_empty() {
                grant(&mut seats, me);
                "GM grant taken: no one seated or bound to ask".to_string()
            } else {
                seats.claims.insert(me, Claim { consenters, yeses: BTreeSet::new(),
                    seats_revision: seats.revision, roster_revision });
                "GM claim opened; it waits for each yes".to_string()
            }
        },
        SeatAction::Withdraw => {
            seats.claims.remove(&me).ok_or("this participant has no open GM claim")?;
            "GM claim withdrawn".to_string()
        },
        SeatAction::Consent { claimant } => {
            let current = seats.claims.get(claimant).is_some_and(|claim| seats.current(claim, roster_revision));
            let claim = seats.claims.get_mut(claimant).ok_or("no open GM claim by that participant")?;
            if !current { return Err("that GM claim is stale: the grants or roster changed since it was made".into()); }
            if !claim.consenters.contains(&me) { return Err("this participant's yes is not one this claim needs".into()); }
            // A GM's yes is given from a GM window; a bound player's from a player window.
            let wanted = if claim.consenters == seats.gms { Seat::Dm } else { Seat::Player };
            if access.seat != wanted { return Err("give this yes from the seat the claim asks".into()); }
            claim.yeses.insert(me);
            if claim.yeses == claim.consenters {
                grant(&mut seats, *claimant);
                "GM claim granted".to_string()
            } else { "yes recorded on the GM claim".to_string() }
        },
        SeatAction::Release => {
            if access.seat != Seat::Dm || !seats.gms.remove(&me) {
                return Err("only a GM window releases its own GM grant".into());
            }
            seats.revision = seats.revision.checked_add(1).ok_or("seat revision overflow")?;
            "GM grant released".to_string()
        },
    };
    bindings.record_seats(seats);
    bindings.validate(session)?;
    Ok((note, bindings.encode()?))
}

fn grant(seats: &mut Seats, claimant: Id) {
    seats.claims.remove(&claimant);
    seats.gms.insert(claimant);
    seats.revision = seats.revision.saturating_add(1);
}
