// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use isocosm_views::{ItemsRequest,ItemsOutcome};

impl NetBridge {
    pub fn native_items(&mut self,input:ItemsRequest)->Option<RequestId> {
        let request=self.request_ids.issue();
        self.actor.command(BridgeCommand::NativeItems {request,input}).then_some(request)
    }
    pub fn take_items_results(&mut self)->Vec<Correlated<Result<ItemsOutcome,String>>> {
        std::mem::take(&mut self.items_results)
    }
}

pub(super) async fn host_request(host:&HostNet,input:ItemsRequest)->Result<ItemsOutcome,String> {
    match input {
        ItemsRequest::Quote(input)=>Ok(ItemsOutcome::Quote {preview:host.quote_native_items(&input).await?,input}),
        ItemsRequest::Apply(input)=>{host.native_items(&input).await?;Ok(ItemsOutcome::Applied)},
    }
}

pub(super) async fn client_request(client:&ClientNet,input:ItemsRequest)->Result<ItemsOutcome,String> {
    match input {
        ItemsRequest::Quote(input)=>{
            let snapshot=client.state().await.ok_or("item quote needs the authenticated host snapshot")?;
            let (campaign,access)=crate::native_party::context(&snapshot,
                isonetry::native_table::TableSender::Endpoint(client.endpoint_key()))?;
            let table=snapshot.native_table.as_ref().ok_or("item quote has no actual native table")?;
            let preview=isocosm_native::native::items::prepare_item(campaign.table_session(),&table.game,access,&input)?;
            Ok(ItemsOutcome::Quote {input,preview})
        },
        ItemsRequest::Apply(input)=>{client.native_items(input).await?;Ok(ItemsOutcome::Submitted)},
    }
}
