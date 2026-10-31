// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use isocosm_views::{InstitutionRequest,InstitutionOutcome};

impl NetBridge {
    pub fn native_institution(&mut self,input:InstitutionRequest)->Option<RequestId> {
        let request=self.request_ids.issue();
        self.actor.command(BridgeCommand::NativeInstitution {request,input}).then_some(request)
    }
    pub fn take_institution_results(&mut self)->Vec<Correlated<Result<InstitutionOutcome,String>>> {
        std::mem::take(&mut self.institution_results)
    }
}

pub(super) async fn host_request(host:&HostNet,input:InstitutionRequest)->Result<InstitutionOutcome,String> {
    match input {
        InstitutionRequest::Quote(input)=>Ok(InstitutionOutcome::Quote(host.quote_native_institution(&input).await?)),
        InstitutionRequest::Apply(input)=>{host.native_institution(&input).await?;Ok(InstitutionOutcome::Applied)},
    }
}

pub(super) async fn client_request(client:&ClientNet,input:InstitutionRequest)->Result<InstitutionOutcome,String> {
    match input {
        InstitutionRequest::Quote(input)=>{
            let snapshot=client.state().await.ok_or("institution quote needs the authenticated host snapshot")?;
            let (campaign,access)=crate::native_party::context(&snapshot,
                isonetry::native_table::TableSender::Endpoint(client.endpoint_key()))?;
            let table=snapshot.native_table.as_ref().ok_or("institution quote has no actual native table")?;
            if isonetry::native_table::table_digest(Some(table)).as_deref()!=Some(input.expected_table.as_str()) {
                return Err("institution quote names another authenticated table cut".into());
            }
            let preview=isocosm_native::native::institutions::prepare_institution(campaign.table_session(),access,&input.input)?;
            Ok(InstitutionOutcome::Quote(isonetry::NativeInstitutionQuote {
                expected_table:input.expected_table,input:input.input,preview}))
        },
        InstitutionRequest::Apply(input)=>{client.native_institution(input).await?;Ok(InstitutionOutcome::Submitted)},
    }
}
