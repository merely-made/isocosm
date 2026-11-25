// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A founding distribution. Kingdoms are drawn without replacement. Each
//! kingdom independently draws its richness; no lineage has a founding parent.
use crate::{
    Founding, Result, draw,
    ecology::{Role, Web},
};
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
    pub(crate) fn role(&self, i: u32) -> Role {
        let k = self.repertoire.as_ref().and_then(|r| {
            if let Some(k) = r.bodied_kingdoms.get(i as usize) {
                return Some(*k);
            }
            let mut offset = i;
            for (k, count) in r.kingdom_draw.as_ref()?.draw(self.seed).ok()? {
                if k.bodied() {
                    if offset < count {
                        return Some(k);
                    }
                    offset -= count;
                }
            }
            None
        });
        match k {
            Some(FoundingForm::Plant) => Role::Producer,
            Some(FoundingForm::Fungus) => Role::Decomposer,
            Some(FoundingForm::Animal) if self.web == Web::Diets => {
                [Role::Grazer, Role::Predator, Role::Omnivore]
                    [(draw(self.seed, "animal-diet", &[i.into()]) % 3) as usize]
            },
            Some(FoundingForm::Animal) => Role::Grazer,
            _ => Role::of(self.web, i),
        }
    }
}
