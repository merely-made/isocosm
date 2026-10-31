// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

pub(super) enum BridgeCommand {
    NativeComposedStorylet { request:RequestId, input:isocosm_views::StoryletPhysicalRequest },
    NativeInstitution { request:RequestId, input:isocosm_views::InstitutionRequest },
    NativeItems { request:RequestId, input:isocosm_views::ItemsRequest },
    NativePhysical { request:RequestId, input:isocosm_views::PhysicalRequest },
    NativePack { request:RequestId, intent:isocosm_views::PackRequest, choreography:Vec<isocosm_native::native::pack::Choreography> },
    NativeParty { request:RequestId, intent:isocosm_views::PartyRequest },
    NativeTime { request:RequestId, intent:cosmograph_overlay::vtt::TimeIntent },
    NativeTableAdmission { request: RequestId, input: isonetry::native_table::NativeTableAdmissionInput },
    NativeTableBinding { request: RequestId, input: isonetry::NativeTableBindingInput },
    NativeSetup {
        request: RequestId, setup: isocosm_native::founding::Setup,
        expected: isocosm_native::founding::NativeWorld,
        record: GenerationRecord, item_owner: Option<TokenId>,
    },
    NativeCampaign {
        request: RequestId, profile: cosmograph::Founding, mode: cosmograph::Execution,
        record: GenerationRecord, item_owner: Option<TokenId>,
    },
    /// A quick-start pack's datasheet text, founded in one step (case 1041).
    NativeQuickstart { request: RequestId, pack: String, seed: u64, mode: cosmograph::Execution },
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
    StoryletPhysicalResult(Correlated<Result<isocosm_views::StoryletPhysicalOutcome,String>>),
    LocalSender(Option<isonetry::native_table::TableSender>),
    InstitutionResult(Correlated<Result<isocosm_views::InstitutionOutcome,String>>),
    ItemsResult(Correlated<Result<isocosm_views::ItemsOutcome,String>>),
    PhysicalResult(Correlated<Result<isocosm_views::PhysicalOutcome,String>>),
    PackReading(Correlated<Result<isocosm_views::PackReading,String>>),
    PartyReading(Correlated<Result<isocosm_views::PartyView, String>>),
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
