//! Desktop persistence for Isocosm's private campaign layer.
//!
//! The public map stays a readable JSON document. GM-only campaign state uses
//! Muniment's typed slot over its transactional redb backend, so secrets never
//! share that map file and a later web host can replace redb with OPFS at the
//! same `SlotStore` boundary.

use std::path::Path;

use isocosm_campaign::CampaignStore;
use isonetry::GameSnapshot;
use muniment::{Journal, JsonSlots, RedbBackend};
use serde::{Deserialize, Serialize};

const PRIVATE_CAMPAIGN_SLOT: &str = "isocosm/campaign/private";
const CAMPAIGN_CHECKPOINT_SLOT: &str = "isocosm/campaign/checkpoint";
/// The slots' names before format 3 (case 1040): read when the new slot is
/// empty, written forward, then deleted.
const OLD_PRIVATE_CAMPAIGN_SLOT: &str = "isometry/campaign/private";
const OLD_CAMPAIGN_CHECKPOINT_SLOT: &str = "isometry/campaign/checkpoint";

/// The durable campaign boundary: shared state plus GM-private state in one
/// store transaction. `format` gives future migrations an explicit switch.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CampaignCheckpoint {
    pub format: u32,
    pub public: GameSnapshot,
    pub private: CampaignStore,
    pub history: Journal<isonetry::GameEvent>,
    /// The public state immediately before `history` began. Checkpoints written
    /// before source time omit it and remain readable, but cannot fabricate a
    /// replayable past from their current snapshot.
    #[serde(default)]
    pub history_origin: Option<GameSnapshot>,
}

impl CampaignCheckpoint {
    /// 3 moved the slots from `isometry/` to `isocosm/` (case 1040).
    pub const FORMAT: u32 = 3;

    pub fn new(
        public: GameSnapshot,
        private: CampaignStore,
        history: Journal<isonetry::GameEvent>,
        history_origin: Option<GameSnapshot>,
    ) -> Self {
        Self {
            format: Self::FORMAT,
            public,
            private,
            history,
            history_origin,
        }
    }
}

pub struct CampaignRepository {
    slots: JsonSlots<RedbBackend>,
}

impl CampaignRepository {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let backend = RedbBackend::open(path).map_err(|error| error.to_string())?;
        Ok(Self {
            slots: JsonSlots::new(backend),
        })
    }

    /// Save the authoritative campaign checkpoint. One slot write commits the
    /// public snapshot and private store together in Muniment's backend.
    pub fn save_checkpoint(&self, checkpoint: &CampaignCheckpoint) -> Result<(), String> {
        pollster::block_on(self.slots.save(CAMPAIGN_CHECKPOINT_SLOT, checkpoint))
            .map_err(|error| error.to_string())
    }

    pub fn load_checkpoint(&self) -> Result<Option<CampaignCheckpoint>, String> {
        let checkpoint: Option<CampaignCheckpoint> =
            self.load_moving(CAMPAIGN_CHECKPOINT_SLOT, OLD_CAMPAIGN_CHECKPOINT_SLOT, |old: &mut CampaignCheckpoint| {
                old.format = CampaignCheckpoint::FORMAT;
            })?;
        match checkpoint {
            Some(checkpoint) if (1..=CampaignCheckpoint::FORMAT).contains(&checkpoint.format) => {
                Ok(Some(checkpoint))
            },
            Some(checkpoint) => Err(format!(
                "unsupported campaign checkpoint format: {}",
                checkpoint.format
            )),
            None => Ok(None),
        }
    }

    /// Legacy private-state slot. Existing W0a sidecars load through this while
    /// the public JSON map remains their source of truth.
    pub fn load_private(&self) -> Result<CampaignStore, String> {
        self.load_moving(PRIVATE_CAMPAIGN_SLOT, OLD_PRIVATE_CAMPAIGN_SLOT, |_| {})
            .map(|campaign| campaign.unwrap_or_default())
    }

    /// Reads `slot`, or else the pre-format-3 `old` slot. An old value is
    /// written to `slot` and the old slot deleted only after that write lands.
    fn load_moving<T: Serialize + serde::de::DeserializeOwned>(
        &self,
        slot: &str,
        old: &str,
        upgrade: impl FnOnce(&mut T),
    ) -> Result<Option<T>, String> {
        let error = |error: muniment::StoreError| error.to_string();
        if let Some(value) = pollster::block_on(self.slots.load(slot)).map_err(error)? {
            return Ok(Some(value));
        }
        let Some(mut value) = pollster::block_on(self.slots.load::<T>(old)).map_err(error)? else {
            return Ok(None);
        };
        upgrade(&mut value);
        pollster::block_on(self.slots.save(slot, &value)).map_err(error)?;
        pollster::block_on(self.slots.delete(old)).map_err(error)?;
        Ok(Some(value))
    }
}

#[cfg(test)]
#[path = "campaign_store/h7_bench_fixture.rs"]
mod h7_bench_fixture;

#[cfg(test)]
#[path = "campaign_store/region_fixture.rs"]
mod region_fixture;

#[cfg(test)]
mod tests {
    use super::*;
    use isocosm_campaign::{
        EntropyTape, EquipmentSlot, GenValue, GenerationRecord, GeneratorRequest, Inventory,
        ItemId, ItemInstance, ItemProposal, RevealCondition, SecretFact, WorldFact,
    };
    use isocosm_core::{MapDocument, TokenId, TurnList};
    use isonetry::GameEvent;


    #[test]
    fn campaign_checkpoint_survives_reopen() {
        let path = std::env::temp_dir().join(format!(
            "isocosm-campaign-{}-{}.redb",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut campaign = CampaignStore::new();
        campaign.insert_secret(SecretFact {
            id: "sword-01.curse".to_owned(),
            text: "The sword is cursed.".to_owned(),
            tags: vec!["item:sword-01".to_owned()],
            reveal: RevealCondition::Manual,
        });

        let fact = WorldFact {
            id: "history.founding".to_owned(),
            kind: "history".to_owned(),
            text: "The city was founded on a ford.".to_owned(),
            tags: Vec::new(),
        };
        let mut history = Journal::new();
        history.append(GameEvent::Fact(fact.clone()));
        let generation = GenerationRecord {
            id: "generated.river-blade.1".to_owned(),
            request: GeneratorRequest {
                generator: "demo:forge-item".to_owned(),
                args: GenValue::Text {
                    value: "river".to_owned(),
                },
                locks: Default::default(),
            },
            entropy: EntropyTape::from_seed(7).draw(),
            proposal: GenValue::Item {
                item: ItemProposal {
                    template: "demo:river-blade".to_owned(),
                    name: "River Blade".to_owned(),
                    tags: vec!["fixture".to_owned()],
                },
            },
        };
        history.append(GameEvent::Generation(generation.clone()));
        let mut inventory = Inventory::default();
        inventory
            .insert(ItemInstance {
                id: ItemId::new("checkpoint.sword"),
                template: "srd5e:longsword".to_owned(),
                name: "Checkpoint Sword".to_owned(),
                quantity: 1,
                tags: Vec::new(),
                modifiers: Vec::new(),
                appearance_layers: vec!["weapon:longsword".to_owned()],
            })
            .unwrap();
        inventory
            .equip(EquipmentSlot::MainHand, ItemId::new("checkpoint.sword"))
            .unwrap();
        let public = GameSnapshot {
            map: MapDocument::new("checkpoint", 2, 2),
            turns: TurnList::new(),
            roll_log: Vec::new(),
            journal: vec![fact],
            inventories: std::collections::BTreeMap::from([(TokenId(1), inventory)]),
            generations: vec![generation],
            maps: Default::default(),
            active_map: None,
            world: Default::default(),
            native: None,
            native_table: None,
            clocks: Default::default(),

            party_cap: isonetry::default_party_cap(),
            last_beats: Vec::new(),
            beat_seq: 0,
            applied_actions: Default::default(),
        };
        let checkpoint = CampaignCheckpoint::new(public.clone(), campaign, history, Some(public));

        CampaignRepository::open(&path)
            .unwrap()
            .save_checkpoint(&checkpoint)
            .unwrap();
        let restored = CampaignRepository::open(&path)
            .unwrap()
            .load_checkpoint()
            .unwrap();
        assert_eq!(restored, Some(checkpoint.clone()));

        // A format-2 save under the old slot name reads once, moves, and is
        // stamped format 3; the old slot is gone afterwards (case 1040).
        let repository = CampaignRepository::open(&path).unwrap();
        let mut old = checkpoint.clone();
        old.format = 2;
        pollster::block_on(repository.slots.delete(CAMPAIGN_CHECKPOINT_SLOT)).unwrap();
        pollster::block_on(repository.slots.save(OLD_CAMPAIGN_CHECKPOINT_SLOT, &old)).unwrap();
        assert_eq!(repository.load_checkpoint().unwrap(), Some(checkpoint));
        let gone: Option<CampaignCheckpoint> =
            pollster::block_on(repository.slots.load(OLD_CAMPAIGN_CHECKPOINT_SLOT)).unwrap();
        assert!(gone.is_none());
        drop(repository);

        std::fs::remove_file(path).unwrap();
    }
}
