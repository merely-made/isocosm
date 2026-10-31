//! What feats and features grant, and when a feat is out of reach.

use super::*;
use records::{Feat, Grant, Requirement};

/// Why `req` is not met by `build`, if it is not.
fn unmet(r: &Records, build: &Build, req: &Requirement) -> Option<String> {
    let skills = |list: &[String]| list.iter().map(|s| records::title(s)).collect::<Vec<_>>().join(", ");
    match req {
        Requirement::Trained(skill) if !build.trained(skill) => {
            Some(format!("requires training in {}", records::title(skill)))
        },
        Requirement::AnyTrained(list) if list.is_empty() && build.skills.is_empty() => {
            Some("requires training in a skill".into())
        },
        Requirement::AnyTrained(list) if !list.is_empty() && !list.iter().any(|s| build.trained(s)) => {
            Some(format!("requires training in one of {}", skills(list)))
        },
        Requirement::Lore(name) if !build.lores.contains(name) => Some(format!("requires {name}")),
        Requirement::AnyLore if build.lores.is_empty() => Some("requires training in a Lore".into()),
        Requirement::Attribute { key, at_least } if build.modifier(key) < *at_least => Some(format!(
            "requires {} {at_least:+}; yours is {:+}",
            records::attribute_name(key),
            build.modifier(key)
        )),
        Requirement::Feat(other) if !build.feats.contains(other) => {
            Some(format!("requires the {} feat", r.feats[other].name))
        },
        Requirement::AnyOf(list) => {
            let missing: Vec<_> = list.iter().filter_map(|q| unmet(r, build, q)).collect();
            (missing.len() == list.len()).then(|| missing.join("; or "))
        },
        Requirement::Unavailable(why) => Some(why.clone()),
        _ => None,
    }
}

/// Why `feat` cannot be taken by `build`, if it cannot.
pub(super) fn refusal(r: &Records, build: &Build, feat: &Feat) -> Option<String> {
    if build.feats.contains(&feat.id) && !feat.repeatable {
        return Some("already taken".into());
    }
    if let Some(why) = feat.requires.iter().find_map(|q| unmet(r, build, q)) {
        return Some(why);
    }
    for grant in &feat.grants {
        match grant {
            Grant::ArmorStep
                if ["light", "medium", "heavy"].iter().all(|k| build.ranks.get(*k).is_some_and(|r| *r > 0)) =>
            {
                return Some("you are already trained in every armour category".into());
            },
            Grant::RankChoice { options, rank }
                if options.iter().all(|o| build.ranks.get(o).is_some_and(|r| r >= rank)) =>
            {
                return Some("every option is already at that rank".into());
            },
            Grant::OtherAncestry
                if !r.ancestries.keys().any(|a| Some(a) != build.ancestry.as_ref()) =>
            {
                return Some("no other ancestry is in these records".into());
            },
            _ => {},
        }
    }
    None
}

impl Walk<'_> {
    /// One trained skill picked from `from`; when every one of those is
    /// already trained, any untrained skill (Player Core's rule for doubled
    /// training).
    pub(super) fn pick_skill(&mut self, id: &str, title: &str, from: &[String]) -> Result<(), Stop> {
        let mut options = skill_options(&self.build, from);
        if options.iter().all(|o| o.refused.is_some()) {
            options = skill_options(&self.build, &[]);
        }
        let skill = self.take(Slot { id: id.into(), kind: "skill", title: title.into(), options })?;
        self.build.skills.insert(skill, 1);
        Ok(())
    }

    /// Fixed training; already trained, a free pick replaces it.
    pub(super) fn train(&mut self, skill: &str, id: &str, by: &str) -> Result<(), Stop> {
        if self.build.trained(skill) {
            let title = format!("{by}: already trained in {}, choose another skill", records::title(skill));
            return self.pick_skill(&format!("{id}/again"), &title, &[]);
        }
        self.build.skills.insert(skill.to_owned(), 1);
        Ok(())
    }

    /// Gain a feat or feature and whatever it grants.
    pub(super) fn feat(&mut self, id: &str, prefix: &str) -> Result<(), Stop> {
        let r = self.r;
        let feat = &r.feats[id];
        self.build.feats.push(id.to_owned());
        for (n, grant) in feat.grants.iter().enumerate() {
            self.grant(grant, &format!("{prefix}/{n}"), &feat.name)?;
        }
        Ok(())
    }

    /// Choose a feat of `category` (class feats are the build's class's).
    pub(super) fn feat_slot(&mut self, id: &str, kind: &'static str, title: &str, category: &str) -> Result<(), Stop> {
        let r = self.r;
        let owner = match category {
            "ancestry" => self.build.ancestry.clone(),
            "class" => self.build.class.clone(),
            _ => None,
        };
        let options = r
            .feats
            .values()
            .filter(|f| f.in_category(category) && (owner.is_none() || f.owner == owner))
            .map(|f| Opt {
                value: f.id.clone(),
                label: f.name.clone(),
                summary: f.summary.clone(),
                refused: refusal(r, &self.build, f),
            })
            .collect();
        let feat = self.take(Slot { id: id.into(), kind, title: title.into(), options })?;
        self.feat(&feat, id)
    }

    pub(super) fn grant(&mut self, grant: &Grant, id: &str, by: &str) -> Result<(), Stop> {
        let b = &mut self.build;
        match grant {
            Grant::Train(skill) => return self.train(skill, id, by),
            Grant::TrainChoice { count, from } => {
                for n in 0..*count {
                    self.pick_skill(&format!("{id}/skill:{n}"), &format!("{by}: trained skill {}", n + 1), from)?;
                }
            },
            Grant::TrainElseFeat { skill, feat, choice } => {
                if !b.trained(skill) {
                    b.skills.insert(skill.clone(), 1);
                } else if !b.feats.contains(feat) {
                    return self.grant(&Grant::FeatWith { feat: feat.clone(), value: choice.clone() }, id, by);
                }
            },
            Grant::FeatWith { feat, value } => {
                let name = self.r.feats[feat].name.clone();
                b.feats.push(feat.clone());
                b.options.push((name, value.clone()));
            },
            Grant::Pick(options) => {
                let opts = options.keys().map(|k| Opt {
                    value: k.clone(), label: records::title(k), summary: String::new(), refused: None,
                }).collect();
                let key = self.take(Slot { id: format!("{id}/pick"), kind: "choice", title: by.to_owned(), options: opts })?;
                for (n, inner) in options[&key].iter().enumerate() {
                    self.grant(inner, &format!("{id}/{key}/{n}"), by)?;
                }
            },
            Grant::TrainedSkillChoice => {
                let options = records::SKILLS.iter().map(|(s, _)| Opt {
                    value: (*s).to_owned(),
                    label: records::title(s),
                    summary: String::new(),
                    refused: if !b.trained(s) {
                        Some("not trained".into())
                    } else if b.options.iter().any(|(k, v)| k == by && v == s) {
                        Some("already chosen for this feat".into())
                    } else {
                        None
                    },
                }).collect();
                let skill = self.take(Slot { id: format!("{id}/skill"), kind: "choice", title: format!("{by}: skill"), options })?;
                self.build.options.push((by.to_owned(), skill));
            },
            Grant::Lore(name) => {
                if !b.lores.contains(name) {
                    b.lores.push(name.clone());
                }
            },
            Grant::Feat(feat) => {
                if !b.feats.contains(feat) {
                    return self.feat(feat, &format!("{id}/{feat}"));
                }
            },
            Grant::FeatChoice(category) => {
                return self.feat_slot(&format!("{id}/feat"), "feat", &format!("{by}: {category} feat"), category);
            },
            Grant::Choice { key, options } => {
                let options = options.iter().map(|o| Opt {
                    value: o.clone(), label: records::title(o), summary: String::new(), refused: None,
                }).collect();
                let value = self.take(Slot { id: format!("{id}/{key}"), kind: "choice", title: format!("{by}: {key}"), options })?;
                self.build.options.push((by.to_owned(), value));
            },
            Grant::RankChoice { options, rank } => {
                let options = options.iter().map(|o| Opt {
                    value: o.clone(),
                    label: records::title(o),
                    summary: format!("Becomes rank {rank}."),
                    refused: b.ranks.get(o).is_some_and(|x| x >= rank).then(|| "already at that rank".into()),
                }).collect();
                let target = self.take(Slot { id: format!("{id}/rank"), kind: "rank", title: format!("{by}: proficiency"), options })?;
                let at = self.build.ranks.entry(target).or_default();
                *at = (*at).max(*rank);
            },
            Grant::Rank { target, rank } => {
                let at = b.ranks.entry(target.clone()).or_default();
                *at = (*at).max(*rank);
            },
            Grant::ArmorStep => {
                if let Some(k) = ["light", "medium", "heavy"].into_iter().find(|k| b.ranks.get(*k).is_none_or(|r| *r == 0)) {
                    b.ranks.insert(k.into(), 1);
                }
            },
            Grant::OtherAncestry => {
                let r = self.r;
                let options = r.ancestries.values().map(|a| Opt {
                    value: a.id.clone(),
                    label: a.name.clone(),
                    summary: a.summary.clone(),
                    refused: (Some(&a.id) == self.build.ancestry.as_ref()).then(|| "your own ancestry".into()),
                }).collect();
                let value = self.take(Slot { id: format!("{id}/ancestry"), kind: "other-ancestry", title: format!("{by}: ancestry"), options })?;
                self.build.options.push((by.to_owned(), value));
            },
            Grant::Speed(n) => b.speed_bonus += n,
            Grant::HitPointsPerLevel(n) => b.hp_per_level += n,
            Grant::DyingMax(n) => b.dying_max = Some(b.dying_max.unwrap_or(4).max(*n)),
            Grant::RecoveryDc(n) => b.recovery_dc += n,
            Grant::Initiative(n) => b.initiative += n,
            Grant::Resistance(kind) => b.resistances.push(kind.clone()),
        }
        Ok(())
    }
}
