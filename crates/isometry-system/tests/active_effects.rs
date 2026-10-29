#[path = "../examples/effect_demo/mod.rs"]
mod demo;
use demo::*;
use isometry_campaign::{EquipmentSlot, ItemId, ItemModifier, ItemModifierKind};
use isometry_core::{FieldValue, HeightSightRules, MapDocument, Rng, TokenId};
use isometry_system::{System, active_effects::*, pf2e_srd, sheet_with_turn_counters, srd_5e};

mod active_effects {
    mod casting;
    mod consumers;
    mod lifecycle;
    mod state;
}
