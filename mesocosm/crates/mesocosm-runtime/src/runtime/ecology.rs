// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Explicit generated-world offer gate. Structural generation is only a candidate.

use super::{Founded, Runtime};
use crate::spatial::{Game, Setup as SpatialSetup};
use isocosm::{Execution, directing::is_participant, ecology::{
    BirthRequirement, CaptureRequirement, CaptureStock, CounterProof, CounterRequirement,
    IndependenceSpec, JointSpec, JointWitness, ReturnRequirement, Selection, run_joint_validated,
}};
use serde::{Deserialize, Serialize};

mod offer;
mod seal;
pub use seal::{Seal,SealData};

fn has_living_role(genesis: &isocosm::simulation::Genesis, lineage: &str, role: &str) -> bool {
    genesis.population.groups.values().any(|group| {
        group.count > 0
            && group.entity.alive
            && !is_participant(&group.entity)
            && group.entity.lineage == lineage
            && group.entity.traits.contains(role)
    })
}

#[derive(Debug)]
pub enum GeneratedSetupError { MissingViableConsumerFood, Invalid(String) }

fn generated_specification(genesis: &isocosm::simulation::Genesis) -> Result<JointSpec, GeneratedSetupError> {
    let founding = genesis.founding.as_ref().ok_or_else(|| GeneratedSetupError::Invalid("generated ecology needs its founding".into()))?;
    let producers: Vec<u32> = (0..founding.lineages)
        .filter(|index| {
            let lineage = format!("lineage:{index}");
            has_living_role(genesis, &lineage, "life:producer")
                && genesis.rules.processes.contains_key(&format!("ecology:produce-{index}"))
                && genesis.rules.processes.contains_key(&format!("ecology:upkeep-{index}"))
                && genesis.rules.processes.contains_key(&format!("ecology:birth-{index}"))
        })
        .collect();

    let played = founding.played.map(|played| played.lineage);
    let mut viable = Vec::new();
    for consumer in 0..founding.lineages {
        let consumer_lineage = format!("lineage:{consumer}");
        if !has_living_role(genesis, &consumer_lineage, "life:consumer") {
            continue;
        }
        let Some(feed) = genesis.rules.processes.get(&format!("ecology:feed-{consumer}")) else {
            continue;
        };
        let Some(target) = feed.target.as_ref().filter(|target| target.alive == Some(true)) else {
            continue;
        };
        for &producer in &producers {
            let producer_lineage = format!("lineage:{producer}");
            if (target.among.is_empty() || target.among.contains(&producer_lineage))
                && target.lineage.as_ref().is_none_or(|lineage| lineage == &producer_lineage)
            {
                viable.push((consumer != played.unwrap_or(u32::MAX), consumer, producer));
            }
        }
    }
    viable.sort_unstable();
    let (_, consumer, producer) = viable.into_iter().next()
        .ok_or(GeneratedSetupError::MissingViableConsumerFood)?;
    let selection = |process: String, actor: u32, target: Option<u32>| Selection {
        process,
        actor_lineage: format!("lineage:{actor}"),
        target_lineage: target.map(|lineage| format!("lineage:{lineage}")),
    };
    Ok(JointSpec {
        version: 1,
        key: "mesocosm:default-875".into(),
        horizon: 60,
        capture: CaptureRequirement {
            selection: selection(format!("ecology:produce-{producer}"), producer, None),
            stock: CaptureStock::EnergyAccount("sim:energy".into()),
            at_least: 1,
        },
        returned: ReturnRequirement {
            selection: selection(format!("ecology:upkeep-{producer}"), producer, None),
            account: "world:soil".into(),
            at_least: 1,
        },
        reproduction: BirthRequirement {
            selection: selection(format!("ecology:birth-{producer}"), producer, None),
            at_least: 1,
        },
        counterplay: CounterRequirement {
            selection: selection(format!("ecology:feed-{consumer}"), consumer, Some(producer)),
            proof: CounterProof::TargetDebited {
                account: format!("matter:{producer}-0"),
                at_least: 1,
            },
        },
        // The candidate is compared with a fresh basis from the same founding;
        // no extra native definition is being claimed.
        independence: Some(IndependenceSpec {
            version: 1,
            extras: vec![],
            scalars: vec![],
        }),
    })
}

fn is_false(value: &bool) -> bool { !*value }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Setup {
    pub version: u32,
    /// Exact generated candidate, including authored role declarations and lifecycle settings.
    pub founded: Founded,
    pub specification: JointSpec,
    pub execution: Execution,
    /// Explicit physical inputs; absent retains the authored fixture's historical site-grain mode.
    pub spatial: Option<SpatialSetup>,
    /// Realize generator-owned templates. Older setup files omit this field
    /// and retain their historical site-grain meaning.
    #[serde(default, skip_serializing_if = "is_false")]
    pub generated_templates: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Assessment {
    pub witness: JointWitness,
    pub spatial: Game,
}

impl Setup {
    /// The product's bounded generator-authored 875 witness. The 60-tick
    /// horizon is the existing native 890 qualification horizon, not an
    /// inference from Mesocosm's much longer epoch length.
    pub fn generated(founded: Founded, execution: Execution) -> Result<Self, String> {
        Self::generated_classified(founded, execution).map_err(|error| match error {
            GeneratedSetupError::MissingViableConsumerFood => "default generated ecology offer needs a living producer and a living consumer whose native feed targets it".into(),
            GeneratedSetupError::Invalid(message) => message,
        })
    }

    pub fn generated_classified(founded: Founded, execution: Execution) -> Result<Self, GeneratedSetupError> {
        if founded.genesis.founding.as_ref().is_none_or(|f| f.preset.is_some()
            || f.affinity.is_some() || f.ecology_rates != isocosm::ecology::Rates::A1) {
            return Err(GeneratedSetupError::Invalid("default ecology offer requires an ordinary A1 founding without preset or affinity".into()));
        }
        let specification = generated_specification(&founded.genesis)?;
        let mut founded = founded;
        founded.genesis.rules.ecology_witness = Some(specification.clone());
        founded.genesis.validate().map_err(GeneratedSetupError::Invalid)?;
        Ok(Self { version: 1, founded, specification, execution, spatial: None, generated_templates: true })
    }

    pub fn check(&self) -> Result<(), String> {
        if self.version != 1 || self.founded.genesis.founding.is_none() {
            return Err("ecology offer setup must name an exact generated candidate at version 1".into());
        }
        if self.founded.pace.round == 0 || self.founded.pace.scoring == 0 {
            return Err("ecology offer requires explicitly positive lifecycle pace".into());
        }
        if self.generated_templates && self.spatial.is_some() {
            return Err("ecology offer must name one physical setup source".into());
        }
        self.founded.genesis.validate()?;
        self.specification.validate_genesis(&self.founded.genesis)?;
        if self.founded.genesis.rules.ecology_witness.as_ref() != Some(&self.specification) {
            return Err("ecology offer and native rule specifications differ".into());
        }
        Ok(())
    }

    /// Runs the actual supplied clock provider over one native trajectory.
    /// The same trajectory supplies the native evidence required before admission.
    pub fn assess(&self) -> Result<Assessment, String> {
        let (run, spatial) = self.assess_validated(&mut |_, _| {})?;
        let (witness, _) = run.into_parts();
        Ok(Assessment { witness, spatial })
    }

    fn assess_validated(&self, progress: &mut dyn FnMut(u64, u64))
        -> Result<(isocosm::ecology::ValidatedJointRun, Game), String> {
        self.check()?;
        let mut game = Game::default();
        let mut initialized = false;
        let mut physical = false;
        let run = run_joint_validated(&self.founded.genesis, &self.specification, self.execution, &mut |session| {
            if !initialized {
                if let Some(setup) = &self.spatial {
                    game.setup(session, setup)?;
                    physical = true;
                } else if self.generated_templates {
                    // Ruling 889's generator-owned stands and routes provide
                    // the default physical source; no coordinates are chosen here.
                    if !game.found_generated(session)? {
                        return Err("default generated ecology offer requires generator-declared physical templates".into());
                    }
                    physical = true;
                }
                initialized = true;
            }
            let result = if physical { game.advance_clock(session)? }
                else { session.advance_tick_with_flows()? };
            progress(result.tick, self.specification.horizon);
            Ok(result)
        })?;
        if physical { game.validate_replay(run.session())?; }
        Ok((run, game))
    }

    pub fn offer(&self, ticks_per_second: u32) -> Result<Runtime, String> {
        self.offer_with_progress(ticks_per_second, &mut |_, _| {})
    }

    /// Reports completed live clocks. The final clock precedes native replay
    /// validation; returning successfully still requires the full admission.
    pub fn offer_with_progress(&self, ticks_per_second: u32,
        progress: &mut dyn FnMut(u64, u64)) -> Result<Runtime, String> {
        let founding=self.founded.genesis.founding.as_ref().ok_or("generated candidate has no founding")?;
        // This producer captures its real basis; authored JSON cannot replace it.
        let basis=isocosm::ecology::GeneratedBasis::new(founding)?;
        let criterion=self.specification.independence.as_ref()
            .ok_or("generated-world offer needs explicit native mechanical-effect declarations879")?;
        let (run, spatial) = self.assess_validated(progress)?;
        let admitted = isocosm::ecology::admit_validated(run, criterion, &basis)?;
        offer::accepted(self, spatial, admitted, ticks_per_second)
    }
}

/// An absent marker is compatible for saved loads, but not a fresh generated offer.
pub(super) fn fresh_offer(founded: &Founded) -> Result<(), String> {
    if founded.genesis.founding.is_some() {
        return Err(if founded.genesis.rules.ecology_witness.is_none() {
            "new generated world requires an explicit joint ecology witness before it can be offered".into()
        } else {
            "generated world must pass its joint witness and native mechanical-effect admission879".into()
        });
    }
    Ok(())
}
