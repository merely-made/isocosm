// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The world rules a new generated world may switch on (lineage-rounds D2,
//! D9, D11, D12; cases 1215, 1216, 1223, 1226). They live here, not in the
//! host, because the ecology admission lays them over its basis as well as
//! over the candidate (case 875's baseline), and the offer is built in this
//! crate.

use serde::{Deserialize, Serialize};

/// The directing rules a new generated world may switch on (lineage-rounds
/// D9, D11, D12; case 1215).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct WorldRules {
    pub drift: bool, pub draft: bool, pub sheet: bool, pub climate: bool, pub regard: bool, pub exertion: bool,
    /// Hosting, and a microbe to hold (cases 1223, 1225, 1226).
    pub hosting: bool,
    /// Members' learning (cases 1051, 1058). A generated world founds no
    /// lineage policy, so this founds one: the smallest variation that
    /// counts as set, and for each lineage that decides, a policy that
    /// scores as the hand-written score does.
    pub learning: bool,
    /// The bound of the climate's turn, in degrees (case 1216); needs the climate.
    pub turn: Option<i32>,
    /// Rates scaled by adult body mass (case 1148): set in the founding, so
    /// it reaches only the A1 world that is generated from it.
    pub mass_law: bool,
    /// Producers' young lie dormant (cases 1192 to 1197): set in the
    /// founding, so it reaches only the A1 world that is generated from it.
    /// The field's absence, in a saved setup, means off, as it did when that
    /// was saved; a newly founded world gets it from [`WorldRules::generated`].
    pub dormancy: bool,
    /// Limits on consumers: they feed only while not full and bear young only
    /// below a per-site ceiling. Set in the founding, so it reaches only the A1
    /// world that is generated from it. Like dormancy, an absent field in a
    /// saved setup means off; a newly founded world gets it from
    /// [`WorldRules::generated`] (ruling D27).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub limits: bool,
    /// Dead bodies give their matter back to the site's soil at a slow rate
    /// without a decomposer. Set in the founding, so it reaches only the A1
    /// world that is generated from it. Like limits, an absent field in a
    /// saved setup means off, and an off rule is not written; a newly founded
    /// world gets it from [`WorldRules::generated`] (owner's ruling,
    /// 2026-11-12).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub rot: bool,
    /// Dead flesh ages from fresh to toxic, and a mouth that eats matter it
    /// does not tolerate is harmed instead of refused (lineage-rounds D36,
    /// D37). Set in the founding, so it reaches only the A1 world that is
    /// generated from it. Off unless asked: an absent field in a saved setup
    /// means off, and an off rule is not written.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub spoilage: bool,
    /// Catalog parts that act at the meal are lowered to kinds the world
    /// declares, which lines reach by editing and drift; none is founded on a
    /// line (lineage-rounds D37). Set in the founding, so it reaches only the
    /// A1 world that is generated from it. Off unless asked: an absent field
    /// in a saved setup means off, and an off rule is not written.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub repertoire: bool,
    /// Tough plant matter harms a mouth with no grinding, and dead matter is
    /// classed as litter, wood or carrion (lineage-rounds D37). Set in the
    /// founding, so it reaches only the A1 world that is generated from it.
    /// Off unless asked: an absent field in a saved setup means off, and an
    /// off rule is not written.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub categories: bool,
    /// Armour, spines and glands (contact harm and toxin on being eaten), each
    /// at a cost, against mouths' attacks and tolerances (lineage-rounds D39,
    /// defence and counter-defence). Set in the founding, so it reaches only
    /// the A1 world that is generated from it. Off unless asked: an absent
    /// field in a saved setup means off, and an off rule is not written.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub guard: bool,
    /// A producer line and a decomposer line trade soil for tissue, each paying
    /// only while the other does (lineage-rounds D39 model 5, mutualism). Set in
    /// the founding, so it reaches only the A1 world generated from it. Off
    /// unless asked: an absent field in a saved setup means off, and an off rule
    /// is not written.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub mutualism: bool,
    /// Young grow from their line's current recipe, so revisions and drift
    /// pass down, instead of copying their parent (lineage-rounds D40). Set in
    /// the founding, so it reaches only the A1 world that is generated from
    /// it. Like rot, an absent field in a saved setup means off (the young
    /// keep copying), and an off rule is not written; a newly founded world
    /// gets it from [`WorldRules::generated`] (owner's ruling, 2026-11-13).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub inherit: bool,
}

thread_local! {
    static MEASURED: std::cell::RefCell<Option<cosmograph::rules::Climate>> = const { std::cell::RefCell::new(None) };
}

/// The climate a world gets: the default, unless a measurement on this thread
/// has put another in place with [`with_climate`].
fn measured_climate() -> cosmograph::rules::Climate {
    MEASURED.with(|held| held.borrow().clone()).unwrap_or_default()
}

/// Runs `body` with `climate` standing in for the default on this thread, for
/// measurements that search the climate's numbers; nothing else uses it.
pub fn with_climate<R>(climate: cosmograph::rules::Climate, body: impl FnOnce() -> R) -> R {
    let before = MEASURED.with(|held| held.replace(Some(climate)));
    let out = body();
    MEASURED.with(|held| *held.borrow_mut() = before);
    out
}

impl WorldRules {
    /// No rule is chosen.
    pub fn is_off(&self) -> bool { *self == Self::default() }

    /// The rules a new generated A1 world founds with unless the player asks
    /// otherwise: the crowding brake (2026-11-11), the consumers' limits
    /// (D27), rot (2026-11-12) and young growing from the recipe (D40,
    /// 2026-11-13) are on, and nothing else.
    /// `default()` and `all()` stay "nothing chosen" because that is what an
    /// absent field in a saved `Setup` meant when it was saved; the product
    /// default lives only here, in the host's starting configuration.
    pub fn generated() -> Self { Self { dormancy: true, limits: true, rot: true, inherit: true, ..Self::default() } }

    /// The three rules of the rounds; the climate is its own flag.
    pub fn all() -> Self { Self { drift: true, draft: true, sheet: true, climate: false, regard: false, exertion: false,
        hosting: false, learning: false, turn: None, mass_law: false, dormancy: false, limits: false, rot: false, spoilage: false, repertoire: false, categories: false, guard: false, mutualism: false, inherit: false }
    }

    /// Sets the chosen rules in a generated genesis; a rule left off stays as found.
    ///
    /// # Errors
    /// `--turn` without the climate, or `--hosting` in a world with no soil
    /// to seed from.
    pub fn apply(&self, genesis: &mut cosmograph::simulation::Genesis) -> Result<(), String> {
        if self.mass_law && genesis.founding.as_ref().is_none_or(|f| f.mass_law.is_none()) {
            return Err("--mass-law is a founding rule: it needs the generated A1 world".into());
        }
        if self.dormancy && genesis.founding.as_ref().is_none_or(|f| f.dormancy.is_none()) {
            return Err("--dormancy is a founding rule: it needs the generated A1 world".into());
        }
        if self.limits && genesis.founding.as_ref().is_none_or(|f| f.limits.is_none()) {
            return Err("--limits is a founding rule: it needs the generated A1 world".into());
        }
        if self.rot && genesis.founding.as_ref().is_none_or(|f| f.rot.is_none()) {
            return Err("--rot is a founding rule: it needs the generated A1 world".into());
        }
        if self.spoilage && genesis.founding.as_ref().is_none_or(|f| f.spoilage.is_none()) {
            return Err("--spoilage is a founding rule: it needs the generated A1 world".into());
        }
        if self.repertoire && genesis.founding.as_ref().is_none_or(|f| f.repertoire.is_none()) {
            return Err("--repertoire is a founding rule: it needs the generated A1 world".into());
        }
        if self.categories && genesis.founding.as_ref().is_none_or(|f| f.categories.is_none()) {
            return Err("--categories is a founding rule: it needs the generated A1 world".into());
        }
        if self.guard && genesis.founding.as_ref().is_none_or(|f| f.guard.is_none()) {
            return Err("--guard is a founding rule: it needs the generated A1 world".into());
        }
        if self.mutualism && genesis.founding.as_ref().is_none_or(|f| f.mutualism.is_none()) {
            return Err("--mutualism is a founding rule: it needs the generated A1 world".into());
        }
        if self.inherit && genesis.founding.as_ref().is_none_or(|f| !f.inherit) {
            return Err("--inherit is a founding rule: it needs the generated A1 world".into());
        }
        if self.turn.is_some() && !self.climate { return Err("--turn needs --climate".into()); }
        if self.climate {
            let climate = cosmograph::rules::Climate { turn: self.turn, ..measured_climate() };
            genesis.rules.climate = Some(climate);
            // The sim adds each lineage's strain process on founding; adding
            // them here, which is idempotent, keeps the witness driver's
            // check that the sim's genesis is the one it was given.
            cosmograph::climate::processes(&mut genesis.rules)?;
        }
        if self.exertion { genesis.rules.exertion = Some(cosmograph::rules::Exertion::default()); }
        if self.hosting { crate::microbe::declare(genesis)?; }
        if self.learning {
            let needs = genesis.rules.mind.as_ref().map_or(0, |m| m.needs.len());
            let directing = genesis.rules.directing.get_or_insert_with(Default::default);
            // One candidate and no variation: the gate learning needs, and
            // nothing drawn, so every policy is the founding one.
            if directing.policy == Default::default() {
                directing.policy = cosmograph::rules::PolicyVariation { founding: 0, expression: 0, candidates: 1 };
            }
            directing.learning = Some(cosmograph::rules::Learning::default());
            for lineage in genesis.lineages.values_mut().filter(|l| l.decides.is_some() && l.policy.is_none()) {
                lineage.policy = Some(cosmograph::directing::policy::LineagePolicy::founding(needs));
            }
        }
        if !(self.drift || self.draft || self.sheet || self.regard) { return Ok(()); }
        let directing = genesis.rules.directing.get_or_insert_with(Default::default);
        if self.drift { directing.drift = true; }
        if self.draft { directing.draft = Some(cosmograph::rules::Draft::default()); }
        if self.sheet { directing.sheet = true; }
        if self.regard { directing.regard = Some(cosmograph::rules::Regard::default()); }
        Ok(())
    }
}
