// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A founding distribution. Kingdoms are drawn without replacement. Each
//! kingdom independently draws its richness; no lineage has a founding parent.
use crate::{Founding, Result, draw, ecology::Role};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// The founding mechanisms shipped by the sim. Kingdom identities in worlds
/// and in the eligible pool remain namespaced keys.
pub enum FoundingForm {
    Plant,
    Animal,
    Fungus,
    Microbe,
    WorldClass,
}

impl FoundingForm {
    pub const ALL: [Self; 5] = [
        Self::Plant,
        Self::Animal,
        Self::Fungus,
        Self::Microbe,
        Self::WorldClass,
    ];
    pub fn key(self) -> &'static str {
        match self {
            Self::Plant => "kingdom:flora",
            Self::Animal => "kingdom:fauna",
            Self::Fungus => "kingdom:myco",
            Self::Microbe => "kingdom:micro",
            Self::WorldClass => "kingdom:world",
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Plant => "Plant",
            Self::Animal => "Animal",
            Self::Fungus => "Fungus",
            Self::Microbe => "Microbe",
            Self::WorldClass => "World-class",
        }
    }
    pub(crate) fn bodied(self) -> bool {
        matches!(self, Self::Plant | Self::Animal | Self::Fungus)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KingdomDraw {
    pub pool: BTreeSet<String>,
    pub kingdoms: [u32; 2],
    pub per_kingdom: [u32; 2],
}
impl Default for KingdomDraw {
    fn default() -> Self {
        Self {
            pool: FoundingForm::ALL
                .into_iter()
                .map(|f| f.key().to_owned())
                .collect(),
            kingdoms: [2, 5],
            per_kingdom: [3, 12],
        }
    }
}
impl KingdomDraw {
    pub fn draw(&self, seed: u64) -> Result<Vec<(FoundingForm, u32)>> {
        let [lo, hi] = self.kingdoms;
        let [few, many] = self.per_kingdom;
        if lo == 0
            || lo > hi
            || hi as usize > self.pool.len()
            || few == 0
            || few > many
            || u64::from(hi) * u64::from(many) > 256
        {
            return Err(
                "choose a positive kingdom range within the pool and up to 256 total lineages"
                    .into(),
            );
        }
        let mut pool = Vec::new();
        for key in &self.pool {
            let form = FoundingForm::ALL
                .into_iter()
                .find(|f| f.key() == key)
                .ok_or_else(|| format!("{key} has no registered founding mechanism"))?;
            pool.push(form);
        }
        pool.sort_by_key(|k| (draw(seed, "founding-kingdom", &[*k as u64]), *k));
        let count =
            lo + (draw(seed, "founding-kingdom-count", &[]) % u64::from(hi - lo + 1)) as u32;
        pool.truncate(count as usize);
        pool.sort();
        Ok(pool
            .into_iter()
            .map(|k| {
                (
                    k,
                    few + (draw(seed, "founding-lineage-richness", &[k as u64])
                        % u64::from(many - few + 1)) as u32,
                )
            })
            .collect())
    }
}

/// The ways of making a living a kingdom may weigh, with the role each
/// founds: the paces the sim's ecology has.
pub const LIVINGS: [&str; 5] = ["fix", "graze", "hunt", "forage", "rot"];
const ROLES: [Role; 5] = [Role::Producer, Role::Grazer, Role::Predator, Role::Omnivore, Role::Decomposer];

/// Of a world's bodied founders, the share that must fix, as a fraction:
/// tissue enters a world only through fixing, so one drawn with too few
/// fixers has nothing under what eats.
const FIXERS: [usize; 2] = [2, 5];

/// The living each of `forms` founds, by lineage index. Each is drawn by its
/// kingdom's weights. Where fewer than [`FIXERS`] of them fix, founders are
/// turned to fixing in a drawn order, those whose kingdom weighs fixing at
/// all first, until enough do: a world of fungi alone gets its lichens.
pub(crate) fn livings(seed: u64, forms: &[FoundingForm]) -> Vec<Role> {
    let catalog = crate::catalog::catalog();
    let weight = |form: FoundingForm, living: &str| {
        catalog.kingdom(form.key()).and_then(|k| k.living.get(living)).copied().map_or(0, u64::from)
    };
    let mut roles: Vec<Role> = forms
        .iter()
        .enumerate()
        .map(|(i, form)| {
            let total: u64 = LIVINGS.iter().map(|l| weight(*form, l)).sum();
            let mut at = draw(seed, "founder-living", &[i as u64]) % total.max(1);
            let found = LIVINGS.iter().position(|l| {
                let w = weight(*form, l);
                if at < w {
                    return true;
                }
                at -= w;
                false
            });
            found.map_or(Role::Decomposer, |at| ROLES[at])
        })
        .collect();
    let needed = (forms.len() * FIXERS[0]).div_ceil(FIXERS[1]).max(usize::from(!forms.is_empty()));
    let mut turn: Vec<usize> = (0..forms.len()).filter(|i| roles[*i] != Role::Producer).collect();
    turn.sort_by_key(|i| (weight(forms[*i], "fix") == 0, draw(seed, "founder-fixes", &[*i as u64]), *i));
    let fixing = roles.iter().filter(|r| **r == Role::Producer).count();
    for i in turn.into_iter().take(needed.saturating_sub(fixing)) {
        roles[i] = Role::Producer;
    }
    // A mouth for living plant tissue has nothing to eat in a world drawn
    // without plants: such a founder hunts what is there.
    if !forms.contains(&FoundingForm::Plant) {
        for role in roles.iter_mut().filter(|r| **r == Role::Grazer) {
            *role = Role::Predator;
        }
    }
    roles
}

impl Founding {
    /// Project an editor draft into a different draw without assigning an
    /// authored plant map to a fungus that now has its old numeric key.
    pub fn carry_body_edits(&mut self, previous: &Founding) -> Result<()> {
        let slots = |f: &Founding| -> Result<Vec<String>> {
            let Some(draw) = f.repertoire.as_ref().and_then(|r| r.kingdom_draw.as_ref()) else {
                return Ok(Vec::new());
            };
            Ok(draw
                .draw(f.seed)?
                .into_iter()
                .filter(|(form, _)| form.bodied())
                .flat_map(|(form, n)| (0..n).map(move |slot| format!("{}/{slot}", form.key())))
                .collect())
        };
        if self.body_maps.is_empty()
            && self
                .repertoire
                .as_ref()
                .is_none_or(|r| r.lineage_edits.is_empty())
        {
            return Ok(());
        }
        let old = slots(previous)?;
        let new = slots(self)?;
        let Some(rep) = self
            .repertoire
            .as_mut()
            .filter(|r| r.kingdom_draw.is_some())
        else {
            return Ok(());
        };
        for (i, slot) in old.iter().enumerate() {
            if let Some(body) = self.body_maps.get(&format!("lineage:{i}")) {
                rep.lineage_edits.insert(slot.clone(), body.clone());
            }
        }
        self.body_maps = new
            .into_iter()
            .enumerate()
            .filter_map(|(i, slot)| {
                Some((
                    format!("lineage:{i}"),
                    rep.lineage_edits.get(&slot)?.clone(),
                ))
            })
            .collect();
        Ok(())
    }

    pub(crate) fn drawn_founding(&self) -> Result<Option<(Self, Vec<(FoundingForm, u32)>)>> {
        let Some(config) = self
            .repertoire
            .as_ref()
            .and_then(|r| r.kingdom_draw.as_ref())
        else {
            return Ok(None);
        };
        let roster = config.draw(self.seed)?;
        let mut f = self.clone();
        let r = f.repertoire.as_mut().unwrap();
        if !r.founders {
            return Err("kingdom draws require generated lineage recipes".into());
        }
        r.bodied_kingdoms = roster
            .iter()
            .filter(|(k, _)| k.bodied())
            .flat_map(|&(k, n)| std::iter::repeat_n(k, n as usize))
            .collect();
        f.lineages = r.bodied_kingdoms.len() as u32;
        if let Some(p) = &mut f.played {
            if f.lineages == 0 {
                f.played = None
            } else {
                p.lineage = p.lineage.min(f.lineages - 1);
            }
        }
        if f.lineages != 0 && (self.population < u64::from(f.lineages) || self.cohort_size == 0) {
            return Err(format!(
                "starting population must cover all {} body lineages",
                f.lineages
            ));
        }
        Ok(Some((f, roster)))
    }
    /// The form each bodied lineage was drawn as, by lineage index; empty
    /// where the founding draws no kingdoms.
    pub(crate) fn forms(&self) -> Vec<FoundingForm> {
        let Some(r) = &self.repertoire else { return Vec::new() };
        if !r.bodied_kingdoms.is_empty() {
            return r.bodied_kingdoms.clone();
        }
        let roster = r.kingdom_draw.as_ref().and_then(|d| d.draw(self.seed).ok()).unwrap_or_default();
        let bodied = roster.into_iter().filter(|(form, _)| form.bodied());
        bodied.flat_map(|(form, n)| std::iter::repeat_n(form, n as usize)).collect()
    }

    /// The kingdom lineage `i` is of: the one it was drawn as, which is its
    /// tissue and its weights and says nothing of how it lives (case 1219),
    /// or, where the founding draws none, the one its role is generated in.
    pub(crate) fn kingdom_key(&self, i: u32) -> &'static str {
        self.forms().get(i as usize).map_or_else(|| self.role(i).kingdom(), |form| form.key())
    }

    /// How lineage `i`'s founder makes its living. Where kingdoms are drawn
    /// it is drawn too, by the weights its kingdom gives each way
    /// (`catalog::Kingdom::living`), so any kingdom may found any living;
    /// then held to what a world needs to run at all ([`livings`]).
    pub(crate) fn role(&self, i: u32) -> Role {
        let forms = self.forms();
        match forms.get(i as usize) {
            Some(_) => livings(self.seed, &forms)[i as usize],
            None => Role::of(self.web, i),
        }
    }
}
