// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

impl HostNet {
    pub async fn quote_native_institution(&self,input:&crate::NativeInstitutionInput)
        ->Result<crate::NativeInstitutionQuote,String> {
        self.session.lock().await.quote_native_institution(crate::native_table::TableSender::Host,input)
    }
    pub async fn native_institution(&self,input:&crate::NativeInstitutionInput)->Result<(),String> {
        let out=self.session.lock().await.native_institution(crate::native_table::TableSender::Host,input)?;
        dispatch(&self.peers,out).await;
        Ok(())
    }
    pub async fn quote_native_items(&self,input:&isocosm_native::native::items::ItemInput)
        ->Result<isocosm_native::native::items::ItemPreview,String> {
        self.session.lock().await.quote_native_items(crate::native_table::TableSender::Host,input)
    }
    pub async fn native_items(&self,input:&isocosm_native::native::items::ItemInput)->Result<(),String> {
        let out=self.session.lock().await.native_items(crate::native_table::TableSender::Host,input)?;
        dispatch(&self.peers,out).await;
        Ok(())
    }
    pub async fn quote_native_physical(&self, input: &isocosm_native::native::physical_map::PhysicalMapInput)
        -> Result<isocosm_native::native::physical_map::PhysicalMapPreview, String> {
        self.session.lock().await.quote_native_physical(input)
    }
    pub async fn native_physical(&self, input: &isocosm_native::native::physical_map::PhysicalMapInput)
        -> Result<(), String> {
        let out = self.session.lock().await.native_physical(input)?;
        dispatch(&self.peers, out).await;
        Ok(())
    }
    pub async fn native_pack(&self,input:&isocosm_native::native::pack::PackEnvelope)->Result<(),String> {
        let out=self.session.lock().await.native_pack(crate::native_table::TableSender::Host,input)?;
        dispatch(&self.peers,out).await;
        Ok(())
    }
    pub async fn register_generated_pack(&self,record:&isocosm_campaign::GenerationRecord,
        requirements:isocosm_campaign::StoryletRequirements,roles:Vec<isocosm_campaign::RoleSlot>,
        choreography:Vec<isocosm_native::native::pack::Choreography>,max_depth:usize)->Result<(),String> {
        let out=self.session.lock().await.register_generated_pack(record,requirements,roles,choreography,max_depth)?;
        dispatch(&self.peers,out).await;
        Ok(())
    }
    pub async fn native_hook(&self,envelope:&cosmograph_overlay::vtt::VttIntentEnvelope)->Result<(),String> {
        let out=self.session.lock().await.native_hook(envelope)?;
        dispatch(&self.peers,out).await;
        Ok(())
    }
    pub async fn native_journey(&self, input: &isocosm_native::native::joint_journey::JourneyEnvelope) -> Result<(), String> {
        let out = self.session.lock().await.native_journey(crate::native_table::TableSender::Host, input)?;
        dispatch(&self.peers, out).await;
        Ok(())
    }

    pub async fn native_party(&self, envelope: &cosmograph_overlay::vtt::VttIntentEnvelope) -> Result<(), String> {
        let out = self.session.lock().await.native_party(crate::native_table::TableSender::Host, envelope)?;
        dispatch(&self.peers, out).await;
        Ok(())
    }

    pub async fn native_location(&self,input:&isocosm_native::native::party::LocationInput)->Result<(),String> {
        let out=self.session.lock().await.native_location(input)?;
        dispatch(&self.peers,out).await;
        Ok(())
    }

    pub async fn native_time(&self, envelope: &cosmograph_overlay::vtt::VttIntentEnvelope) -> Result<(), String> {
        let out = self.session.lock().await.native_time(crate::native_table::TableSender::Host, envelope)?;
        dispatch(&self.peers, out).await;
        Ok(())
    }
    pub async fn commit_native_table_admission(&self, input: &crate::native_table::NativeTableAdmissionInput) -> Result<(), String> {
        let out = self.session.lock().await.commit_native_table_admission(input)?;
        dispatch(&self.peers, out).await;
        Ok(())
    }

    pub async fn authenticated_endpoints(&self) -> Vec<[u8; 32]> {
        self.session.lock().await.authenticated_endpoints()
    }

    pub async fn commit_native_table_binding(&self, input: &crate::NativeTableBindingInput) -> Result<(), String> {
        let out = self.session.lock().await.commit_native_table_binding(input)?;
        dispatch(&self.peers, out).await;
        Ok(())
    }
    pub async fn commit_native_setup(
        &self, setup: &isocosm_native::founding::Setup,
        expected: &isocosm_native::founding::NativeWorld,
        record: &isocosm_campaign::GenerationRecord,
        item_owner: Option<isocosm_core::TokenId>,
    ) -> Result<(), String> {
        let out = self.session.lock().await.commit_native_setup(setup, expected, record, item_owner)?;
        dispatch(&self.peers, out).await;
        Ok(())
    }

    pub async fn commit_native_quickstart(&self, pack: &str, seed: u64, mode: cosmograph::Execution)
        -> Result<(), String> {
        let out = self.session.lock().await.commit_native_quickstart(pack, seed, mode)?;
        dispatch(&self.peers, out).await;
        Ok(())
    }

    pub async fn commit_native_campaign(
        &self,
        profile: &cosmograph::Founding,
        record: &isocosm_campaign::GenerationRecord,
        item_owner: Option<isocosm_core::TokenId>,
        mode: cosmograph::Execution,
    ) -> Result<(), String> {
        let out = self.session.lock().await
            .commit_native_campaign(profile, record, item_owner, mode)?;
        dispatch(&self.peers, out).await;
        Ok(())
    }
}

impl ClientNet {
    pub async fn native_institution(&self,input:crate::NativeInstitutionInput)->Result<(),String> {
        write_frame(&mut *self.send.lock().await,&crate::NetMessage::NativeInstitution(input)).await
    }
    pub async fn native_items(&self,input:isocosm_native::native::items::ItemInput)->Result<(),String> {
        write_frame(&mut *self.send.lock().await,&crate::NetMessage::NativeItem(input)).await
    }
}
