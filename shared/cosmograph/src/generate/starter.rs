// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The ordinary world a seed founds: every optional ecology the bodied
//! generator has, with founders drawn from the catalogue. Mesocosm's New
//! World opens on it, and anything that wants to read what seeds make can
//! found the same recipe without the editor.

use super::Founding;
use crate::{directing::Played, ecology, map};

/// The consumer lineages that found one community, and the sites it starts across.
const COMMUNITY: u64 = 4;
const HOME: u64 = 2;
/// Founding members for each bodied lineage of the starting recipe, and for
/// each site of its map: enough of each line to breed, enough in all to
/// cover the ground.
const MEMBERS: u64 = 12;
const COVER: u64 = 6;

impl Founding {
    /// Where founding group `group` of lineage `lineage` starts. In a
    /// composed world what stays where it grows, the fixers and the eaters
    /// of the dead, is scattered over the whole map, so the ground is
    /// covered and whatever wanders finds something there. What eats the
    /// living founds communities: its lineages are dealt among a few homes,
    /// each a short run of sites, dealt like cards from a deck sorted by
    /// kingdom and living so each home holds its share of every kind, and
    /// different parts of the map begin with different animals. Other
    /// worlds scatter all their founders.
    pub(super) fn founder_site(&self, lineage: u64, group: u64) -> u64 {
        let sites = u64::from(self.sites);
        let draw = |domain: &str, of: u64| crate::draw(self.seed, domain, &[of]);
        if !self.composed() || !self.role(lineage as u32).consumer() {
            return draw("habitat", group) % sites;
        }
        let mut deck: Vec<u32> = (0..self.lineages).filter(|i| self.role(*i).consumer()).collect();
        deck.sort_by_key(|i| (self.kingdom_key(*i), self.role(*i) as u8, draw("community", u64::from(*i)), *i));
        let communities = (deck.len() as u64 / COMMUNITY).clamp(1, sites);
        let dealt = deck.iter().position(|i| u64::from(*i) == lineage).unwrap_or(0) as u64 % communities;
        let home = draw("community-site", dealt) % sites;
        (home + draw("habitat", group) % HOME.min(sites)) % sites
    }
}

impl Founding {
    /// The starting recipe for `seed`.
    ///
    /// # Panics
    ///
    /// Never: the default kingdom draw is valid for every seed.
    #[must_use]
    pub fn starter(seed: u64) -> Self {
        let mut draft = Self {
            seed,
            web: ecology::Web::Diets,
            population: 90,
            cohort_size: 2,
            // The A1 rates are declared at a founding reserve of eight: a
            // founder with less starves before its first meal or its first
            // fixing.
            stock_min: 8,
            stock_max: 16,
            ecology: true,
            ecology_rates: ecology::Rates::A1,
            // A leaf fixes by the tissue behind it, and a founder's ten units are
            // spread over its whole body: full rate at four backed cells is
            // what a founder's leaf can hold.
            bodies: Some(crate::bodied::Bodies { canopy_growth: true, fixing_budget: 4, ..Default::default() }),
            inherit: true,
            // A defence pays rent by its cells, and the catalogue's organs
            // hold nine to a hundred: at the rule's own eight cells a unit, a
            // founder's one gland cost four times its upkeep and it starved
            // however it fed. Thirty-two keeps a defence dear and payable.
            guard: Some(ecology::Guard { rent_cells: 32, ..Default::default() }),
            categories: Some(Default::default()),
            nutrients: Some(Default::default()),
            mutualism: Some(Default::default()),
            detritus: Some(Default::default()),
            // What keeps a founded web from eating itself out: consumers that
            // stop at full and at a site's ceiling, producers' young that lie
            // dormant and wake, and the dead that age, spoil and rot.
            limits: Some(Default::default()),
            dormancy: Some(crate::rules::Dormancy::flat()),
            rot: Some(ecology::Rot { decomposer_bites: Some(ecology::DECOMPOSER_BITES), ..Default::default() }),
            spoilage: Some(Default::default()),
            repertoire: Some(crate::repertoire::Repertoire {
                founders: true,
                across_kingdoms: true,
                kingdom_draw: Some(Default::default()),
                ..Default::default()
            }),
            place_memory: Some(Default::default()),
            played: Some(Played { lineage: 1, region_sites: 1 }),
            ..Default::default()
        };
        let grid = Self::starter_grid(seed);
        draft.sites = grid.width * grid.height;
        draft.map = Some(map::Layout::Grid(grid));
        let drawn = draft.repertoire.as_ref().and_then(|r| r.kingdom_draw.as_ref()).expect("set above");
        let roster = drawn.draw(seed).expect("valid default draw");
        draft.lineages = roster.iter().map(|(_, n)| n).sum();
        let bodied: u32 = roster.iter().filter(|(form, _)| form.bodied()).map(|(_, n)| n).sum();
        draft.population = draft.population.max(MEMBERS * u64::from(bodied)).max(COVER * u64::from(draft.sites));
        draft
    }

    /// The map `seed` draws for [`Founding::starter`]: a plane, or one seed
    /// in four a sphere folded from six faces.
    #[must_use]
    pub fn starter_grid(seed: u64) -> map::Grid {
        let mut grid = map::Grid::drawn(seed);
        if crate::draw(seed, "world-outline", &[]) % 4 == 0 {
            let face = 1 + (crate::draw(seed, "world-face", &[]) % 6) as u32;
            grid.width = 3 * face;
            grid.height = 2 * face;
            grid.shape = "shape:sphere".into();
        }
        grid
    }
}
