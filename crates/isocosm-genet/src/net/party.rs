// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use isocosm_views::{PartyRequest, PartyView};
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
    if let PartyRequest::AuthorLocation(input) = request {
        host.native_location(&input).await?;
        return Ok(None);
    }
    if let PartyRequest::AuthorStatement(input) = request {
        let input = crate::native_party::statement_admission(&snapshot, &input)?;
        host.commit_native_table_admission(&input).await?;
        return Ok(None);
    }
    if let PartyRequest::Hook(intent)=request {
        let input=crate::native_party::hook_envelope(&snapshot,intent)?;
        host.native_hook(&input).await?;
        return Ok(None);
    }
    if let PartyRequest::ReadAt { party,settings } = request {
        return crate::native_party::read_with_settings(&snapshot,TableSender::Host,&party,Some(settings)).map(Some);
    }
    if let PartyRequest::Presentation(input) = &request {
        let input = crate::native_party::presentation_input(&snapshot,input)?;
        host.commit_native_table_binding(&input).await?;
        return Ok(None);
    }
    if let PartyRequest::Read { party } = request {
        return crate::native_party::read(&snapshot, TableSender::Host, &party).map(Some);
    }
    if let PartyRequest::Journey(request) = request {
        if let isocosm_views::JourneyRequest::Quote { party,input } = request {
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
    if matches!(&request, PartyRequest::AuthorStatement(_) | PartyRequest::AuthorLocation(_)) { return Err("sources require the actual host author door".into()); }
    if matches!(&request,PartyRequest::Hook(_)) {return Err("hook decisions require the actual host".into());}
    if matches!(&request,PartyRequest::Presentation(_)) {
        return Err("map presentation authoring requires the actual host".into());
    }
    let snapshot = client.state().await.ok_or("party request needs the authenticated host snapshot")?;
    let sender = TableSender::Endpoint(client.endpoint_key());
    if let PartyRequest::ReadAt { party,settings } = request {
        return crate::native_party::read_with_settings(&snapshot,sender,&party,Some(settings)).map(Some);
    }
    if let PartyRequest::Read { party } = request {
        return crate::native_party::read(&snapshot, sender, &party).map(Some);
    }
    if let PartyRequest::Journey(request) = request {
        if let isocosm_views::JourneyRequest::Quote { party,input } = request {
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
