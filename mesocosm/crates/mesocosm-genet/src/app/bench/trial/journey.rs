// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The bench's built-in journey preset and its read-only door onto the
//! default effect pack.
//!
//! One direction only. Grants come from accepted history through `Trial`'s
//! adapter; this module only ever asks the bound journey what it already
//! owns. Nothing here grants, spends or writes anything, and a drawn mark
//! cannot create a grant.

use crate::section::{GlyphOrientation, SpatialGlyph, stroke};
use mesocosm_core::{
    PartId, World,
    effect_pack::{
        Amount, Bearer, DEFAULT_EFFECT, EffectPackTable, MarkForm, MarkRequest, Refusal,
    },
    embodiment::bearing_parts,
};
use mesocosm_runtime::{
    Trial,
    glyphs::{AcceptedKind, EventGrant, GlyphRules},
};
use serde::Serialize;
use wing_glyphs::{
    CanonSpec, ExpressionSpec, ExpressionTable, GlyphDefinition, GlyphExpression, SCHEMA_VERSION,
};

/// The one base glyph the preset grants, carrying the pack's one effect.
const BASE_GLYPH: &str = "mesocosm:reshape";
const CANON_ID: &str = "mesocosm:bench-trial-canon";
/// One inherent unlock at ten experience, as the runtime example uses.
const UNLOCK_THRESHOLDS: [u64; 1] = [10];

/// Preset rules for the controlled body of `world`. `None` when nothing is
/// controlled; `Trial::new` already refuses such a world.
pub(super) fn default_rules(world: &World) -> Option<GlyphRules> {
    let organism = world.controlled_id()?;
    Some(GlyphRules {
        canon: CanonSpec {
            version: SCHEMA_VERSION,
            limits: Default::default(),
            id: CANON_ID.into(),
            revision: 1,
            glyphs: vec![GlyphDefinition {
                id: BASE_GLYPH.into(),
                display: "#".into(),
                effect: DEFAULT_EFFECT.into(),
            }],
            variants: vec![],
        },
        individual: format!("mesocosm:bench-trial-organism-{}", organism.0),
        organism,
        unlock_thresholds: UNLOCK_THRESHOLDS.to_vec(),
        grants: vec![
            EventGrant {
                event: AcceptedKind::Carved,
                glyph: BASE_GLYPH.into(),
            },
            EventGrant {
                event: AcceptedKind::Moved,
                glyph: BASE_GLYPH.into(),
            },
            EventGrant {
                event: AcceptedKind::Fed,
                glyph: BASE_GLYPH.into(),
            },
            // A producer's feeding earns the same base glyph (ruling 5): the
            // bound body in every trial scenario only ever takes up soil.
            EventGrant {
                event: AcceptedKind::Uptake,
                glyph: BASE_GLYPH.into(),
            },
        ],
    })
}

/// The trial preset's authored expression table.
///
/// Three base glyphs over the five native process ids, authored and **not
/// shuffled**: a world that shuffles pins the permutation into its own
/// definition once (ruling 2), and a disposable trial has no such definition
/// to pin it into. A preset file supplies this table later, exactly as it
/// will supply the canon; until then the bench carries the default so the
/// reading has something to be read against.
///
/// Only `mesocosm:reshape` is in the preset canon. The other two entries are
/// carried so the table is a table rather than a single pair, and so the
/// many-to-many inverse is exercised by the bench and not only by a test.
const EXPRESSION_ID: &str = "mesocosm:bench-trial-expression";
const REACH_GLYPH: &str = "mesocosm:reach";
const GUARD_GLYPH: &str = "mesocosm:guard";

fn expressed(glyph: &str, traits: &[&str]) -> GlyphExpression {
    GlyphExpression {
        glyph: glyph.into(),
        traits: traits.iter().map(|id| (*id).to_string()).collect(),
    }
}

/// The five native definitions of `mesocosm-core`'s registry, as the
/// qualified ids `ProcessId::qualified()` returns. Nothing converts at this
/// boundary: the table keys on the same bytes the registry answers with.
fn default_expression() -> ExpressionTable {
    ExpressionTable::new(ExpressionSpec {
        version: SCHEMA_VERSION,
        id: EXPRESSION_ID.into(),
        canon_revision: 1,
        entries: vec![
            // Taking the world in, either way round: a mass admits material
            // and a plate makes substance out of the world itself.
            expressed(BASE_GLYPH, &["mesocosm:intake", "mesocosm:fix"]),
            // Acting on what is around the body, and reading it.
            expressed(REACH_GLYPH, &["mesocosm:contract", "mesocosm:sense"]),
            // The one definition no shape grows.
            expressed(GUARD_GLYPH, &["mesocosm:secrete"]),
        ],
        limits: Default::default(),
    })
    .expect("the bench preset expression table is admitted")
}

/// The probe's word for what bears the effect's glyph.
pub(super) fn borne_label(borne: Option<Bearer>) -> &'static str {
    match borne {
        Some(Bearer::Embodied) => "embodied",
        Some(Bearer::Journeyed) => "journeyed",
        Some(Bearer::Held) => "held",
        None => "none",
    }
}

/// One owned effect and the act that earned it, for the receipt.
#[derive(Clone, Debug, Serialize)]
struct Owned {
    effect: String,
    acquired_by: Option<AcceptedKind>,
}

/// What the bound journey is and what it has earned. Read-only receipt.
#[derive(Clone, Debug, Serialize)]
pub(super) struct Reading {
    organism: Option<u32>,
    owned: Vec<Owned>,
}

/// One resolved request as a spatial glyph. `age` runs 0 to 1 across the
/// mark's life. The pack named the stroke, colour, form and lifetime; the
/// placement per form is the bench's, as it was before the pack existed.
pub(super) fn glyph(request: &MarkRequest, age: f32, height: f32, size: f32) -> SpatialGlyph {
    let scale = size * f32::from(request.scale_permille) / 1000.;
    let at = request.at.map(|v| v as f32);
    let (centre, angle, orientation) = match request.form {
        // A trail between two recorded places, tied to neither.
        MarkForm::PathTrail => {
            let to = request.to.unwrap_or(request.at).map(|v| v as f32);
            let mut centre = [0, 1, 2].map(|i| at[i] * (1. - age) + to[i] * age);
            centre[1] += height;
            (centre, age * 0.6, GlyphOrientation::CameraFacing)
        },
        // An emission rising off the recipient while the flow continues.
        MarkForm::SustainedEmission => {
            let mut centre = at;
            centre[1] += height + age * scale * 2.;
            (centre, 0., GlyphOrientation::CameraFacing)
        },
        // A stable mark that stays put on the ground.
        MarkForm::SurfaceInscription => {
            let mut centre = at;
            centre[1] += height;
            (
                centre,
                0.,
                GlyphOrientation::WorldPlane {
                    right: [1., 0., 0.],
                    up: [0., 1., 0.],
                },
            )
        },
    };
    SpatialGlyph {
        centre,
        size: scale * (1. - age * 0.5),
        angle,
        glyph: stroke(request.stroke),
        orientation,
        color: request.color.map(|channel| f32::from(channel) / 255.),
    }
}

/// The part face this request wants to sit on, if any.
///
/// Only an inscription has a face: the pack already refuses an anchor to any
/// other bearing, and the absence is what distinguishes a mark **on** a body
/// from one beside it.
pub(super) fn face_of(request: &MarkRequest) -> Option<PartId> {
    (request.form == MarkForm::SurfaceInscription)
        .then_some(request.anchor)
        .flatten()
}

/// The pack the bench executes against the bound journey, and the expression
/// table it reads the body against.
pub(super) struct Binding {
    pack: EffectPackTable,
    expression: ExpressionTable,
}

impl Binding {
    pub(super) fn new() -> Self {
        Self {
            pack: EffectPackTable::default_pack(),
            expression: default_expression(),
        }
    }

    /// Every glyph the bound body currently embodies. The reading is the
    /// runtime's and it is `&`-only in both directions: asking cannot express
    /// anything, and a drawn mark cannot put a site on a part.
    pub(super) fn embodied(&self, driver: &Trial) -> Vec<String> {
        driver.glyphs().map_or_else(Vec::new, |reading| {
            reading
                .embodied_glyphs(driver.world(), &self.expression)
                .into_iter()
                .collect()
        })
    }

    /// The living parts whose sites express the preset's base glyph, in part
    /// order. Empty when nothing embodies it, which is the same answer as
    /// "there is no face to anchor to".
    pub(super) fn expressing_parts(&self, driver: &Trial) -> Vec<PartId> {
        let Some(reading) = driver.glyphs() else {
            return Vec::new();
        };
        let world = driver.world();
        world
            .organisms
            .iter()
            .find(|o| o.id == reading.rules().organism && o.is_alive())
            .map(|o| bearing_parts(&o.phenotype, world.ruleset(), &self.expression, BASE_GLYPH))
            .unwrap_or_default()
    }

    /// What bears the effect's glyph for the bound body, or `None` when
    /// nothing does. **The embodied reading is asked first**: a living part
    /// expressing the glyph is the strongest claim on it, and the journey's
    /// memory answers only when no part does. Read-only.
    pub(super) fn borne_by(&self, driver: &Trial) -> Option<Bearer> {
        let reading = driver.glyphs()?;
        if self.embodied(driver).iter().any(|id| id == BASE_GLYPH) {
            return Some(Bearer::Embodied);
        }
        reading
            .owns_effect(DEFAULT_EFFECT)
            .then_some(Bearer::Journeyed)
    }

    /// Ask the body, then the journey, then the pack. Form, stroke, colour and
    /// lifetime come from **what bears the glyph**, never from the mark's own
    /// kind; placement and amount stay the event's.
    ///
    /// `ordinal` is the marked record's own sequence; it chooses which of the
    /// bearing parts this mark sits on, and nothing else.
    ///
    /// `owned` is the bearer's own answer to "do you have this glyph",
    /// passed by value: the body's for an embodied bearing, the journey's for
    /// a journeyed one. A glyph nothing bears refuses, and the mark is
    /// counted withheld rather than drawn. The pack can reach neither side.
    pub(super) fn resolve(
        &self,
        driver: &Trial,
        ordinal: u64,
        at: [i32; 3],
        to: Option<[i32; 3]>,
        amount: Amount,
    ) -> Result<MarkRequest, Refusal> {
        let reading = driver.glyphs().ok_or_else(|| Refusal::NotAcquired {
            effect: DEFAULT_EFFECT.into(),
        })?;
        let borne_by = self.borne_by(driver).ok_or_else(|| Refusal::NotAcquired {
            effect: DEFAULT_EFFECT.into(),
        })?;
        // The face the pack's inscription sits on, named by part rather than
        // by geometry: core holds no renderer type, and the host reads its own
        // anchor frame off this id.
        //
        // A body usually expresses the glyph on many parts, and the record's
        // own ordinal picks which one this mark sits on, so the inscription
        // is spread across the expressing parts instead of every mark landing
        // on one face. The choice is stable for a given record, so a replay
        // places every mark exactly where the first run did.
        let anchor = (borne_by == Bearer::Embodied)
            .then(|| {
                let parts = self.expressing_parts(driver);
                (!parts.is_empty()).then(|| parts[(ordinal as usize) % parts.len()])
            })
            .flatten();
        let owned = borne_by == Bearer::Embodied || reading.owns_effect(DEFAULT_EFFECT);
        self.pack.resolve_for_glyph(
            reading.journey().canon(),
            BASE_GLYPH,
            owned,
            borne_by,
            at,
            to,
            anchor,
            amount,
        )
    }

    /// The bound body and every effect the pack rules over that its journey
    /// owns, with the accepted act that earned each. Read-only.
    pub(super) fn reading(&self, driver: &Trial) -> Reading {
        let Some(reading) = driver.glyphs() else {
            return Reading {
                organism: None,
                owned: Vec::new(),
            };
        };
        let mut seen = std::collections::BTreeSet::new();
        Reading {
            organism: Some(reading.rules().organism.0),
            owned: self
                .pack
                .rules()
                .iter()
                .filter(|rule| seen.insert(rule.effect.as_str()))
                .filter(|rule| reading.owns_effect(&rule.effect))
                .map(|rule| Owned {
                    effect: rule.effect.clone(),
                    acquired_by: reading.acquired_by(&rule.effect),
                })
                .collect(),
        }
    }

    /// How many grant records the bound journey holds, accepted or refused.
    pub(super) fn grants(&self, driver: &Trial) -> usize {
        driver
            .glyphs()
            .map_or(0, |reading| reading.journey().grants().len())
    }
}
