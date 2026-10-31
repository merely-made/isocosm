//! A saved creation draft (case 1026): the ordered choices a player has made.
//! Nothing else is stored. Walking the choices against the records rebuilds
//! the character and names the next decision, with every option explained
//! and every illegal option carrying its reason, so a draft reopens exactly
//! and a tampered or stale choice is refused rather than repaired.

use super::records::{self, Records};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod grants;
mod steps;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub slot: String,
    pub value: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub choices: Vec<Choice>,
}

/// One option of a decision. A refused option is shown with its reason and
/// cannot be chosen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Opt {
    pub value: String,
    pub label: String,
    pub summary: String,
    pub refused: Option<String>,
}

/// One decision: a stable id (what the draft stores), the kind of step (its
/// explanation lives in the records), a title and the options.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    pub id: String,
    pub kind: &'static str,
    pub title: String,
    pub options: Vec<Opt>,
}

/// A decision already made, kept with the option's own explanation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Made {
    pub slot: String,
    pub kind: &'static str,
    pub title: String,
    pub value: String,
    pub label: String,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Progress {
    Pending(Slot),
    Complete,
}

/// What the choices so far add up to.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Build {
    pub ancestry: Option<String>,
    pub heritage: Option<String>,
    pub background: Option<String>,
    pub class: Option<String>,
    pub racket: Option<String>,
    pub key: Option<String>,
    /// (source, attribute) for every boost taken.
    pub boosts: Vec<(String, String)>,
    pub flaws: Vec<String>,
    /// Proficiency ranks other than skills: perception, saves, class DC,
    /// attack and defense categories.
    pub ranks: BTreeMap<String, u8>,
    pub skills: BTreeMap<String, u8>,
    pub lores: Vec<String>,
    /// Feats and class features, in the order gained.
    pub feats: Vec<String>,
    /// Options recorded by feats, by the feat's name (a craft specialty,
    /// an Assurance skill, an adopted ancestry).
    pub options: Vec<(String, String)>,
    pub speed_bonus: i64,
    pub hp_per_level: i64,
    pub dying_max: Option<i64>,
    pub recovery_dc: i64,
    pub initiative: i64,
    pub resistances: Vec<String>,
}

impl Build {
    pub fn modifier(&self, attribute: &str) -> i64 {
        let up = self.boosts.iter().filter(|(_, a)| a == attribute).count();
        let down = self.flaws.iter().filter(|a| *a == attribute).count();
        up as i64 - down as i64
    }
    pub fn trained(&self, skill: &str) -> bool {
        self.skills.get(skill).is_some_and(|r| *r > 0)
    }
}

/// The walk's result: the build so far, the decisions made, and what is next.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Walked {
    pub build: Build,
    pub made: Vec<Made>,
    pub progress: Progress,
}

enum Stop {
    Pending(Slot),
    Bad(String),
}

struct Walk<'a> {
    r: &'a Records,
    choices: &'a [Choice],
    at: usize,
    build: Build,
    made: Vec<Made>,
}

impl<'a> Walk<'a> {
    /// The stored choice for `slot`, checked; or the slot itself, pending.
    fn take(&mut self, slot: Slot) -> Result<String, Stop> {
        let Some(choice) = self.choices.get(self.at) else {
            return Err(Stop::Pending(slot));
        };
        if choice.slot != slot.id {
            return Err(Stop::Bad(format!(
                "saved choice {} does not answer the next decision, {}",
                choice.slot, slot.title
            )));
        }
        let opt = slot
            .options
            .iter()
            .find(|o| o.value == choice.value)
            .ok_or_else(|| {
                Stop::Bad(format!("{} is not an option for {}", choice.value, slot.title))
            })?;
        if let Some(why) = &opt.refused {
            return Err(Stop::Bad(format!("{}: {why}", opt.label)));
        }
        self.at += 1;
        self.made.push(Made {
            slot: slot.id.clone(),
            kind: slot.kind,
            title: slot.title.clone(),
            value: opt.value.clone(),
            label: opt.label.clone(),
            summary: opt.summary.clone(),
        });
        Ok(choice.value.clone())
    }
}

/// Six attribute options for a boost from `source`; one already boosted
/// from the same source is refused.
fn attribute_options(build: &Build, source: &str, allowed: &[String]) -> Vec<Opt> {
    records::ATTRIBUTES
        .iter()
        .filter(|a| allowed.is_empty() || allowed.iter().any(|x| x == *a))
        .map(|a| {
            let now = build.modifier(a);
            let refused = build
                .boosts
                .iter()
                .any(|(s, x)| s == source && x == a)
                .then(|| "this source already boosted it; boosts from one source go to different attributes".to_owned());
            Opt {
                value: (*a).to_owned(),
                label: records::attribute_name(a).to_owned(),
                summary: format!("{} is {now:+} now; this boost makes it {:+}.", records::attribute_name(a), now + 1),
                refused,
            }
        })
        .collect()
}

/// Skill options; trained ones are refused.
fn skill_options(build: &Build, from: &[String]) -> Vec<Opt> {
    records::SKILLS
        .iter()
        .filter(|(s, _)| from.is_empty() || from.iter().any(|x| x == s))
        .map(|(s, a)| Opt {
            value: (*s).to_owned(),
            label: records::title(s),
            summary: format!(
                "Keys off {} ({:+}); trained adds your level plus 2.",
                records::attribute_name(a),
                build.modifier(a)
            ),
            refused: build.trained(s).then(|| "already trained".to_owned()),
        })
        .collect()
}

/// Walk `draft` against `records`. An error means a saved choice no longer
/// fits: the draft is refused, never silently mended.
pub fn walk(records: &Records, draft: &Draft) -> Result<Walked, String> {
    let mut w = Walk {
        r: records,
        choices: &draft.choices,
        at: 0,
        build: Build::default(),
        made: Vec::new(),
    };
    let progress = match steps::run(&mut w) {
        Ok(()) if w.at == draft.choices.len() => Progress::Complete,
        Ok(()) => return Err("the draft holds choices past a finished character".into()),
        Err(Stop::Pending(slot)) => Progress::Pending(slot),
        Err(Stop::Bad(why)) => return Err(why),
    };
    Ok(Walked {
        build: w.build,
        made: w.made,
        progress,
    })
}

impl Draft {
    /// Answer the pending decision. Refused with the option's reason when it
    /// is illegal, and with the pending decision when `slot` is another one.
    pub fn choose(&mut self, records: &Records, slot: &str, value: &str) -> Result<Made, String> {
        let walked = walk(records, self)?;
        let Progress::Pending(pending) = walked.progress else {
            return Err("every choice is already made".into());
        };
        if pending.id != slot {
            return Err(format!("the next decision is {}", pending.title));
        }
        let opt = pending
            .options
            .iter()
            .find(|o| o.value == value)
            .ok_or_else(|| format!("{value} is not an option for {}", pending.title))?;
        if let Some(why) = &opt.refused {
            return Err(format!("{} refused: {why}", opt.label));
        }
        self.choices.push(Choice {
            slot: slot.to_owned(),
            value: value.to_owned(),
        });
        let walked = walk(records, self)?;
        walked
            .made
            .last()
            .cloned()
            .ok_or_else(|| "choice was not recorded".into())
    }

    pub fn undo(&mut self) -> Result<Choice, String> {
        self.choices.pop().ok_or_else(|| "nothing to undo".into())
    }

    /// The finished build, or the decision still pending.
    pub fn finished(&self, records: &Records) -> Result<Build, String> {
        let walked = walk(records, self)?;
        match walked.progress {
            Progress::Complete => Ok(walked.build),
            Progress::Pending(slot) => Err(format!("still to choose: {}", slot.title)),
        }
    }
}
