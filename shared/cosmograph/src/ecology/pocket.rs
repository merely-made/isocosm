// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A host can house an independent resident in an open, living exchange pocket.
//! The contact is between two individuals. Sharing a lineage or site gives no contact.
use crate::{
    Founding, Result,
    generate::process,
    process::Trade::{Gives, Takes},
    rules::*,
    schema::{Entity, PartId, Relation},
    simulation::Genesis,
};
use serde::{Deserialize, Serialize};

pub const CONTACT: &str = "pocket:houses";
pub const SETTLE: &str = "pocket:settle";
pub const LEAVE: &str = "pocket:leave";
pub const TRADE: &str = "pocket:exchange";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pocket {
    pub host: u32,
    pub resident: u32,
    /// Start the first compatible pair at each site in contact.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub settle_founders: bool,
    /// Developmental run bearing the two exchange walls.
    pub run: u8,
}

/// The seat and its two opposing, tissue-backed exchange walls, in body space.
pub fn opening(e: &Entity, run: u8) -> Option<[PartId; 3]> {
    let doc = e.body.as_ref()?;
    let (floor, _) = e
        .living()
        .find(|(id, _)| e.situs(*id) == Some([run, 0, 0]))?;
    let middle = doc.world_pivot(floor)?;
    let extent = |id| {
        doc.world_yaw(id)
            .map(|yaw| yaw.rotate(e.extent(id)).map(i32::abs))
    };
    let seat = extent(floor)?;
    let top = middle[1] + seat[1];
    let mut walls = [[None; 2]; 2];
    for (id, part) in e.living() {
        if id == floor {
            continue;
        }
        let pos = doc.world_pivot(id)?;
        let half = extent(id)?;
        let dx = pos[0] - middle[0];
        let dz = pos[2] - middle[2];
        if dx.abs() < half[0] && dz.abs() < half[2] && pos[1] + half[1] > top {
            return None;
        }
        if e.parent_of(id) != Some(floor)
            || pos[1] + half[1] <= top
            || pos[1] - half[1] > top
            || !crate::mosaic::ports::active(part)
                .is_some_and(|p| p.trades(Gives) && p.trades(Takes))
        {
            continue;
        }
        if dx.abs() == seat[0] + half[0] && dz.abs() < half[2] {
            walls[0][usize::from(dx > 0)] = Some(id);
        } else if dz.abs() == seat[2] + half[2] && dx.abs() < half[0] {
            walls[1][usize::from(dz > 0)] = Some(id);
        }
    }
    walls
        .into_iter()
        .find_map(|[left, right]| Some([floor, left?, right?]))
}

/// Founding can place an initial resident with a compatible host. Read the
/// developed founders in identity order, once per stored group.
pub(crate) fn founding_contacts(g: &Genesis) -> std::collections::BTreeSet<Relation> {
    let mut contacts = std::collections::BTreeSet::new();
    let Some(cfg) = g
        .founding
        .as_ref()
        .and_then(|f| f.pocket)
        .filter(|p| p.settle_founders)
    else {
        return contacts;
    };
    let host = format!("lineage:{}", cfg.host);
    let resident = format!("lineage:{}", cfg.resident);
    let mut sites = std::collections::BTreeMap::new();
    for (&id, group) in &g.population.groups {
        let body = &group.entity;
        let Some(place) = body.resident_site().filter(|_| body.alive) else {
            continue;
        };
        let (h, r) = sites.entry(place).or_insert((None, None));
        if body.lineage == host && h.is_none() && opening(body, cfg.run).is_some() {
            *h = Some(id);
        }
        if body.lineage == resident
            && r.is_none()
            && super::trades(body, Gives)
            && super::trades(body, Takes)
        {
            *r = Some(id);
        }
    }
    for (h, r) in sites.into_values() {
        if let (Some(host), Some(resident)) = (h, r) {
            contacts.insert(Relation::new(host, CONTACT, resident));
        }
    }
    contacts
}

pub(crate) fn apply(founding: &Founding, g: &mut Genesis) -> Result<()> {
    let Some(pocket) = founding.pocket else {
        return Ok(());
    };
    let rule = founding
        .mutualism
        .ok_or("a resident pocket needs mutualism")?;
    let (h, r) = (pocket.host, pocket.resident);
    if h >= founding.lineages
        || r >= founding.lineages
        || super::Role::of(founding.web, h) != super::Role::Decomposer
        || super::Role::of(founding.web, r) != super::Role::Producer
    {
        return Err("a pocket needs a decomposer host and a producer resident".into());
    }
    if founding
        .body_maps
        .get(&format!("lineage:{h}"))
        .is_none_or(|m| usize::from(pocket.run) >= m.grammar.seed.len())
    {
        return Err("the pocket run is missing from the host map".into());
    }
    use Binding::{Actor, Place, Target as Guest};
    let host = format!("matter:{h}-0");
    let sugar = format!("matter:{r}-0");
    let soil = "world:soil";
    let partner = Target {
        same_place: true,
        alive: Some(true),
        among: [format!("lineage:{r}")].into(),
        lineage: None,
        weighted: false,
    };
    let actor = Query::Trait {
        who: Actor,
        key: format!("ability:cycle-{h}"),
    };
    let guest = Query::Trait {
        who: Guest,
        key: format!("ability:cycle-{r}"),
    };
    let opening = Query::Pocket {
        who: Actor,
        run: pocket.run,
    };
    let organ = Query::Trades {
        who: Guest,
        what: Gives,
    };
    let takes = Query::Trades {
        who: Guest,
        what: Takes,
    };
    let related = Query::Related {
        kind: CONTACT.into(),
    };
    let account = |who, key: &str, at_least| Query::Account {
        who,
        key: key.into(),
        at_least,
    };
    let move_stock = |from, to, key: &str, n: u64| Effect::Transfer {
        from,
        to,
        account: key.into(),
        amount: n.into(),
    };
    let change = |present| {
        vec![Effect::Relate {
            kind: CONTACT.into(),
            present,
        }]
    };
    let mut settle = process(SETTLE, Causation::Choice, change(true));
    settle.requires.extend([
        Query::PocketVacant,
        opening.clone(),
        organ.clone(),
        takes.clone(),
    ]);
    let mut leave = process(LEAVE, Causation::Choice, change(false));
    leave.requires.push(related.clone());
    let mut trade = process(
        TRADE,
        Causation::Transition,
        vec![
            move_stock(Place, Actor, soil, rule.nutrient),
            move_stock(Actor, Guest, soil, rule.nutrient),
            move_stock(Guest, Actor, &sugar, rule.sugar),
            Effect::Spend {
                from: vec![host.clone()],
                to: Place,
                amount: rule.fee.into(),
                into: Some(soil.into()),
            },
        ],
    );
    trade.requires.extend([
        related,
        opening,
        organ,
        takes,
        account(Place, soil, rule.nutrient),
        account(Guest, &sugar, rule.sugar),
        account(Actor, &host, rule.fee),
    ]);
    trade.period = Some(minutes_ticks(rule.period, g.rules.tick_microseconds()));
    trade.priority = 1;
    // Replace only this pair's site-wide gifts. The resident still fixes with
    // received soil, and the host still has to digest the sugar it receives.
    for key in [
        format!("mutualism:nutrient-{h}-{r}"),
        format!("mutualism:sugar-{r}-{h}"),
    ] {
        g.rules
            .processes
            .remove(&key)
            .ok_or("pocket requires the native mutualism turns")?;
    }
    g.rules.relations.insert(CONTACT.into());
    for mut p in [settle, leave, trade] {
        p.target = Some(partner.clone());
        p.requires
            .extend([actor.clone(), guest.clone(), Query::Alive(Guest)]);
        g.rules.processes.insert(p.id.clone(), p);
    }
    Ok(())
}
