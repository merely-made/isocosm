//! The Player Core order of creation: ancestry, heritage, background, class,
//! attributes, skills, then feats.

use super::*;

fn record_options<'r, T: 'r>(
    items: impl Iterator<Item = &'r T>,
    view: impl Fn(&T) -> (&str, &str, &str),
) -> Vec<Opt> {
    items
        .map(|item| {
            let (id, name, summary) = view(item);
            Opt {
                value: id.to_owned(),
                label: name.to_owned(),
                summary: summary.to_owned(),
                refused: None,
            }
        })
        .collect()
}

fn slot(id: impl Into<String>, kind: &'static str, title: impl Into<String>, options: Vec<Opt>) -> Slot {
    Slot {
        id: id.into(),
        kind,
        title: title.into(),
        options,
    }
}

impl Walk<'_> {
    fn boost(&mut self, id: String, title: String, kind: &'static str, source: &str, allowed: &[String])
        -> Result<(), Stop> {
        let options = attribute_options(&self.build, source, allowed);
        let value = self.take(slot(id, kind, title, options))?;
        self.build.boosts.push((source.to_owned(), value));
        Ok(())
    }
}

pub(super) fn run(w: &mut Walk<'_>) -> Result<(), Stop> {
    let r = w.r;
    // Ancestry and its boosts.
    let options = record_options(r.ancestries.values(), |a| (&a.id, &a.name, &a.summary));
    let ancestry = &r.ancestries[&w.take(slot("ancestry", "ancestry", "Ancestry", options))?];
    w.build.ancestry = Some(ancestry.id.clone());
    w.build.languages.extend(ancestry.languages.iter().cloned());
    w.build.gear.extend(ancestry.items.iter().cloned());
    w.build.coins = STARTING_COINS;
    let listed = ancestry.boosts.iter().any(|b| !b.is_empty()) || !ancestry.flaws.is_empty();
    let alternate = listed && {
        let fixed: Vec<_> = ancestry.boosts.iter().filter(|b| !b.is_empty())
            .map(|b| b.iter().map(|a| records::attribute_name(a)).collect::<Vec<_>>().join(" or ")).collect();
        let free = ancestry.boosts.iter().filter(|b| b.is_empty()).count();
        let flaws: Vec<_> = ancestry.flaws.iter().map(|a| records::attribute_name(a)).collect();
        let options = vec![
            Opt { value: "listed".into(), label: "Listed boosts".into(), refused: None,
                summary: format!("{}, {free} free boost(s), and a flaw to {}.", fixed.join(", "), flaws.join(", ")) },
            Opt { value: "free".into(), label: "Two free boosts".into(), refused: None,
                summary: "Two free boosts to different attributes and no flaw.".into() },
        ];
        w.take(slot("ancestry-boosts", "ancestry-boosts", "Ancestry boosts", options))? == "free"
    };
    let boosts: Vec<Vec<String>> = if alternate { vec![vec![], vec![]] } else { ancestry.boosts.clone() };
    if !alternate {
        w.build.flaws.extend(ancestry.flaws.iter().cloned());
    }
    for (n, allowed) in boosts.iter().enumerate() {
        if allowed.len() == 1 {
            w.build.boosts.push(("ancestry".into(), allowed[0].clone()));
        } else {
            w.boost(format!("ancestry-boost:{n}"), format!("Ancestry boost {}", n + 1), "ancestry-boost", "ancestry", allowed)?;
        }
    }
    // Heritage.
    let options = record_options(r.heritages.values().filter(|h| h.ancestry == ancestry.id), |h| (&h.id, &h.name, &h.summary));
    let heritage = &r.heritages[&w.take(slot("heritage", "heritage", "Heritage", options))?];
    w.build.heritage = Some(heritage.id.clone());
    let mut heritage_feat = None;
    for (n, grant) in heritage.grants.iter().enumerate() {
        match grant {
            // Chosen with the other feats, once the attributes are known.
            records::Grant::FeatChoice(category) => heritage_feat = Some(category.clone()),
            other => w.grant(other, &format!("heritage/{n}"), &heritage.name)?,
        }
    }
    // Background.
    let options = r.backgrounds.values().map(|b| Opt {
        value: b.id.clone(), label: b.name.clone(), summary: b.summary.clone(), refused: b.unavailable.clone(),
    }).collect();
    let background = &r.backgrounds[&w.take(slot("background", "background", "Background", options))?];
    w.build.background = Some(background.id.clone());
    w.boost("background-boost".into(), "Background boost".into(), "background-boost", "background", &background.boosts)?;
    w.boost("background-free-boost".into(), "Free background boost".into(), "background-free-boost", "background", &[])?;
    if let Some(skill) = &background.skill {
        w.train(skill, "background/skill", &background.name)?;
    }
    let lore = if background.lore.len() == 1 {
        background.lore[0].clone()
    } else {
        let options = background.lore.iter().map(|l| Opt {
            value: l.clone(), label: l.clone(), summary: "A Lore keys off Intelligence.".into(), refused: None,
        }).collect();
        w.take(slot("background/lore", "lore", format!("{} Lore", background.name), options))?
    };
    w.build.lores.push(lore);
    for (n, grant) in background.grants.iter().enumerate() {
        w.grant(grant, &format!("background/{n}"), &background.name)?;
    }
    if let Some(feat) = &background.feat {
        w.feat(feat, "background/feat")?;
    }
    // Class.
    let options = record_options(r.classes.values(), |c| (&c.id, &c.name, &c.summary));
    let class = &r.classes[&w.take(slot("class", "class", "Class", options))?];
    w.build.class = Some(class.id.clone());
    for (k, v) in &class.ranks {
        let rank = w.build.ranks.entry(k.clone()).or_default();
        *rank = (*rank).max(*v);
    }
    for (n, grant) in class.grants.iter().enumerate() {
        w.grant(grant, &format!("class/{n}"), &class.name)?;
    }
    let mut keys = class.key.clone();
    if let Some(group) = &class.option {
        let options = record_options(
            r.feats.values().filter(|f| f.option.as_ref() == Some(group) && f.owner.as_ref() == Some(&class.id)),
            |f| (&f.id, &f.name, &f.summary),
        );
        let racket = w.take(slot("racket", "racket", records::title(group), options))?;
        keys.extend(r.feats[&racket].key.iter().cloned());
        w.build.racket = Some(racket.clone());
        w.feat(&racket, "racket")?;
    }
    for skill in &class.skills {
        w.train(skill, &format!("class/{skill}"), &class.name)?;
    }
    if !class.skill_choice.is_empty() {
        w.pick_skill("class-skill", &format!("{} skill", class.name), &class.skill_choice)?;
    }
    // Key attribute and the four free boosts.
    let options = attribute_options(&w.build, "class", &keys);
    let key = w.take(slot("key-attribute", "key-attribute", "Key attribute", options))?;
    w.build.boosts.push(("class".into(), key.clone()));
    w.build.key = Some(key);
    for n in 0..4 {
        w.boost(format!("free-boost:{n}"), format!("Free boost {}", n + 1), "free-boost", "free", &[])?;
    }
    // Skills: the class's count plus Intelligence.
    let picks = (class.additional_skills + w.build.modifier("int")).max(0);
    for n in 0..picks {
        w.pick_skill(&format!("skill:{n}"), &format!("Trained skill {} of {picks}", n + 1), &[])?;
    }
    // Languages: the ancestry's extra picks plus Intelligence.
    let from = if ancestry.additional_languages.is_empty() { r.common_languages.clone() }
        else { ancestry.additional_languages.clone() };
    let picks = ancestry.extra_languages + w.build.modifier("int").max(0);
    for n in 0..picks {
        w.language(&format!("language:{n}"), &format!("Language {} of {picks}", n + 1), &from)?;
    }
    // Feats.
    w.feat_slot("ancestry-feat", "ancestry-feat", "Ancestry feat", "ancestry")?;
    if let Some(category) = heritage_feat {
        w.feat_slot("heritage-feat", "heritage-feat", &format!("{} feat", heritage.name), &category)?;
    }
    if class.skill_feat_at_first {
        w.feat_slot("skill-feat", "skill-feat", "Skill feat", "skill")?;
    }
    w.feat_slot("class-feat", "class-feat", "Class feat", "class")?;
    // Starting gear, one purchase at a time.
    for n in 0.. {
        let mut options = vec![Opt { value: "done".into(), label: "Stop buying".into(), refused: None,
            summary: format!("Keep {} as coins.", money(w.build.coins)) }];
        options.extend(r.items.values().map(|i| Opt {
            value: i.id.clone(), label: format!("{} ({})", i.name, money(i.price)), summary: i.summary.clone(),
            refused: if i.price > w.build.coins {
                Some(format!("costs {}; {} left", money(i.price), money(w.build.coins)))
            } else if i.rarity != "common" && !access(&w.build, i) {
                Some(format!("{}: needs access", i.rarity))
            } else { None },
        }));
        let title = format!("Starting gear: purchase {} ({} left)", n + 1, money(w.build.coins));
        let item = w.take(slot(format!("gear:{n}"), "gear", title, options))?;
        if item == "done" {
            break;
        }
        w.build.coins -= r.items[&item].price;
        w.build.gear.push(item);
    }
    Ok(())
}

/// Access to an uncommon item: Dwarven Weapon Familiarity opens dwarf weapons.
fn access(build: &Build, item: &records::Item) -> bool {
    item.has("dwarf") && build.feats.iter().any(|f| f == "dwarven-weapon-familiarity")
}
