// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

pub(super) enum BridgeCommand {
    NativeComposedStorylet { request:RequestId, input:isometry_views::StoryletPhysicalRequest },
    NativeInstitution { request:RequestId, input:isometry_views::InstitutionRequest },
    NativeItems { request:RequestId, input:isometry_views::ItemsRequest },
    NativePhysical { request:RequestId, input:isometry_views::PhysicalRequest },
    NativePack { request:RequestId, intent:isometry_views::PackRequest, choreography:Vec<isometry_campaign::native::pack::Choreography> },
    NativeParty { request:RequestId, intent:isometry_views::PartyRequest },
    NativeTime { request:RequestId, intent:isocosm_overlay::vtt::TimeIntent },
    NativeTableAdmission { request: RequestId, input: isonetry::native_table::NativeTableAdmissionInput },
    NativeTableBinding { request: RequestId, input: isonetry::NativeTableBindingInput },
    NativeSetup {
        request: RequestId, setup: isometry_campaign::founding::Setup,
        expected: isometry_campaign::founding::NativeWorld,
        record: GenerationRecord, item_owner: Option<TokenId>,
    },
    NativeCampaign {
        request: RequestId, profile: isocosm::Founding, mode: isocosm::Execution,
        record: GenerationRecord, item_owner: Option<TokenId>,
    },
    Event(GameEvent),
    Travel {
        request: RequestId,
        token: TokenId,
        crossing: isonetry::RequestId,
        party: String,
    },
    /// A *client* asking the host to resolve an action. Carries no verdict.
    Action(ActionIntent),
    Campaign {
        request: RequestId,
        record: GenerationRecord,
        item_owner: Option<TokenId>,
        party: Option<String>,
    },
    Storylet {
        request: RequestId,
        key: String,
        item_owner: Option<TokenId>,
    },
    Whisper {
        to: String,
        text: String,
    },
}

pub(super) enum BridgeUpdate {
    StoryletPhysicalResult(Correlated<Result<isometry_views::StoryletPhysicalOutcome,String>>),
    LocalSender(Option<isonetry::native_table::TableSender>),
    InstitutionResult(Correlated<Result<isometry_views::InstitutionOutcome,String>>),
    ItemsResult(Correlated<Result<isometry_views::ItemsOutcome,String>>),
    PhysicalResult(Correlated<Result<isometry_views::PhysicalOutcome,String>>),
    PackReading(Correlated<Result<isometry_views::PackReading,String>>),
    PartyReading(Correlated<Result<isometry_views::PartyView, String>>),
    NativeEndpoints(Vec<[u8; 32]>),
    HostReady {
        ticket: String,
        snapshot: GameSnapshot,
        campaign: CampaignStore,
        history: Journal<GameEvent>,
    },
    HostState {
        snapshot: GameSnapshot,
        campaign: CampaignStore,
        history: Journal<GameEvent>,
        players: Vec<String>,
    },
    ClientState(GameSnapshot),
    /// Client action requests the host must adjudicate. They surface here rather
    /// than being answered inside the session, because `isonetry` is
    /// rules-blind: only the app holds a `System` that can say whether you hit.
    ActionIntents(Vec<ActionIntent>),
    Whispers(Vec<(String, String)>),
    CampaignFinished(Correlated<Result<(), String>>),
    Failed(String),
}
