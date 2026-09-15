// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The execution table for a default effect pack, beside `effect_experiment`.
//!
//! One authored rule per `(effect, bearer)` pair says how an owned effect
//! returns. **The axis is what bears the glyph, not which act earned it**
//! (ruled 2026-09-15): a glyph is had, not performed, so the question a form
//! answers is whether a living part expresses it, whether the journey
//! remembers it, or whether an item holds it.
//! `resolve` emits a presentation-neutral [`MarkRequest`]: it
//! names no renderer type, touches no `World`, holds no `&mut` anything, and
//! cannot reach a `Journey`. Ownership is the caller's answer, passed in by
//! value, so nothing here can grant a glyph or change one saved byte.

use crate::body::PartId;
use crate::effect_experiment::Glyph;
use serde::{Deserialize, Serialize};
use wing_glyphs::{
    BehaviourKind, Canon, CostShape, CostUnit, EffectDeclaration, EffectPackSpec, ReceiverClass,
    SCHEMA_VERSION,
};

/// The one base effect the default pack carries its bearer rules on.
pub const DEFAULT_EFFECT: &str = "mesocosm:reshape-reference";
pub const VOXEL_UNIT: &str = "mesocosm:voxel";
pub const MASS_UNIT: &str = "mesocosm:mass_mg";

/// Base mark life for a one-shot return, in simulation ticks.
pub const BASE_LIFETIME_TICKS: u16 = 8;
/// The most an amount may extend a sustained emission, in ticks.
pub const SUSTAINED_LIFETIME_SPAN: u16 = 8;
/// Amount saturation points. Above these the curve is flat, by construction.
/// The mass cap is calibrated on the specimen bench's measured record: accepted
/// meals and soil uptake there run 3 to 28 mg, so a cap in the thousands would
/// spend the curve's whole span on amounts the simulation never records.
pub const VOXEL_CAP: u64 = 64;
pub const MASS_CAP_MG: u64 = 32;
/// `scale_permille` at a zero amount, and the span an amount may add.
pub const BASE_SCALE_PERMILLE: u16 = 1000;
pub const SCALE_SPAN_PERMILLE: u16 = 1000;

/// The journey-rule axis: **what bears the glyph**.
///
/// The manner, not the payload. Which trait bears it and which part expresses
/// it are the reading's answer and travel on [`MarkRequest::anchor`]; a rule
/// is authored against the manner so the `(effect, bearer)` key stays finite
/// and a lookup cannot depend on a table this crate does not hold.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Bearer {
    /// A living, attached part expresses a trait that expresses the glyph.
    /// A fact about a body, and the one that can be lost by being cut.
    Embodied,
    /// Experienced, with no living part expressing it: the journey remembers
    /// it and the body does not bear it. Nothing can take this back.
    Journeyed,
    /// An item, sacrificable at ascension. The borg tier; **declared here and
    /// deliberately unauthored**, so asking for it refuses rather than
    /// borrowing another bearer's form.
    Held,
}

/// The forms a bearer returns as. A host maps these to its own orientation
/// and placement; this enum names no renderer type.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkForm {
    SurfaceInscription,
    SustainedEmission,
    PathTrail,
}

/// One authored rule: this effect, borne this way, returns like this.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackRule {
    pub effect: String,
    pub borne_by: Bearer,
    pub form: MarkForm,
    pub stroke: Glyph,
    pub color: [u8; 4],
    pub cost: CostShape,
    /// Free text naming the bearing this rule answers to. The explanation
    /// template reads it; nothing re-derives it from event or bearer names.
    pub citation: String,
}

/// The amount the accepted record supplies, always an integer already in the
/// record. `Voxels`: `Event::Carved.removed`. `MealMass`: `Event::Fed.mass_mg`.
/// `UptakeMass`: `RecordedFlow.amount_mg` under `Process::Uptake`, which binds
/// to the feeding glyph (ruled 2026-09-15). `None`: `Event::Moved`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Amount {
    Voxels(u32),
    MealMass(u64),
    UptakeMass(u64),
    None,
}

impl Amount {
    pub fn magnitude(self) -> u64 {
        match self {
            Self::Voxels(v) => u64::from(v),
            Self::MealMass(m) | Self::UptakeMass(m) => m,
            Self::None => 0,
        }
    }
    pub fn cap(self) -> u64 {
        match self {
            Self::Voxels(_) => VOXEL_CAP,
            Self::MealMass(_) | Self::UptakeMass(_) => MASS_CAP_MG,
            Self::None => 1,
        }
    }
    pub fn unit(self) -> &'static str {
        match self {
            Self::Voxels(_) => VOXEL_UNIT,
            Self::MealMass(_) | Self::UptakeMass(_) => MASS_UNIT,
            Self::None => "",
        }
    }
    /// A fixed saturating integer curve, never a float and never an RNG draw,
    /// so a replay reproduces it exactly. Monotone, `BASE` at zero, flat at
    /// `BASE + SPAN` from the cap upward.
    pub fn scale_permille(self) -> u16 {
        let cap = self.cap();
        let magnitude = self.magnitude().min(cap);
        let span = u64::from(SCALE_SPAN_PERMILLE);
        BASE_SCALE_PERMILLE + (magnitude * span / cap) as u16
    }
}

/// Presentation-neutral marking order. No renderer type appears here.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MarkRequest {
    pub form: MarkForm,
    pub stroke: Glyph,
    pub color: [u8; 4],
    pub at: [i32; 3],
    /// Movement's arrival, for `PathTrail`. `None` otherwise.
    pub to: Option<[i32; 3]>,
    /// The bearing part to emit from, when one bears it. A host reads its own
    /// anchor frame off this id; core names no renderer type and no face.
    /// `None` for a bearer that is not a part, which is exactly the
    /// distinction between an inscription on a body and a mark beside one.
    pub anchor: Option<PartId>,
    /// 1000 = the preset's base size. Derived from `Amount`, never RNG.
    pub scale_permille: u16,
    pub lifetime_ticks: u16,
    pub explanation: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Refusal {
    NotAcquired { effect: String },
    NoRule { effect: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectPackTable {
    rules: Vec<PackRule>,
}

fn unit(id: &str, label: &str) -> CostUnit {
    CostUnit {
        id: id.into(),
        label: label.into(),
    }
}

impl EffectPackTable {
    pub fn new(rules: Vec<PackRule>) -> Result<Self, String> {
        let table = Self { rules };
        table.validate()?;
        Ok(table)
    }

    /// The authored rules on one glyph, keyed by what bears it: a glyph a
    /// living part expresses is inscribed on that part's surface, and one the
    /// journey remembers without a bearer emits beside the body for as long
    /// as it is sustained. [`Bearer::Held`] is the borg tier and is
    /// deliberately unauthored, so it refuses rather than drawing.
    pub fn default_pack() -> Self {
        Self {
            rules: vec![
                PackRule {
                    effect: DEFAULT_EFFECT.into(),
                    borne_by: Bearer::Embodied,
                    form: MarkForm::SurfaceInscription,
                    stroke: Glyph::Slashes,
                    color: [255, 115, 51, 255],
                    cost: CostShape::PerUse {
                        unit: unit(VOXEL_UNIT, "removed voxels"),
                    },
                    citation: "the living part expressing it, read off the phenotype                         against the expression table at this canon revision"
                        .into(),
                },
                PackRule {
                    effect: DEFAULT_EFFECT.into(),
                    borne_by: Bearer::Journeyed,
                    form: MarkForm::SustainedEmission,
                    stroke: Glyph::Quotes,
                    color: [255, 191, 77, 255],
                    cost: CostShape::WhileSustained {
                        unit: unit(MASS_UNIT, "milligrams"),
                    },
                    citation: "the journey's first acquisition                         mesocosm.trial/{baseline}/{post}/history/{sequence} and its canon revision"
                        .into(),
                },
            ],
        }
    }

    /// The shared declaration for what `default_pack` executes (D1). Data
    /// only: wing-glyphs admits and validates it; it runs nothing.
    pub fn default_declarations() -> EffectPackSpec {
        EffectPackSpec {
            version: SCHEMA_VERSION,
            id: "mesocosm:default-effect-pack".into(),
            revision: 1,
            declarations: vec![EffectDeclaration {
                effect: DEFAULT_EFFECT.into(),
                display: "#".into(),
                behaviour: BehaviourKind::Inscribe,
                receiver: ReceiverClass::Terrain,
                cost: CostShape::PerUse {
                    unit: unit(VOXEL_UNIT, "removed voxels"),
                },
                explanation: "{actor} returns {effect} on {receiver} for {amount} {unit}".into(),
            }],
            limits: Default::default(),
        }
    }

    pub fn rules(&self) -> &[PackRule] {
        &self.rules
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.rules.is_empty() {
            return Err("effect pack table requires at least one rule".into());
        }
        let mut seen = std::collections::BTreeSet::new();
        for rule in &self.rules {
            if rule.effect.is_empty() || rule.citation.is_empty() {
                return Err("pack rule requires an effect ID and a citation".into());
            }
            if !seen.insert((rule.effect.as_str(), rule.borne_by)) {
                return Err(format!(
                    "duplicate pack rule for {} borne by {:?}",
                    rule.effect, rule.borne_by
                ));
            }
        }
        Ok(())
    }

    /// Every rule's effect is declared in `pack`. Coverage only: one effect
    /// carries several bearer rules with different cost shapes, so the
    /// declaration's single cost is not a rule's cost and is not compared.
    pub fn validate_against(&self, pack: &wing_glyphs::EffectPack) -> Result<(), String> {
        self.validate()?;
        for rule in &self.rules {
            if pack.declaration(&rule.effect).is_none() {
                return Err(format!("effect pack declares no {}", rule.effect));
            }
        }
        Ok(())
    }

    pub fn rule(&self, effect: &str, borne_by: Bearer) -> Option<&PackRule> {
        self.rules
            .iter()
            .find(|r| r.effect == effect && r.borne_by == borne_by)
    }

    /// `owned` is the journey's answer, passed in. This function cannot reach
    /// a `Journey`, cannot grant, and cannot mutate anything.
    #[allow(clippy::too_many_arguments)]
    pub fn resolve(
        &self,
        effect: &str,
        owned: bool,
        borne_by: Bearer,
        at: [i32; 3],
        to: Option<[i32; 3]>,
        anchor: Option<PartId>,
        amount: Amount,
    ) -> Result<MarkRequest, Refusal> {
        if !owned {
            return Err(Refusal::NotAcquired {
                effect: effect.to_owned(),
            });
        }
        let rule = self.rule(effect, borne_by).ok_or(Refusal::NoRule {
            effect: effect.to_owned(),
        })?;
        let scale_permille = amount.scale_permille();
        let lifetime_ticks = match rule.cost {
            CostShape::Free | CostShape::PerUse { .. } => BASE_LIFETIME_TICKS,
            // A sustained emission is refreshed each tick the flow continues;
            // the caller retires it on a one-tick gap. The amount only says
            // how long one unretired refresh lasts.
            CostShape::WhileSustained { .. } => {
                BASE_LIFETIME_TICKS
                    + ((scale_permille - BASE_SCALE_PERMILLE) * SUSTAINED_LIFETIME_SPAN
                        / SCALE_SPAN_PERMILLE)
            },
        };
        let explanation = format!(
            "{:?} borne by {:?} returns as {:?}; cited to {}. {} {} sets scale {}‰ over {} ticks.",
            rule.stroke,
            borne_by,
            rule.form,
            rule.citation,
            amount.magnitude(),
            if amount.unit().is_empty() {
                "(no metered amount)"
            } else {
                amount.unit()
            },
            scale_permille,
            lifetime_ticks
        );
        Ok(MarkRequest {
            form: rule.form,
            stroke: rule.stroke,
            color: rule.color,
            at,
            to: if rule.form == MarkForm::PathTrail {
                to
            } else {
                None
            },
            // Only a bearing part can be anchored to. A journeyed mark has no
            // face to sit on, and that absence is the whole distinction.
            anchor: (rule.borne_by == Bearer::Embodied)
                .then_some(anchor)
                .flatten(),
            scale_permille,
            lifetime_ticks,
            explanation,
        })
    }

    /// The revision-aware door (R7). The live effect of an owned glyph follows
    /// the *current* canon, not the effect recorded at acquisition, so one
    /// glyph acquired under two revisions does not wrongly draw the same mark.
    #[allow(clippy::too_many_arguments)]
    pub fn resolve_for_glyph(
        &self,
        canon: &Canon,
        glyph: &str,
        owned: bool,
        borne_by: Bearer,
        at: [i32; 3],
        to: Option<[i32; 3]>,
        anchor: Option<PartId>,
        amount: Amount,
    ) -> Result<MarkRequest, Refusal> {
        let effect = canon.effect(glyph).ok_or(Refusal::NoRule {
            effect: glyph.to_owned(),
        })?;
        self.resolve(effect, owned, borne_by, at, to, anchor, amount)
    }
}

impl Default for EffectPackTable {
    fn default() -> Self {
        Self::default_pack()
    }
}

#[cfg(test)]
mod tests;
