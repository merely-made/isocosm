// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Actual endpoint identity stamps the party panel's journey request.

use super::*;
use isometry_views::{JourneyRequest,JourneyQuote,PartyView};
use isonetry::native_table::TableSender;
use isometry_campaign::native::joint_journey::{JourneyEnvelope,JourneyInput,preview_departure};

pub(crate) fn envelope(snapshot: &GameSnapshot, sender: TableSender, request: JourneyRequest)
    -> Result<JourneyEnvelope,String> {
    let (campaign,access) = crate::native_party::context(snapshot,sender)?;
    let intent = match request {
        JourneyRequest::Propose { party,departures } => JourneyInput::Propose { party,departures },
        JourneyRequest::Approve { proposal } => JourneyInput::Approve { proposal },
        JourneyRequest::Apply { proposal } => JourneyInput::Apply { proposal },
        JourneyRequest::Quote { .. } => return Err("native stock quote is a read, not a journey command".into()),
    };
    Ok(JourneyEnvelope { participant:access.participant.0,tick:campaign.table_session().sim.state().tick,intent })
}

pub(crate) fn quote(snapshot: &GameSnapshot, sender: TableSender, party: &str,
    input: isometry_campaign::native::joint_journey::DepartureInput) -> Result<PartyView,String> {
    let (campaign,access) = crate::native_party::context(snapshot,sender)?;
    let table = snapshot.native_table.as_ref().ok_or("native departure quote needs its admitted table")?;
    let native = preview_departure(campaign.table_session(),access,&table.time_roster()?,party,&input)?;
    let mut view = crate::native_party::read(snapshot,sender,party)?;
    view.journey_quote = Some(JourneyQuote { input,native });
    Ok(view)
}
