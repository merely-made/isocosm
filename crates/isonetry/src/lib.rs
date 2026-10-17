//! The Isocosm family's session layer over one host-owned core World.
//!
//! [`family`] records accepted native history and opaque product sections;
//! replicas apply those records without repeating product admission. The
//! optional `family-net` feature supplies host-signed records over Stickleback
//! alongside Graphshell's admitted request stream.
//!
//! The existing VTT `HostSession` and `ClientSession` state machines,
//! campaign store and proposal APIs remain during migration under ruling 921.
//! Their concrete game-event protocol is separate from the family record API.

#[cfg(feature = "vtt")]
pub use isocosm_vtt::founding::NativeWorld;
#[cfg(feature = "vtt")]
pub use session::{NativePublication, NativeTravelResolved, NativeTimeResolved, NativePartyResolved,
    NativeJourneyResolved, NativeHookResolved, NativePackResolved, NativePhysicalResolved, NativeItemsResolved, TravelMember, TravelTableResult};
#[cfg(feature = "vtt")]
pub use session::{InstitutionSender,NativeInstitutionInput,NativeInstitutionQuote,NativeInstitutionsResolved};
#[cfg(feature = "vtt")]
pub use session::NativeSecretResolved;
#[cfg(feature = "vtt")]
pub use session::NativeComposedStoryletResolved;
#[cfg(feature = "vtt")]
mod mode_game;
#[cfg(feature = "vtt")]
pub use mode_game::ModeGame;
pub mod family;
#[cfg(feature = "family-net")]
pub mod family_network;
#[cfg(feature = "vtt")]
mod protocol;
#[cfg(feature = "vtt")]
pub mod native_table;
#[cfg(feature = "vtt")]
pub use native_table::{NativeTable, NativeTableAdmissionInput, NativeTableBindingInput, NativeTableResolved,
    NativeTokenMove, PeerParticipant, BoardBinding, TokenCharacter, TileCell};
#[cfg(feature = "vtt")]
mod session;
#[cfg(feature = "vtt")]
mod source_time;
#[cfg(feature = "vtt")]
pub mod sim;

#[cfg(feature = "campaign-p2p")]
pub mod campaign_space;

#[cfg(feature = "campaign-p2p-net")]
pub mod campaign_sync;

#[cfg(feature = "campaign-moot")]
pub mod campaign_moot;

#[cfg(feature = "campaign-murm")]
pub mod campaign_secrets;

#[cfg(feature = "vtt")]
pub use protocol::{
    default_party_cap, ActionIntent, ActionResolved, GameEvent, GameSnapshot, NetMessage, Outbound,
    PeerId, Recipient, RequestId, TransitionResolved, PROTOCOL_VERSION, ROLL_LOG_CAP,
};
#[cfg(feature = "vtt")]
pub use session::{
    apply_game, resolve_transition, resolve_transition_for_party, ClientSession, GameError,
    HostSession,
};
#[cfg(feature = "vtt")]
pub use source_time::{GameSourceHistory, GameSourceTimeError, NativeSiteVersion};

#[cfg(feature = "iroh")]
pub mod iroh_link;
