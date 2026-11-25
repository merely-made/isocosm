// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::Panel;
use cosmograph::{
    Execution, Founding, Session,
    coarse::{Region, RegionalForecast, RegionalModel},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::mpsc::{self, Receiver, TryRecvError},
};

const SAMPLE: u64 = 12;
const HORIZON: u64 = 96;

struct Projection {
    start: BTreeMap<u64, Region>,
    future: RegionalForecast,
    surfaces: BTreeSet<String>,
}

impl Projection {
    fn of(session: Session) -> Result<Self, String> {
        let surfaces = session
            .sim
            .state()
            .spreads
            .identities
            .iter()
            .filter_map(|id| {
                if let cosmograph::spread::Identity::Surface(line) = id {
                    Some(line.clone())
                } else {
                    None
                }
            })
            .collect();
        let model = RegionalModel::observe(&session, SAMPLE)?;
        Ok(Self {
            surfaces,
            start: model.start().clone(),
            future: model.project(HORIZON)?,
        })
    }

    fn describe(&self, site: u64, lineage: &str, name: &str) -> Option<String> {
        let before = self.start.get(&site)?;
        let after = self.future.sites.get(&site)?;
        let count = |r: &Region| r.living.get(lineage).copied().unwrap_or(0);
        let stock = |r: &Region| r.biomass.get(lineage).copied().unwrap_or(0);
        let mut resources: Vec<_> = before
            .resources
            .keys()
            .chain(after.resources.keys())
            .filter(|k| *k == "world:soil" || k.starts_with("world:soil-"))
            .collect();
        resources.sort_unstable();
        resources.dedup();
        let resources = resources
            .into_iter()
            .map(|k| {
                let name = k.trim_start_matches("world:");
                format!(
                    "{name} {} → {} mg",
                    before.resources.get(k).unwrap_or(&0),
                    after.resources.get(k).unwrap_or(&0)
                )
            })
            .collect::<Vec<_>>()
            .join(" · ");
        if self.surfaces.contains(lineage) {
            return Some(format!(
                "{name} · site {site}, {lineage} · surface biomass {} → {} mg · {resources}",
                before.surfaces.get(lineage).unwrap_or(&0),
                after.surfaces.get(lineage).unwrap_or(&0)
            ));
        }
        let traffic = [(true, "gained"), (false, "lost")]
            .into_iter()
            .filter_map(|(gain, verb)| {
                let (process, amount) = self.future.strongest_flow(site, lineage, gain)?;
                let activity = [
                    ("ecology:produce-", "photosynthesis"),
                    ("ecology:feed-", "feeding"),
                    ("ecology:upkeep-", "upkeep"),
                    ("mutualism:exchange-", "nutrient exchange"),
                    ("mutualism:assimilate-", "assimilating sugar"),
                    ("nutrients:tuned", "mineral uptake"),
                    ("guard:rent", "maintaining defences"),
                ]
                .into_iter()
                .find(|(prefix, _)| process.starts_with(prefix))
                .map_or(process, |(_, label)| label);
                Some(format!(
                    " · {verb} {amount} mg via {activity} (largest sampled flow)"
                ))
            })
            .collect::<String>();
        Some(format!(
            "{name} · site {site}, {lineage} · population {} → {} · living matter {} → {} mg{}{}{traffic}",
            count(before),
            count(after),
            stock(before),
            stock(after),
            if resources.is_empty() { "" } else { " · " },
            resources
        ))
    }
}

fn living_lines(sites: &BTreeMap<u64, Region>) -> BTreeSet<&str> {
    sites.values().flat_map(Region::extant_lineages).collect()
}

struct Candidate {
    seed: u64,
    survived: usize,
    started: usize,
}

impl Candidate {
    fn at(seed: u64, start: &BTreeMap<u64, Region>, end: &BTreeMap<u64, Region>) -> Self {
        let start = living_lines(start);
        let end = living_lines(end);
        Self {
            seed,
            survived: start.intersection(&end).count(),
            started: start.len(),
        }
    }

    fn rank(&self, other: &Self) -> std::cmp::Ordering {
        (self.survived * other.started.max(1), self.survived)
            .cmp(&(other.survived * self.started.max(1), other.survived))
    }
}

struct Comparison {
    edited: Projection,
    original: Option<Projection>,
}
enum Reading {
    Forecast(Comparison),
    Search {
        settings: Founding,
        winner: Candidate,
        tried: usize,
        confirmed: usize,
    },
}
type Outcome = Result<Reading, String>;

#[derive(Default)]
pub(super) struct Outlook {
    pending: Option<Receiver<Outcome>>,
    result: Option<Outcome>,
    stale: bool,
    searching: bool,
}

impl Outlook {
    pub(super) fn invalidate(&mut self) {
        self.result = None;
        self.stale = self.pending.is_some();
    }
}

impl Panel {
    pub(super) fn forecast(&mut self) -> Result<(), String> {
        if self.forecasting() {
            return Ok(());
        }
        self.next_tick = None;
        let edited = self.session.clone();
        let original = self.original.clone();
        self.start_outlook(false, move || {
            Ok(Reading::Forecast(Comparison {
                edited: Projection::of(edited)?,
                original: original.map(Projection::of).transpose()?,
            }))
        })
    }

    pub(super) fn search(&mut self) -> Result<(), String> {
        if self.forecasting() {
            return Ok(());
        }
        let settings = self.recipe()?;
        let mut choices = vec![settings.clone()];
        for index in 1..=2 {
            let seed = cosmograph::draw(settings.seed, "founding-persistence", &[index]);
            choices.push(self.recipe_for_seed(seed)?);
        }
        self.start_outlook(true, move || {
            let mut shortlist = Vec::new();
            let mut last_error = None;
            for recipe in choices {
                let estimate = || -> Result<_, String> {
                    let world = Session::new(recipe.generate()?, Execution::Grouped)?;
                    let model = RegionalModel::observe(&world, SAMPLE)?;
                    let estimate =
                        Candidate::at(recipe.seed, model.start(), &model.project(HORIZON)?.sites);
                    if estimate.started == 0 {
                        return Err("This draw has no body or surface population to compare.".into());
                    }
                    Ok((estimate, world))
                };
                match estimate() {
                    Ok(item) => shortlist.push(item),
                    Err(reason) => last_error = Some(reason),
                }
            }
            let tried = shortlist.len();
            shortlist.sort_by(|(a, _), (b, _)| b.rank(a));
            let mut winner: Option<Candidate> = None;
            let mut confirmed = 0;
            for (candidate, mut world) in shortlist.into_iter().take(2) {
                let mut exact = || -> Result<_, String> {
                    let before = Region::read(&world.sim)?;
                    for _ in 0..HORIZON {
                        let _ = world.advance_tick_with_flows()?;
                    }
                    Ok(Candidate::at(
                        candidate.seed,
                        &before,
                        &Region::read(&world.sim)?,
                    ))
                };
                match exact() {
                    Ok(result) => {
                        confirmed += 1;
                        if winner.as_ref().is_none_or(|best| result.rank(best).is_gt()) {
                            winner = Some(result);
                        }
                    },
                    Err(reason) => last_error = Some(reason),
                }
            }
            Ok(Reading::Search {
                settings,
                winner: winner
                    .ok_or_else(|| last_error.unwrap_or_else(|| "No seeds to compare".into()))?,
                tried,
                confirmed,
            })
        })
    }

    pub(super) fn open_candidate(&mut self) -> Result<(), String> {
        let Some(Ok(Reading::Search {
            settings, winner, ..
        })) = &self.outlook.result
        else {
            return Err("Search for an ecology first.".into());
        };
        if &self.recipe()? != settings {
            return Err("World settings changed. Search with these settings first.".into());
        }
        let old_seed = std::mem::replace(&mut self.seed, super::field(winner.seed));
        if let Err(reason) = self.project() {
            self.seed = old_seed;
            return Err(reason);
        }
        Ok(())
    }

    pub(super) fn candidate(&self) -> Option<String> {
        let Ok(Reading::Search {
            winner,
            tried,
            confirmed,
            ..
        }) = self.outlook.result.as_ref()?
        else {
            return None;
        };
        Some(format!(
            "Compared {tried} seeds · ran {confirmed} finalists for {} ticks · seed {} retained {}/{} founding populations.",
            HORIZON, winner.seed, winner.survived, winner.started
        ))
    }

    fn start_outlook(
        &mut self,
        searching: bool,
        job: impl FnOnce() -> Outcome + Send + 'static,
    ) -> Result<(), String> {
        let (sender, receiver) = mpsc::channel();
        std::thread::Builder::new()
            .name("regional-ecology-outlook".into())
            .spawn(move || {
                let _ = sender.send(job());
            })
            .map_err(|e| format!("Could not start the forecast: {e}"))?;
        self.outlook = Outlook {
            pending: Some(receiver),
            result: None,
            stale: false,
            searching,
        };
        Ok(())
    }

    pub(in crate::app::bench) fn forecasting(&self) -> bool {
        self.outlook.pending.is_some()
    }

    pub(in crate::app::bench) fn poll_forecast(&mut self) {
        let Some(receiver) = &self.outlook.pending else {
            return;
        };
        let result = match receiver.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => Err("The forecast worker stopped.".into()),
        };
        self.outlook.pending = None;
        if !self.outlook.stale {
            self.outlook.result = Some(result);
        }
    }

    pub(super) fn outlook_words(&self) -> Vec<String> {
        let Some(result) = &self.outlook.result else {
            return if self.forecasting() && !self.outlook.stale {
                vec![if self.outlook.searching {
                    format!("Comparing seeds and running finalists for {HORIZON} ticks…")
                } else {
                    format!("Sampling {SAMPLE} ticks of unattended activity…")
                }]
            } else {
                Vec::new()
            };
        };
        let pair = match result {
            Ok(Reading::Forecast(pair)) => pair,
            Ok(Reading::Search { .. }) => return Vec::new(),
            Err(why) => return vec![format!("Forecast unavailable · {why}")],
        };
        let forecast = &pair.edited.future;
        let lineage = self.lineage.as_str();
        let mut lines = vec![format!(
            "Regional estimate · ticks {}–{} · based on {} sampled ticks",
            forecast.start_tick, forecast.end_tick, forecast.sample_ticks
        )];
        if let Some(original) = &pair.original {
            lines.extend(original.describe(self.site, lineage, "Original"));
        }
        lines.extend(pair.edited.describe(
            self.site,
            lineage,
            if pair.original.is_some() {
                "Edited"
            } else {
                "Outlook"
            },
        ));
        lines
    }
}
