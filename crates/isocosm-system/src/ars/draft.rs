//! Ars Magica creation as a saved draft (cases 1026, 1063): only the ordered
//! choices are stored; walking them against the pack's records rebuilds the
//! character and names the next decision, each option explained and each
//! refused one with its reason. The draft and decision types are the ones
//! Pathfinder's draft uses.

use super::records::{Records, Trait};
use super::rules::{ability_cap, ability_xp, art_xp, score, spend_for, with_affinity};
pub use crate::pf2e::draft::{Choice, Draft, Made, Opt, Progress, Slot};
use std::collections::BTreeMap;

/// A Virtue or Flaw taken, with what it is taken for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Taken {
    pub id: String,
    pub target: Option<String>,
    /// Points it counts: 0 free, 1 Minor, 3 Major.
    pub points: u32,
    pub flaw: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Build {
    /// `magus` or `grog`.
    pub kind: String,
    pub house: Option<String>,
    pub traits: Vec<Taken>,
    pub characteristics: BTreeMap<String, i64>,
    pub age: u32,
    /// Experience by Ability or Art id.
    pub xp: BTreeMap<String, u32>,
    pub spells: Vec<String>,
}

impl Build {
    pub fn has(&self, id: &str) -> bool {
        self.traits.iter().any(|t| t.id == id)
    }
    pub fn for_target(&self, id: &str, target: &str) -> bool {
        self.traits.iter().any(|t| t.id == id && t.target.as_deref() == Some(target))
    }
    fn points(&self, flaw: bool) -> u32 {
        self.traits.iter().filter(|t| t.flaw == flaw).map(|t| t.points).sum()
    }
    pub fn affinity(&self, id: &str) -> bool {
        self.for_target("affinity-with-ability", id) || self.for_target("affinity-with-art", id)
    }
    pub fn ability(&self, id: &str) -> u32 {
        score(self.xp.get(id).copied().unwrap_or(0), ability_xp)
    }
    pub fn art(&self, id: &str) -> u32 {
        score(self.xp.get(id).copied().unwrap_or(0), art_xp)
    }
    pub fn magus(&self) -> bool {
        self.kind == "magus"
    }
}

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

fn opt(value: &str, label: impl Into<String>, summary: impl Into<String>, refused: Option<String>) -> Opt {
    Opt { value: value.into(), label: label.into(), summary: summary.into(), refused }
}

const DONE: &str = "done";

impl Walk<'_> {
    fn take(&mut self, slot: Slot) -> Result<String, Stop> {
        let Some(choice) = self.choices.get(self.at) else { return Err(Stop::Pending(slot)) };
        if choice.slot != slot.id {
            return Err(Stop::Bad(format!("saved choice {} does not answer {}", choice.slot, slot.title)));
        }
        let o = slot.options.iter().find(|o| o.value == choice.value)
            .ok_or_else(|| Stop::Bad(format!("{} is not an option for {}", choice.value, slot.title)))?;
        if let Some(why) = &o.refused {
            return Err(Stop::Bad(format!("{}: {why}", o.label)));
        }
        self.at += 1;
        self.made.push(Made { slot: slot.id.clone(), kind: slot.kind, title: slot.title.clone(),
            value: o.value.clone(), label: o.label.clone(), summary: o.summary.clone() });
        Ok(choice.value.clone())
    }

    fn slot(id: &str, kind: &'static str, title: impl Into<String>, options: Vec<Opt>) -> Slot {
        Slot { id: id.into(), kind, title: title.into(), options }
    }

    fn run(&mut self) -> Result<(), Stop> {
        let r = self.r;
        let kind = self.take(Self::slot("type", "type", "Character type", vec![
            opt("magus", "Magus", "A Hermetic wizard of the Order.", None),
            opt("grog", "Grog", "A minor character of the covenant.", None),
        ]))?;
        self.build.kind = kind.clone();
        for t in r.virtues.iter().filter(|t| t.automatic && t.types.contains(&kind)) {
            self.build.traits.push(Taken { id: t.id.clone(), target: None, points: 0, flaw: false });
        }
        if self.build.magus() {
            let houses = r.houses.iter().map(|h| opt(&h.id, &h.name, &h.summary, None)).collect();
            let house = self.take(Self::slot("house", "house", "House", houses))?;
            let h = r.house(&house).expect("offered");
            let benefit = r.trait_(&h.benefit).expect("checked");
            let targets = h.targets.iter().map(|t| opt(t, format!("{} ({})", benefit.name, r.name(t)), &benefit.summary, None)).collect();
            let target = self.take(Self::slot("house-benefit", "house-benefit", format!("House {} benefit", h.name), targets))?;
            self.build.house = Some(house);
            self.build.traits.push(Taken { id: benefit.id.clone(), target: Some(target), points: 0, flaw: false });
        }
        self.virtues()?;
        self.characteristics()?;
        let ages = if self.build.magus() { &r.rules.magus_ages } else { &r.rules.grog_ages };
        let ages = ages.iter().map(|a| opt(&a.to_string(), format!("{a} years old"), "", None)).collect();
        self.build.age = self.take(Self::slot("age", "age", "Starting age", ages))?.parse().map_err(|_| Stop::Bad("age".into()))?;
        self.build.xp.insert("living-language".into(), r.rules.native_language_xp);
        let childhood: Vec<String> = r.abilities.iter().filter(|a| a.childhood).map(|a| a.id.clone()).collect();
        self.spend("childhood", "Early childhood", r.rules.childhood_xp, &childhood, false)?;
        let mut years = self.build.age.saturating_sub(r.rules.childhood_years);
        if self.build.magus() {
            years = years.saturating_sub(r.rules.apprenticeship_years);
        }
        let learnable = self.learnable(false);
        self.spend("later-life", "Later life", years * r.rules.later_life_xp_per_year, &learnable, false)?;
        let pools: Vec<&Trait> = r.virtues.iter().filter(|t| t.pool.is_some() && self.build.has(&t.id)).collect();
        for t in pools {
            let pool = t.pool.as_ref().unwrap();
            let allowed: Vec<String> = r.abilities.iter()
                .filter(|a| pool.kinds.contains(&a.kind) || pool.abilities.contains(&a.id))
                .filter(|a| !a.gift || self.build.has("the-gift"))
                .map(|a| a.id.clone()).collect();
            self.spend(&format!("bonus-{}", t.id), "bonus", pool.xp, &allowed, false)?;
        }
        if self.build.magus() {
            let mut allowed = self.learnable(true);
            allowed.extend(r.arts.iter().map(|a| a.id.clone()));
            self.spend("apprenticeship", "Apprenticeship", r.rules.apprenticeship_xp, &allowed, true)?;
            self.spells()?;
        }
        Ok(())
    }

    /// Abilities this character may buy now; apprenticeship opens every type.
    fn learnable(&self, apprenticed: bool) -> Vec<String> {
        let opened: Vec<&str> = self.build.traits.iter()
            .filter_map(|t| self.r.trait_(&t.id).and_then(|t| t.opens.as_deref())).collect();
        self.r.abilities.iter()
            .filter(|a| a.kind == "general" || apprenticed || opened.contains(&a.kind.as_str()))
            .filter(|a| !a.gift || self.build.has("the-gift"))
            .map(|a| a.id.clone()).collect()
    }

    fn virtues(&mut self) -> Result<(), Stop> {
        let r = self.r;
        loop {
            let grog = !self.build.magus();
            let (flaws, virtues) = (self.build.points(true), self.build.points(false));
            let count = |flaw: bool| self.build.traits.iter().filter(|t| t.flaw == flaw && t.points > 0).count() as u32;
            let mut options = Vec::new();
            for t in r.virtues.iter().chain(&r.flaws) {
                if t.automatic || !t.types.contains(&self.build.kind) {
                    continue;
                }
                let flaw = r.flaws.iter().any(|f| f.id == t.id);
                let taken = self.build.traits.iter().filter(|x| x.id == t.id).count();
                let min_points = if t.weight == "major" { 3 } else { 1 };
                let refused = t.refused.clone()
                    .or_else(|| (taken > 0 && !t.repeatable).then(|| "already taken".into()))
                    .or_else(|| t.max.is_some_and(|m| taken >= m).then(|| "taken as often as allowed".into()))
                    .or_else(|| (grog && t.weight == "major").then(|| "grogs take only Minor Virtues and Flaws".into()))
                    .or_else(|| (grog && t.kind == "story").then(|| "grogs take no Story Flaws".into()))
                    .or_else(|| (grog && count(flaw) >= r.rules.grog_minor_max).then(|| format!("grogs take at most {}", r.rules.grog_minor_max)))
                    .or_else(|| (!grog && flaw && flaws + min_points > r.rules.virtue_points_max)
                        .then(|| format!("Flaws may not pass {} points", r.rules.virtue_points_max)))
                    .or_else(|| (!grog && !flaw && virtues + min_points > r.rules.virtue_points_max)
                        .then(|| format!("Virtues may not pass {} points", r.rules.virtue_points_max)));
                let weight = match t.weight.as_str() { "major" => "Major", "minor" => "Minor", _ => "Minor or Major" };
                let label = format!("{} ({weight} {}{})", t.name, t.kind.replace('-', " "), if flaw { " Flaw" } else { "" });
                options.push(opt(&t.id, label, &t.summary, refused));
            }
            let over = if grog { count(false) > count(true) } else { virtues > flaws };
            options.push(opt(DONE, "Done with Virtues and Flaws",
                format!("Flaws {flaws} points, Virtues {virtues} points."),
                over.then(|| "Virtues may not outweigh Flaws".into())));
            let id = self.take(Self::slot("virtue", "virtue", "Virtues and Flaws", options))?;
            if id == DONE {
                return Ok(());
            }
            let t = r.trait_(&id).expect("offered");
            let flaw = r.flaws.iter().any(|f| f.id == id);
            let points = match t.weight.as_str() {
                "major" => 3,
                "either" if !grog => {
                    let w = self.take(Self::slot("weight", "weight", format!("{} weight", t.name), vec![
                        opt("minor", "Minor (1 point)", "", None),
                        opt("major", "Major (3 points)", "",
                            (flaws + 3 > r.rules.virtue_points_max).then(|| format!("Flaws may not pass {} points", r.rules.virtue_points_max))),
                    ]))?;
                    if w == "major" { 3 } else { 1 }
                },
                _ => 1,
            };
            let target = match t.target.as_deref() {
                Some(kind) => {
                    let pool: Vec<(String, String)> = match kind {
                        "ability" => r.abilities.iter().map(|a| (a.id.clone(), a.name.clone())).collect(),
                        "art" => r.arts.iter().map(|a| (a.id.clone(), a.name.clone())).collect(),
                        _ => r.arts.iter().filter(|a| a.kind == "form").map(|a| (a.id.clone(), a.name.clone())).collect(),
                    };
                    let options = pool.iter().map(|(v, n)| opt(v, n.clone(), "",
                        self.build.for_target(&id, v).then(|| "already taken for this".into()))).collect();
                    Some(self.take(Self::slot("target", "target", format!("{} for", t.name), options))?)
                },
                None => None,
            };
            self.build.traits.push(Taken { id, target, points, flaw });
        }
    }

    fn characteristics(&mut self) -> Result<(), Stop> {
        let r = self.r;
        for c in &r.characteristics {
            let spent: i64 = self.build.characteristics.values()
                .map(|v| r.rules.characteristic_costs[&v.to_string()]).sum();
            let left = r.rules.characteristic_points - spent;
            let mut options = Vec::new();
            for value in -3..=3i64 {
                let cost = r.rules.characteristic_costs[&value.to_string()];
                let refused = (cost > left).then(|| format!("costs {cost}; {left} points left"));
                let label = format!("{value:+} ({})", if cost >= 0 { format!("costs {cost}") } else { format!("gives {}", -cost) });
                options.push(opt(&value.to_string(), label, "", refused));
            }
            let title = format!("{} ({left} points left)", c.name);
            let v: i64 = self.take(Self::slot(&format!("char-{}", c.id), "characteristic", title, options))?
                .parse().map_err(|_| Stop::Bad("characteristic".into()))?;
            self.build.characteristics.insert(c.id.clone(), v);
        }
        Ok(())
    }

    /// Spend a pool one score at a time on `allowed`, until done or spent.
    fn spend(&mut self, id: &str, title: &str, pool: u32, allowed: &[String], arts_open: bool) -> Result<(), Stop> {
        let r = self.r;
        let kind: &'static str = match id { "childhood" => "childhood", "later-life" => "later-life",
            "apprenticeship" => "apprenticeship", _ => "bonus" };
        let mut left = pool;
        loop {
            let mut options = Vec::new();
            for target in allowed {
                let art = r.art(target).is_some();
                let (have, cost) = if art { (self.build.art(target), art_xp as fn(u32) -> u32) }
                    else { (self.build.ability(target), ability_xp as fn(u32) -> u32) };
                let affinity = self.build.affinity(target);
                let current = self.build.xp.get(target).copied().unwrap_or(0);
                let spend = spend_for(cost(have + 1).saturating_sub(current), affinity);
                let cap = (!art).then(|| ability_cap(r, self.build.age, affinity));
                let refused = cap.filter(|c| have + 1 > *c).map(|c| format!("score capped at {c} at this age"))
                    .or_else(|| (spend > left).then(|| format!("needs {spend} experience")));
                options.push(opt(target, format!("{} {} ({spend} xp)", r.name(target), have + 1), "", refused));
            }
            if options.iter().all(|o| o.refused.is_some()) || left == 0 {
                return Ok(());
            }
            let short: Vec<String> = if arts_open {
                r.rules.magus_minimums.iter().filter(|(a, min)| self.build.ability(a) < **min)
                    .map(|(a, min)| format!("{} {min}", r.name(a))).collect()
            } else { Vec::new() };
            options.push(opt(DONE, "Done", format!("{left} experience unspent is lost."),
                (!short.is_empty()).then(|| format!("a magus needs at least {}", short.join(", ")))));
            let choice = self.take(Self::slot(id, kind, format!("{title} ({left} xp left)"), options))?;
            if choice == DONE {
                return Ok(());
            }
            let current = self.build.xp.get(&choice).copied().unwrap_or(0);
            let art = r.art(&choice).is_some();
            let have = if art { self.build.art(&choice) } else { self.build.ability(&choice) };
            let need = if art { art_xp(have + 1) } else { ability_xp(have + 1) }.saturating_sub(current);
            let affinity = self.build.affinity(&choice);
            let spent = spend_for(need, affinity);
            left -= spent;
            *self.build.xp.entry(choice).or_default() += with_affinity(spent, affinity);
        }
    }

    fn spells(&mut self) -> Result<(), Stop> {
        let r = self.r;
        let mut left = r.rules.apprenticeship_spell_levels;
        loop {
            let int = self.build.characteristics.get("int").copied().unwrap_or(0);
            let theory = i64::from(self.build.ability("magic-theory"));
            let mut options = Vec::new();
            for s in &r.spells {
                let form = std::iter::once(&s.form).chain(&s.requisites)
                    .map(|a| self.build.art(a)).min().unwrap_or(0);
                let limit = i64::from(self.build.art(&s.technique)) + i64::from(form) + int + theory + 3;
                let refused = self.build.spells.contains(&s.id).then(|| "already known".to_owned())
                    .or_else(|| (i64::from(s.level) > limit).then(|| format!("level {} passes this magus's limit of {limit}", s.level)))
                    .or_else(|| (s.level > left).then(|| format!("{left} levels left")));
                let label = format!("{} ({} {} {})", s.name, r.name(&s.technique), r.name(&s.form), s.level);
                let summary = format!("{} R: {}, D: {}, T: {}.", s.summary, s.range, s.duration, s.target);
                options.push(opt(&s.id, label, summary, refused));
            }
            if options.iter().all(|o| o.refused.is_some()) {
                return Ok(());
            }
            options.push(opt(DONE, "Done with spells", format!("{left} levels unspent are lost."), None));
            let choice = self.take(Self::slot("spell", "spell", format!("Spells ({left} levels left)"), options))?;
            if choice == DONE {
                return Ok(());
            }
            left -= r.spell(&choice).expect("offered").level;
            self.build.spells.push(choice);
        }
    }
}

pub fn walk(r: &Records, draft: &Draft) -> Result<Walked, String> {
    let mut w = Walk { r, choices: &draft.choices, at: 0, build: Build::default(), made: Vec::new() };
    let progress = match w.run() {
        Ok(()) if w.at == draft.choices.len() => Progress::Complete,
        Ok(()) => return Err("the draft holds choices past a finished character".into()),
        Err(Stop::Pending(slot)) => Progress::Pending(slot),
        Err(Stop::Bad(why)) => return Err(why),
    };
    Ok(Walked { build: w.build, made: w.made, progress })
}

/// Answer the pending decision; refused options are refused here too.
pub fn choose(r: &Records, draft: &mut Draft, slot: &str, value: &str) -> Result<Made, String> {
    let Progress::Pending(pending) = walk(r, draft)?.progress else {
        return Err("every choice is already made".into());
    };
    if pending.id != slot {
        return Err(format!("the next decision is {}", pending.title));
    }
    let o = pending.options.iter().find(|o| o.value == value)
        .ok_or_else(|| format!("{value} is not an option for {}", pending.title))?;
    if let Some(why) = &o.refused {
        return Err(format!("{}: {why}", o.label));
    }
    draft.choices.push(Choice { slot: slot.into(), value: value.into() });
    let walked = walk(r, draft)?;
    walked.made.last().cloned().ok_or_else(|| "nothing was recorded".into())
}

pub fn finished(r: &Records, draft: &Draft) -> Result<Build, String> {
    match walk(r, draft)?.progress {
        Progress::Complete => Ok(walk(r, draft)?.build),
        Progress::Pending(slot) => Err(format!("still to choose: {}", slot.title)),
    }
}
