// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use isometry_views::{PartyRequest, PartyView};
use isonetry::native_table::TableSender;

impl NetBridge {
    pub fn native_party(&mut self, intent: PartyRequest) -> Option<RequestId> {
        let request = self.request_ids.issue();
        self.actor.command(BridgeCommand::NativeParty { request, intent }).then_some(request)
    }
    pub fn take_party_readings(&mut self) -> Vec<Correlated<Result<PartyView, String>>> {
        std::mem::take(&mut self.party_readings)
    }
}

pub(super) async fn host_request(host: &HostNet, request: PartyRequest) -> Result<Option<PartyView>, String> {
    let snapshot = host.snapshot().await;
    if let PartyRequest::Read { party } = request {
        return crate::native_party::read(&snapshot, TableSender::Host, &party).map(Some);
    }
    if let PartyRequest::Journey(request) = request {
        if let isometry_views::JourneyRequest::Quote { party,input } = request {
            return crate::native_journey::quote(&snapshot,TableSender::Host,&party,input).map(Some);
        }
        let input = crate::native_journey::envelope(&snapshot,TableSender::Host,request)?;
        host.native_journey(&input).await?;
        return Ok(None);
    }
    let envelope = crate::native_party::envelope(&snapshot, TableSender::Host, request)?;
    host.native_party(&envelope).await?;
    Ok(None)
}

pub(super) async fn client_request(client: &ClientNet, request: PartyRequest) -> Result<Option<PartyView>, String> {
    let snapshot = client.state().await.ok_or("party request needs the authenticated host snapshot")?;
    let sender = TableSender::Endpoint(client.endpoint_key());
    if let PartyRequest::Read { party } = request {
        return crate::native_party::read(&snapshot, sender, &party).map(Some);
    }
    if let PartyRequest::Journey(request) = request {
        if let isometry_views::JourneyRequest::Quote { party,input } = request {
            return crate::native_journey::quote(&snapshot,sender,&party,input).map(Some);
        }
        let input = crate::native_journey::envelope(&snapshot,sender,request)?;
        client.native_journey(input).await?;
        return Ok(None);
    }
    let envelope = crate::native_party::envelope(&snapshot, sender, request)?;
    client.native_party(envelope).await?;
    Ok(None)
}
