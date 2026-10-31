// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use isocosm_views::{PackReading, PackRequest};
use isocosm_native::native::pack::Choreography;
use isocosm_system::GeneratorLimits;

impl NetBridge {
    pub fn native_pack(&mut self, intent: PackRequest, choreography: Vec<Choreography>) -> Option<RequestId> {
        let request = self.request_ids.issue();
        self.actor.command(BridgeCommand::NativePack { request,intent,choreography }).then_some(request)
    }
    pub fn take_pack_readings(&mut self) -> Vec<Correlated<Result<PackReading,String>>> {
        std::mem::take(&mut self.pack_readings)
    }
}

pub(super) async fn host_request(host: &HostNet, request: PackRequest, choreography: Vec<Choreography>)
    -> Result<Option<PackReading>,String> {
    let snapshot = host.snapshot().await;
    match request {
        PackRequest::Read => crate::native_pack::read(&snapshot,&host.campaign().await).map(Some),
        PackRequest::Submit(intent) => {
            let input = crate::native_pack::envelope(&snapshot,intent)?;
            host.native_pack(&input).await?;
            Ok(None)
        },
        PackRequest::CurrentContent => {
            let (campaign,_) = crate::native_party::context(&snapshot,isonetry::native_table::TableSender::Host)?;
            let input = crate::native_pack::envelope(&snapshot,
                isocosm_native::native::pack::PackInput::Content { content:campaign.content().clone() })?;
            host.native_pack(&input).await?;
            Ok(None)
        },
        PackRequest::Generated { record,requirements,roles } => {
            host.register_generated_pack(&record,requirements,roles,choreography,GeneratorLimits::default().max_value_depth).await?;
            Ok(None)
        },
    }
}

pub(super) fn deliver(out: &Emitter<BridgeUpdate>, request: RequestId, result: Result<Option<PackReading>,String>, read: bool) {
    match result {
        Ok(Some(view)) => out.emit(BridgeUpdate::PackReading(Correlated::new(request,Ok(view)))),
        Ok(None) => out.emit(BridgeUpdate::CampaignFinished(Correlated::new(request,Ok(())))),
        Err(why) if read => out.emit(BridgeUpdate::PackReading(Correlated::new(request,Err(why)))),
        Err(why) => out.emit(BridgeUpdate::CampaignFinished(Correlated::new(request,Err(why)))),
    }
}
