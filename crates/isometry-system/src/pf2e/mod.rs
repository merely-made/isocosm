//! Pathfinder 2e. The skeleton (ability scores, a Strike ladder) stays as it
//! was; Player Core Remaster creation is built beside it (case 1026): records
//! in `data/pf2e/*.toml`, a draft walked one explained choice at a time, and
//! the numbers derived from the finished choices rather than stored.

mod skeleton;
pub use skeleton::pf2e_srd;

pub mod derive;
pub mod draft;
pub mod records;

/// The opaque system key the skeleton and effect packs already use (case
/// 1026: it stays, with a correct label).
pub const SYSTEM: &str = "pf2e-srd";
/// What the table shows for that key.
pub const LABEL: &str = "Pathfinder 2e Player Core (Remaster)";
