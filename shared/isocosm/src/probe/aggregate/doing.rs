// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! One state's act as its members do it: what they read and write of
//! themselves, their site and a meal's prey through `meaning::Parties`,
//! their effects in order, and their takes of shared ground planned or held
//! to their shares (ruling 454).

use super::*;

mod harm;

/// One state's members as their act leaves them, and what the act reads.
pub(super) struct Doing<'a> {
    pub(super) tick: Tick,
    pub(super) rules: &'a Rules,
    pub(super) lineages: Lineages<'a>,
    pub(super) member: Entity,
    pub(super) part: Option<PartId>,
    /// The site as every member's writes leave it.
    pub(super) site: Site,
    /// The site as one member's act sees it: the pass's start and its own
    /// writes.
    pub(super) seen: Site,
    pub(super) count: u64,
    /// The prey as one hunter's act sees it, the share it takes, and the
    /// part its bite lands on.
    pub(super) prey: Option<Entity>,
    pub(super) portion: Option<Ledger>,
    pub(super) bitten: Option<PartId>,
    /// The target is a body the act binds whole, a member's own young
    /// (554), rather than a meal's prey.
    pub(super) bound: bool,
    pub(super) kept: BTreeMap<Key, i64>,
    /// What a carriage lets the member's and the target's parts still take
    /// (581).
    pub(super) caps: [Option<crate::anatomy::Caps>; 2],
    pub(super) draws: &'a BTreeMap<u8, u64>,
    /// What its meal took of the prey, the kinds its lineage learned, and
    /// the births its act made, which the crowd draws for each member.
    pub(super) took: Option<Took>,
    pub(super) lessons: Vec<(Key, Key)>,
    pub(super) born: Vec<Pending>,
}

impl Parties for Doing<'_> {
    fn held(&mut self, who: Binding, key: &str) -> Result<u64> {
        match who {
            Binding::Actor => Ok(crate::anatomy::held(&self.member, self.rules, key)),
            // One member's share of a shared site is the whole of it only
            // where the act stands for one member, the world's own.
            Binding::Place if self.count == 1 => Ok(meaning::value(&self.seen.accounts, key)),
            Binding::Target => match &self.prey {
                Some(prey) => Ok(crate::anatomy::held(prey, self.rules, key)),
                None => Err("the crowd has no target".into()),
            },
            _ => Err("the crowd reads no one member's share of shared ground".into()),
        }
    }
    fn reach(&mut self, who: Binding) -> Result<()> {
        match who {
            Binding::Target if self.prey.is_none() => Err("the crowd has no target".into()),
            _ => Ok(()),
        }
    }
    fn take(&mut self, who: Binding, key: &str, amount: u64) -> Result<()> {
        match who {
            Binding::Actor => match crate::anatomy::take_within(
                &mut self.member,
                self.rules,
                key,
                amount,
                self.caps[0].as_mut(),
            ) {
                Some(done) => done.map(|_| ()),
                None => debit(&mut self.member.accounts, key, amount),
            },
            // Shares make every take of the site whole (ruling 454).
            Binding::Place => {
                let total = amount.checked_mul(self.count).ok_or("amount overflow")?;
                debit(&mut self.site.accounts, key, total)?;
                debit(&mut self.seen.accounts, key, amount)
            },
            Binding::Target if self.bound => {
                let target = self.prey.as_mut().ok_or("no target is bound")?;
                let caps = self.caps[1].as_mut();
                match crate::anatomy::take_within(target, self.rules, key, amount, caps) {
                    Some(done) => done.map(|_| ()),
                    None => debit(&mut target.accounts, key, amount),
                }
            },
            Binding::Target => Err("the crowd takes from a prey only by its meal".into()),
            Binding::Part => Err("a part keeps no ledger".into()),
        }
    }
    fn give(&mut self, who: Binding, key: &str, amount: u64) -> Result<()> {
        match who {
            Binding::Actor => match crate::anatomy::give_within(
                &mut self.member,
                self.rules,
                key,
                amount,
                self.caps[0].as_mut(),
            ) {
                Some(done) => done.map(|_| ()),
                None => credit(&mut self.member.accounts, key, amount),
            },
            Binding::Place => {
                let total = amount.checked_mul(self.count).ok_or("amount overflow")?;
                credit(&mut self.site.accounts, key, total)?;
                credit(&mut self.seen.accounts, key, amount)
            },
            Binding::Target if self.bound => {
                let target = self.prey.as_mut().ok_or("no target is bound")?;
                let caps = self.caps[1].as_mut();
                match crate::anatomy::give_within(target, self.rules, key, amount, caps) {
                    Some(done) => done.map(|_| ()),
                    None => credit(&mut target.accounts, key, amount),
                }
            },
            Binding::Target => Err("the crowd gives a prey nothing".into()),
            Binding::Part => Err("a part keeps no ledger".into()),
        }
    }
    fn body(&mut self, who: Binding) -> Result<&mut Entity> {
        match who {
            Binding::Actor => Ok(&mut self.member),
            Binding::Target if self.bound => self.prey.as_mut().ok_or("no target is bound".into()),
            _ => Err("the crowd writes only the actor's body".into()),
        }
    }
    fn part(&mut self) -> Result<(&mut Entity, PartId)> {
        let part = self.part.ok_or("no part is bound")?;
        Ok((&mut self.member, part))
    }
    fn shift(&mut self, key: &str, delta: i64) -> Result<()> {
        meaning::shift(&mut self.site.conditions, key, delta, self.count)?;
        meaning::shift(&mut self.seen.conditions, key, delta, 1)
    }
    fn development(&mut self, lineage: &str) -> Result<Development> {
        let found = self.lineages.and_then(|l| l.get(lineage));
        found
            .and_then(|l| l.development.clone())
            .ok_or_else(|| format!("{lineage}'s bodies cannot grow here"))
    }
    fn bitten(&self) -> Option<PartId> {
        self.bitten
    }
    fn bound(&mut self, who: Binding, caps: crate::anatomy::Caps) -> Result<()> {
        match who {
            Binding::Actor => self.caps[0] = Some(caps),
            Binding::Target if self.prey.is_some() => self.caps[1] = Some(caps),
            _ => return Err(format!("{who:?} has no parts to bound")),
        }
        Ok(())
    }
    fn room(&mut self, who: Binding, key: &str) -> Result<u64> {
        let (body, caps) = match who {
            Binding::Actor => (&self.member, &self.caps[0]),
            Binding::Target => (self.prey.as_ref().ok_or("no target")?, &self.caps[1]),
            _ => return Err(format!("{who:?} has no parts")),
        };
        Ok(crate::anatomy::room_within(
            body,
            self.rules,
            key,
            caps.as_ref(),
        ))
    }
}

impl Doing<'_> {
    /// The bitten part taken whole, where the share would take all of it
    /// and its crossing and the eater's plan let it land (516, 544); whether
    /// it was.
    fn whole(&mut self, of: &[Key], portion: &Ledger) -> Result<bool> {
        let (Some(prey), Some(id)) = (self.prey.as_mut(), self.bitten) else {
            return Ok(false);
        };
        let Some(part) = prey.parts.get(&id) else {
            return Ok(false);
        };
        let all: u64 = edible(&part.matter, of).values().sum();
        if portion.values().sum::<u64>() < all {
            return Ok(false);
        }
        let developed = |l: &Key| {
            let found = self.lineages.and_then(|m| m.get(l));
            found.and_then(|l| l.development.clone())
        };
        let (Some(mine), Some(theirs)) =
            (developed(&self.member.lineage), developed(&prey.lineage))
        else {
            return Ok(false);
        };
        let affinity = self.rules.affinity.clone().unwrap_or_default();
        let bodies = (&self.member, &*prey);
        let Some(w) = growth::whole(bodies, id, (&mine, &theirs), &affinity) else {
            return Ok(false);
        };
        growth::take_whole(&mut self.member, prey, id, &w).ok_or("bitten part missing")?;
        let eater = self.member.lineage.clone();
        self.lessons.extend(w.kind.map(|k| (eater, k)));
        self.took = Some(Took::Whole(id));
        Ok(true)
    }

    /// Computes what an amount or a guard reads, as the core's stage does.
    pub(super) fn compute<T>(
        &mut self,
        f: impl FnOnce(&mut Read, &mut Draw, &mut PartsOf) -> Result<T>,
    ) -> Result<T> {
        let living = |e: &Entity| -> Vec<(crate::anatomy::Half, Part)> {
            e.living()
                .map(|(id, p)| (e.extent(id), p.clone()))
                .collect()
        };
        let (mine, theirs) = (living(&self.member), self.prey.as_ref().map(living));
        let (rules, b, lineages) = (self.rules, self.rules.body(), self.lineages);
        let mut parts = |who: Binding| -> Result<(Vec<(crate::anatomy::Half, Part)>, BodyRules)> {
            match who {
                Binding::Actor => Ok((mine.clone(), b)),
                Binding::Target => match theirs.clone() {
                    Some(t) => Ok((t, b)),
                    None => Err("no target is bound".into()),
                },
                _ => Err(format!("{who:?} has no parts")),
            }
        };
        let (member, seen, prey, kept) = (&self.member, &self.seen, &self.prey, &self.kept);
        let (caps, bitten) = (&self.caps, self.bitten);
        let part = self
            .part
            .and_then(|id| Some((member.extent(id), member.parts.get(&id)?)));
        let mut read = |r: &Reading| -> Result<i64> {
            let body = |who: Binding| match who {
                Binding::Actor => Ok(member),
                Binding::Target => prey.as_ref().ok_or("no target is bound"),
                _ => Err("not a body"),
            };
            match (r, r.who()) {
                (Reading::Kept { name }, _) => kept
                    .get(name)
                    .copied()
                    .ok_or_else(|| format!("no value {name} was kept")),
                (Reading::Account { key, .. }, Binding::Place) => {
                    i64::try_from(meaning::value(&seen.accounts, key)).map_err(|e| e.to_string())
                },
                (Reading::Account { key, .. }, Binding::Part) => {
                    let (_, part) = part.ok_or("no part is bound")?;
                    i64::try_from(meaning::value(&part.matter, key)).map_err(|e| e.to_string())
                },
                (Reading::Account { key, .. }, who) => {
                    let held = crate::anatomy::held(body(who)?, rules, key);
                    i64::try_from(held).map_err(|e| e.to_string())
                },
                (Reading::Room { key, .. }, who) => {
                    let caps = if who == Binding::Target {
                        &caps[1]
                    } else {
                        &caps[0]
                    };
                    let room = crate::anatomy::room_within(body(who)?, rules, key, caps.as_ref());
                    i64::try_from(room).map_err(|e| e.to_string())
                },
                (Reading::Carried { .. }, who) => {
                    let e = body(who)?;
                    let l = lineages.and_then(|l| l.get(&e.lineage));
                    let d = l.and_then(|l| l.development.as_ref());
                    Ok(meaning::carried(e, r, rules, d, bitten))
                },
                (r, Binding::Part) => {
                    let (half, part) = part.ok_or("no part is bound")?;
                    Ok(i64::try_from(r.of_part(half, part, b)).unwrap_or(i64::MAX))
                },
                (r, who) => Ok(meaning::body_reading(body(who)?, r, rules, lineages)),
            }
        };
        let draws = self.draws;
        let mut draw = |below: u64, slot: u8| -> Result<u64> {
            let fixed = draws.get(&slot).ok_or("a draw the crowd did not fix")?;
            Ok(fixed % below)
        };
        f(&mut read, &mut draw, &mut parts)
    }

    /// Applies `effects` in order; `Ok(false)` is the core's blocked
    /// outcome, an error what the crowd cannot apply at all.
    pub(super) fn run(
        &mut self,
        effects: &[&Effect],
        rules: &Rules,
        how: &mut Run,
        left: &mut Ledger,
    ) -> Result<bool> {
        for &effect in effects {
            if !self.one(effect, rules, how, left)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn one(&mut self, e: &Effect, rules: &Rules, how: &mut Run, left: &mut Ledger) -> Result<bool> {
        match e {
            Effect::Wound { who, cells, slot } => {
                let resolved = self.compute(|r, d, p| cells.resolve(r, d, p))?;
                self.wound(*who, resolved.resolved()?, *slot)
            },
            Effect::Rot { who, amount } => {
                let resolved = self.compute(|r, d, p| amount.resolve(r, d, p))?;
                self.rot(*who, resolved.resolved()?)
            },
            Effect::Keep { name, value } => {
                let Ok(v) = self.compute(|r, d, p| value.eval_in(r, d, p)) else {
                    return Ok(false);
                };
                self.kept.insert(name.clone(), v);
                Ok(true)
            },
            Effect::When { .. } => {
                let Ok(branch) = self.compute(|r, d, p| e.branch(r, d, p)) else {
                    return Ok(false);
                };
                let branch = branch.expect("a guard chooses a branch");
                self.run(&branch.iter().collect::<Vec<_>>(), rules, how, left)
            },
            // The meal's share of its prey, as its pass shared it out: what
            // the part its bite lands on held of it as the pass began (459),
            // or that part whole where the share would take all of it (516).
            Effect::Eat {
                into, of, whole, ..
            } => {
                let Some(portion) = self.portion.take() else {
                    return Err("the crowd eats only in a hunt".into());
                };
                if *whole && self.whole(of, &portion)? {
                    return Ok(true);
                }
                let prey = self.prey.as_mut().ok_or("the crowd eats only in a hunt")?;
                let held = match self.bitten.and_then(|id| prey.parts.get_mut(&id)) {
                    Some(part) => &mut part.matter,
                    None => &mut prey.accounts,
                };
                let offered = edible(held, of);
                let taken: Ledger = portion
                    .into_iter()
                    .map(|(k, v)| {
                        let v = v.min(offered.get(&k).copied().unwrap_or(0));
                        (k, v)
                    })
                    .filter(|(_, v)| *v > 0)
                    .collect();
                let mut total = 0u64;
                for (key, value) in &taken {
                    debit(held, key, *value)?;
                    total += value;
                }
                credit(&mut self.member.accounts, into, total)?;
                self.took = Some(Took::Bite(taken));
                Ok(true)
            },
            // Births, for the crowd to make member by member (447, 448).
            Effect::Bear { clutch, young } => {
                let Ok((lineal, shares)) = births::bear(self, rules, *clutch) else {
                    return Ok(false);
                };
                self.born.push(Pending::Hatch {
                    lineal,
                    clutch: *clutch,
                    shares,
                    young: young.clone(),
                    parent: self.member.clone(),
                });
                Ok(true)
            },
            Effect::Bud { mark, once } => {
                let Ok(b) = births::bud(self, rules, mark) else {
                    return Ok(false);
                };
                if let Some((_, part)) = b.severed {
                    // The child of its parent as it was, the semelparous
                    // parent dying as it severs (521).
                    self.born.push(Pending::Seedling {
                        lineal: b.lineal,
                        part,
                        mark: mark.clone(),
                        parent: self.member.clone(),
                    });
                    self.member.alive &= !*once;
                }
                Ok(true)
            },
            _ => {
                let resolved = match e.computes() {
                    true => match self.compute(|r, d, p| e.resolve(r, d, p)) {
                        Ok(resolved) => resolved,
                        Err(_) => return Ok(false),
                    },
                    false => e.clone(),
                };
                let shared = share_out(&resolved, how, left, &self.seen)?;
                let effect = shared.unwrap_or(resolved);
                match meaning::effect(self, rules, &effect) {
                    None => Err(format!("the crowd cannot apply {effect:?}")),
                    Some(Ok(())) => Ok(true),
                    Some(Err(_)) => Ok(false),
                }
            },
        }
    }
}

/// How a state's act runs: planned on a copy of its site, each member's
/// takes of it recorded; applied with each member's shares; or, outside a
/// pass, as it asks, as the core runs a host's act.
pub(in crate::probe) enum Run<'a> {
    Plan(&'a mut Ledger),
    Act(&'a Ledger),
    Free,
}

/// One member's take of its site, recorded while planning and held to its
/// share when acting; `None` leaves the effect as it was.
fn share_out(e: &Effect, run: &mut Run, left: &mut Ledger, site: &Site) -> Result<Option<Effect>> {
    if matches!(run, Run::Free) {
        return Ok(None);
    }
    match e {
        Effect::Transfer {
            from: Binding::Place,
            account,
            amount,
            ..
        } => {
            let asked = amount.resolved()?;
            let given = match run {
                // A plan asks for the whole take and stages what is there.
                Run::Plan(takes) => {
                    let t = takes.entry(account.clone()).or_default();
                    *t = t.saturating_add(asked);
                    asked.min(meaning::value(&site.accounts, account))
                },
                Run::Act(_) | Run::Free => {
                    let share = left.entry(account.clone()).or_default();
                    let given = asked.min(*share);
                    *share -= given;
                    given
                },
            };
            let mut e = e.clone();
            if let Effect::Transfer { amount, .. } = &mut e {
                *amount = Amount::Fixed(given);
            }
            Ok(Some(e))
        },
        Effect::Transform {
            who: Binding::Place,
            take,
            ..
        } => {
            for (k, v) in take {
                match run {
                    Run::Plan(takes) => {
                        let t = takes.entry(k.clone()).or_default();
                        *t = t.saturating_add(*v);
                    },
                    Run::Act(_) | Run::Free => {
                        let share = left.entry(k.clone()).or_default();
                        if *share < *v {
                            return Err("a transform of shared ground cannot be shared out".into());
                        }
                        *share -= v;
                    },
                }
            }
            Ok(None)
        },
        _ => Ok(None),
    }
}
