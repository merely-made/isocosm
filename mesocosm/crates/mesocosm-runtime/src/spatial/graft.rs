// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Read the actual saved poses beside an exact native carry. A missing or
//! stale binding remains unavailable; no attachment or root pose is chosen.

use super::{Game, Placement};
use isocosm::{Session, schema::Id};
use isocosm_overlay::EntityHandle;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Binding {
    Current(Placement),
    Unbound,
    Unavailable(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BodyReading { pub entity: EntityHandle, pub binding: Binding }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reading { pub actor: BodyReading, pub donor: BodyReading }

/// The same complete native source used by authored spatial admission.
pub fn source(session: &Session) -> String { super::profile::native_source(session) }

impl Reading {
    pub fn of(session: &Session, game: &Game, actor: Id, donor: Id) -> Self {
        let body = |id| {
            let binding = match game.members.get(&id) {
                None => Binding::Unbound,
                Some(at) => match at.realized(&session.sim) {
                    Ok(()) => Binding::Current(at.clone()),
                    Err(why) => Binding::Unavailable(why),
                },
            };
            BodyReading { entity: EntityHandle(id), binding }
        };
        Self { actor: body(actor), donor: body(donor) }
    }
}
