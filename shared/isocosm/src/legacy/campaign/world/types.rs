//! The campaign world: its asserted entries and their fold, storylets, and
//! the table's own state.
//!
//! Split out of `world.rs` on 2026-07-24; behavior unchanged.

use super::*;

/// The campaign's world: the asserted entries its session carries and
/// their fold (768), beside the table's own state.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Carried", into = "Carried")]
pub struct CampaignWorld {
    assertions: Vec<Assertion>,
    asserted: Asserted,
    pub storylets: BTreeMap<String, StoryletProposal>,
    pub faction_sheets: BTreeMap<String, BTreeMap<String, i64>>,
    pub faction_control: BTreeMap<String, String>,
    pub party_node: BTreeMap<String, String>,
    pub party_pace: BTreeMap<String, i64>,
    pub party_known: BTreeMap<String, BTreeSet<String>>,
    pub party_resources: BTreeMap<String, BTreeMap<String, i64>>,
}

/// What a campaign world saves and replicates: the entries, not the fold.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Carried {
    #[serde(default)]
    assertions: Vec<Assertion>,
    #[serde(default)]
    storylets: BTreeMap<String, StoryletProposal>,
    #[serde(default)]
    faction_sheets: BTreeMap<String, BTreeMap<String, i64>>,
    #[serde(default)]
    faction_control: BTreeMap<String, String>,
    #[serde(default)]
    party_node: BTreeMap<String, String>,
    #[serde(default)]
    party_pace: BTreeMap<String, i64>,
    #[serde(default)]
    party_known: BTreeMap<String, BTreeSet<String>>,
    #[serde(default)]
    party_resources: BTreeMap<String, BTreeMap<String, i64>>,
}

impl TryFrom<Carried> for CampaignWorld {
    type Error = String;
    fn try_from(c: Carried) -> Result<Self, String> {
        let asserted = Asserted::fold(&c.assertions).map_err(|r| format!("{r:?}"))?;
        Ok(Self {
            assertions: c.assertions,
            asserted,
            storylets: c.storylets,
            faction_sheets: c.faction_sheets,
            faction_control: c.faction_control,
            party_node: c.party_node,
            party_pace: c.party_pace,
            party_known: c.party_known,
            party_resources: c.party_resources,
        })
    }
}

impl From<CampaignWorld> for Carried {
    fn from(w: CampaignWorld) -> Self {
        Self {
            assertions: w.assertions,
            storylets: w.storylets,
            faction_sheets: w.faction_sheets,
            faction_control: w.faction_control,
            party_node: w.party_node,
            party_pace: w.party_pace,
            party_known: w.party_known,
            party_resources: w.party_resources,
        }
    }
}

impl CampaignWorld {
    /// The asserted entries, in the order they were accepted.
    pub fn assertions(&self) -> &[Assertion] {
        &self.assertions
    }

    /// Their fold.
    pub fn asserted(&self) -> &Asserted {
        &self.asserted
    }

    pub fn factions(&self) -> &BTreeMap<String, Faction> {
        &self.asserted.factions
    }

    pub fn places(&self) -> &BTreeMap<String, Place> {
        &self.asserted.places
    }

    pub fn characters(&self) -> &BTreeMap<String, Character> {
        &self.asserted.characters
    }

    pub fn routes(&self) -> &BTreeMap<String, Route> {
        &self.asserted.routes
    }

    pub fn laws(&self) -> &BTreeMap<String, Law> {
        &self.asserted.laws
    }

    /// History lines in the table's time order.
    pub fn history(&self) -> &[HistoryLine] {
        &self.asserted.history
    }

    /// Edits the entries and refolds them: a draft's edit before it is
    /// committed. Refused, the world is left with what refolded.
    pub fn edit(&mut self, f: impl FnOnce(&mut Vec<Assertion>)) -> Result<(), WorldError> {
        let mut entries = std::mem::take(&mut self.assertions);
        f(&mut entries);
        self.asserted = Asserted::default();
        entries.into_iter().try_for_each(|entry| self.assert(entry))
    }

    /// Asserts one entry: carried when it is new, nothing when it is held
    /// already, refused when something else is held under its key.
    pub fn assert(&mut self, assertion: Assertion) -> Result<(), WorldError> {
        let new = self.asserted.apply(&assertion).map_err(|r| match r {
            Refused::Keyless => WorldError::MissingId,
            Refused::Otherwise(key) => WorldError::ConflictingId(key),
            Refused::Unplaced(key) => WorldError::UnknownRouteEndpoint(key),
        })?;
        if new {
            self.assertions.push(assertion);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoryletRequirements {
    /// Every tag must be carried by at least one committed faction.
    #[serde(default)]
    pub faction_tags: Vec<String>,
    /// IDs are checked against the host-private store, without exposing text.
    #[serde(default)]
    pub hidden_facts: Vec<String>,
    #[serde(default)]
    pub world_laws: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleSlot {
    pub key: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StoryletEffect {
    Fact { fact: WorldFact },
    History { event: HistoryLine },
    Item { item: ItemProposal },
    LocalMap { map: LocalMapProposal },
}

// Storylets remain authored as tagged JSON. Their accepted proposals travel in
// the public session history, where postcard needs an externally tagged enum.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(remote = "StoryletEffect", tag = "type", rename_all = "snake_case")]
enum StoryletEffectJson {
    Fact { fact: WorldFact },
    History { event: HistoryLine },
    Item { item: ItemProposal },
    LocalMap { map: LocalMapProposal },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(remote = "StoryletEffect")]
enum StoryletEffectWire {
    Fact { fact: WorldFact },
    History { event: HistoryLine },
    Item { item: ItemProposal },
    LocalMap { map: LocalMapProposal },
}

impl Serialize for StoryletEffect {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            StoryletEffectJson::serialize(self, serializer)
        } else {
            StoryletEffectWire::serialize(self, serializer)
        }
    }
}

impl<'de> Deserialize<'de> for StoryletEffect {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            StoryletEffectJson::deserialize(deserializer)
        } else {
            StoryletEffectWire::deserialize(deserializer)
        }
    }
}

/// A quality-based narrative opportunity. Matching and casting are pure;
/// committing each effect remains an explicit host operation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoryletProposal {
    pub key: String,
    pub entry: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub requirements: StoryletRequirements,
    #[serde(default)]
    pub roles: Vec<RoleSlot>,
    #[serde(default)]
    pub effects: Vec<StoryletEffect>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoryletResolution {
    pub cast: BTreeMap<String, String>,
    pub effects: Vec<StoryletEffect>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StoryletError {
    MissingFactionTag(String),
    MissingHiddenFact(String),
    MissingWorldLaw(String),
    UncastRole(String),
}
