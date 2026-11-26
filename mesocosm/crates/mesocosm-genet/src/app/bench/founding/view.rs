// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{Bench, Child, Page, Panel, Version};
use crate::app::bench::sim::{button, founding_input as input};
use cambium::{clickable, el, focusable, text};
use cosmograph::rules::Facing;

mod worldgen;
mod growth;

fn action(
    label: impl Into<String>,
    act: impl Fn(&mut Panel) -> Result<(), String> + 'static,
) -> Child {
    let label = label.into();
    Box::new(focusable(clickable(
        el("button", text(label.clone())).attr("aria-label", label),
        move |bench: &mut Bench, _| {
            if let Some(p) = &mut bench.founding {
                p.notice = act(p).err().unwrap_or_default();
            }
        },
    )))
}
fn row(items: Vec<Child>) -> Child {
    Box::new(el("div", items).attr("class", "toolbar author-row"))
}
fn words(t: impl Into<String>) -> Child {
    Box::new(el("p", text(t.into())))
}
fn heading(t: impl Into<String>) -> Child {
    Box::new(el("h2", text(t.into())))
}
fn small(t: &str) -> &str {
    t.split_once(':').map_or(t, |(_, name)| name)
}

pub(in crate::app::bench) fn view(p: &Panel) -> Child {
    let mut content: Vec<Child> = vec![
        Box::new(el("h1", text("Found a world"))),
        row(vec![
            action("World conditions", |p| {
                p.page = Page::World;
                Ok(())
            }),
            action("Lineage body maps", |p| {
                p.page = Page::Body;
                Ok(())
            }),
            action("Project founding", Panel::project),
        ]),
    ];
    if p.page != Page::Watch {
        content.push(row(vec![action("Compare edit", Panel::compare_edit)]));
    }
    match p.page {
        Page::World => world(p, &mut content),
        Page::Body => anatomy(p, &mut content),
        Page::Watch => watch(p, &mut content),
    }
    if !p.notice.is_empty() {
        content.push(Box::new(
            el("p", text(p.notice.clone())).attr("role", "status"),
        ));
    }
    Box::new(el("div", content).attr("class", "bench sim authoring"))
}

fn world(p: &Panel, out: &mut Vec<Child>) {
    out.push(heading("World conditions"));
    if p.draft.composed() {
        out.push(row(vec![action("Find a persistent ecology", Panel::search)]));
        if let Some(summary) = p.candidate() {
            out.push(words(summary));
            out.push(row(vec![action("Open suggested world", Panel::open_candidate)]));
        } else {
            for message in p.outlook_words() { out.push(words(message)); }
        }
    }
    worldgen::settings(p,out);
    out.push(row(vec![
        input("World seed", |b| &mut b.founding.as_mut().unwrap().seed),
        input("Starting body population", |b| {
            &mut b.founding.as_mut().unwrap().population
        }),
        input("Founder cohort size", |b| {
            &mut b.founding.as_mut().unwrap().cohort
        }),
        input("Minimum stock", |b| {
            &mut b.founding.as_mut().unwrap().stock_min
        }),
        input("Maximum stock", |b| {
            &mut b.founding.as_mut().unwrap().stock_max
        }),
    ]));
    out.push(row(vec![
        action("Change climate", |p| {
            p.next_preset();
            Ok(())
        }),
        words(p.draft.preset.map_or("Seeded climate".into(), |c| {
            format!("{} · {}", c.name(), c.question())
        })),
    ]));
    if let Some(cosmograph::map::Layout::Grid(grid)) = &p.draft.map {
        out.push(row(vec![
            words(format!(
                "{} · {} sites · {} lineages · sea share {}‰",
                small(&grid.shape),
                p.draft.sites,
                p.draft.lineages,
                grid.sea_per_mille
            )),
            action(
                if grid.shape == "shape:sphere" {
                    "Use a plane"
                } else {
                    "Use a planet"
                },
                |p| {
                    p.change_outline();
                    Ok(())
                },
            ),
            action("Change sea level", |p| {
                if let Some(cosmograph::map::Layout::Grid(g)) = &mut p.draft.map {
                    g.sea_per_mille = if g.sea_per_mille >= 1000 {
                        0
                    } else {
                        (g.sea_per_mille + 250).min(1000)
                    };
                }
                Ok(())
            }),
        ]));
        let millimetres = grid.side as f64 * p.draft.base_unit_micrometres as f64 / 1000.0;
        out.push(row(vec![
            words(format!("Site width {millimetres} mm")),
            action("Double site width", |p| {
                if let Some(cosmograph::map::Layout::Grid(g)) = &mut p.draft.map {
                    g.side = (g.side * 2).min(1 << 20);
                }
                Ok(())
            }),
            action("Halve site width", |p| {
                if let Some(cosmograph::map::Layout::Grid(g)) = &mut p.draft.map {
                    g.side = (g.side / 2).max(16);
                }
                Ok(())
            }),
        ]));
    } else {
        out.push(words(format!(
            "{} sites · {} lineages",
            p.draft.sites, p.draft.lineages
        )));
    }
    out.push(heading("Found from your own template"));
    out.push(row(vec![
        input("Template or save path", |b| {
            &mut b.founding.as_mut().unwrap().path
        }),
        action("Load template", Panel::load),
        action("Reopen projection", Panel::reopen),
    ]));
    out.push(words("Load a Cosmograph founding, edit its lineages, then project the world. Saving a projection also saves its editable template."));
}

fn anatomy(p: &Panel, out: &mut Vec<Child>) {
    out.push(heading("Lineages"));
    let genesis = p.session.sim.genesis();
    let mut lines = Vec::new();
    let mut all:Vec<_>=genesis.lineages.iter().filter_map(|(k,l)|Some((k.strip_prefix("lineage:")?.parse::<u32>().ok()?,k,l))).collect();
    all.sort_by_key(|(i,_,_)|*i);
    let pages=all.len().div_ceil(6);
    if pages>1 {out.push(row(vec![
        action("Previous lineages",move |p|{p.lineage_page=p.lineage_page.saturating_sub(1);Ok(())}),
        words(format!("Lineages · page {} of {pages}",p.lineage_page+1)),
        action("Next lineages",move |p|{p.lineage_page=(p.lineage_page+1).min(pages-1);Ok(())}),
    ]));}
    for (_,key,line) in all.into_iter().skip(p.lineage_page*6).take(6) {
        let key = key.clone();
        lines.push(action(
            format!("{} · {}", key, small(&line.kingdom)),
            move |p| {
                p.lineage.clone_from(&key);
                p.run = 0;
                p.bank_page = 0;
                Ok(())
            },
        ));
    }
    out.push(row(lines));
    if let Some(played) = p.draft.played
        && p.generated_maps.contains_key(&p.lineage)
        && let Some(index) = p.lineage.strip_prefix("lineage:").and_then(|s| s.parse::<u32>().ok())
    {
        if played.lineage == index {
            out.push(words(format!("You can direct members of {} in this founding.", p.lineage)));
        } else {
            out.push(row(vec![
                words(format!("Played lineage · lineage:{}", played.lineage)),
                action("Play this lineage", move |p| {
                    if let Some(played) = &mut p.draft.played { played.lineage = index; }
                    Ok(())
                }),
            ]));
        }
    }
    out.push(row(vec![
        input("Body map path", |b| &mut b.founding.as_mut().unwrap().path),
        action("Load body map", Panel::load_map),
    ]));
    let Some(map) = p.map() else {
        if let Some(line)=genesis.lineages.get(&p.lineage) {
            if line.kingdom=="kingdom:world" || line.kingdom=="kingdom:micro" {
                if let Some(body)=p.habitats.get(&p.lineage) {
                    out.push(words(format!("{} shapes the ground across up to {} borders. Edit its terrain recipe in World conditions.",p.lineage,body.reach)));
                    return;
                }
                let facts=line.traits.iter().map(|s|small(s)).collect::<Vec<_>>().join(", ");
                let habitat=if line.kingdom=="kingdom:world" {"World-class habitat"}else{"Microbial surface"};
                out.push(words(format!("{} · {habitat} · {facts}",p.lineage)));
                return;
            }
        }
        out.push(words(format!("{} uses its generated anatomy.", p.lineage)));
        if genesis.lineages.get(&p.lineage).is_some_and(|l| {
            matches!(
                l.kingdom.as_str(),
                "kingdom:flora" | "kingdom:fauna" | "kingdom:myco"
            )
        }) && p.draft.bodies.is_some()
        {
            out.push(row(vec![action("Create body map", |p| {
                p.add_map();
                Ok(())
            })]));
        }
        return;
    };
    out.push(heading(format!("{} · body map", p.lineage)));
    // What a grown body of this recipe does, read off the body: its
    // kingdom is what it was drawn as and decides none of it.
    if let Some(line) = genesis.lineages.get(&p.lineage)
        && let Some(development) = &line.development
        && let Ok(sheet) = cosmograph::lineage::sheet::of(&genesis.rules, &genesis.lineages, &p.lineage, development)
    {
        use cosmograph::process::FeedingMode;
        let mut does = Vec::new();
        if sheet.functions.get("function:fix").is_some_and(|cells| *cells > 0) {
            does.push("fixes light".to_owned());
        }
        does.extend(match sheet.feeding {
            FeedingMode::Grazer => Some("eats living plants".to_owned()),
            FeedingMode::Predator => Some("eats living flesh".to_owned()),
            FeedingMode::Omnivore => Some("eats living plants and flesh".to_owned()),
            FeedingMode::Scavenger => Some("eats the dead".to_owned()),
            FeedingMode::Producer => None,
        });
        does.extend(sheet.emits.iter().map(|good| format!("gives off {good}")));
        does.extend(sheet.admits.iter().map(|good| format!("takes up {good}")));
        out.push(words(format!("{} · {} mg grown · {}", small(&line.kingdom), sheet.mass, does.join(" · "))));
    }
    for rule in &map.grammar.rules {
        out.push(words(growth::describe(rule)));
    }
    if let Some(graph) = super::diagram::view(p) {
        out.push(graph);
    }
    out.push(words(
        "Select a run to edit it. Drag a branch around its parent to change its growth direction.",
    ));
    let runs = &map.grammar.seed;
    let mut nodes = Vec::new();
    for (i, r) in runs.iter().enumerate() {
        let parent = r.parent.map(usize::from).or_else(|| i.checked_sub(1));
        let attachment = parent.map_or("root".into(), |a| {
            format!("{:?} of run {}", r.facing, a + 1)
        });
        let tissue = r
            .bears
            .as_ref()
            .map(|k| map.regions.get(k).unwrap_or(k))
            .map_or(String::new(), |k| format!(" + {}", small(k)));
        nodes.push(action(
            format!(
                "Run {} · {}{} · {attachment}",
                i + 1,
                small(&r.segment),
                tissue
            ),
            move |p| {
                p.run = i;
                p.bank_page = 0;
                Ok(())
            },
        ));
    }
    out.push(row(nodes));
    let run = &runs[p.run];
    let repeat = map
        .grammar
        .rules
        .iter()
        .find_map(|r| match r {
            cosmograph::rules::Rule::Repeat { tagma, by } if usize::from(*tagma) == p.run => {
                Some(*by)
            },
            _ => None,
        })
        .unwrap_or(0);
    out.push(heading(format!("Run {} · development", p.run + 1)));
    out.push(words(format!("{} seed segments · {repeat} added by repeat · segment variation ±{} · borne tissue faces {:?}",
        run.segments, run.variance.unwrap_or(map.variance), run.socket)));
    out.push(row(vec![
        action("Add branch here", |p| {
            let i = p.run;
            p.run = p.map_mut().branch_from(i)?;
            Ok(())
        }),
        action("Increase repeat", |p| {
            p.repeat(true);
            Ok(())
        }),
        action("Decrease repeat", |p| {
            p.repeat(false);
            Ok(())
        }),
        action("Change variation", |p| {
            let i = p.run;
            let map = p.map_mut();
            let v = map.grammar.seed[i].variance.unwrap_or(map.variance);
            map.grammar.seed[i].variance = Some(if v >= 3 { 0 } else { v + 1 });
            Ok(())
        }),
        action("Turn tissue socket", |p| {
            let i = p.run;
            let r = &mut p.map_mut().grammar.seed[i];
            r.socket = next_facing(r.socket);
            Ok(())
        }),
    ]));
    let odds = map.absence;
    out.push(row(vec![
        words(format!(
            "Optional appendages omitted on {} of {} draws",
            odds[0], odds[1]
        )),
        action("Change presence odds", |p| {
            let map = p.map_mut();
            map.absence = match map.absence {
                [0, 1] => [1, 4],
                [1, 4] => [1, 2],
                [1, 2] => [3, 4],
                [3, 4] => [1, 1],
                _ => [0, 1],
            };
            Ok(())
        }),
        action("Turn branch", |p| {
            let i = p.run;
            if i == 0 {
                return Err("Choose a branch attached to the root.".into());
            }
            let r = &mut p.map_mut().grammar.seed[i];
            r.facing = next_facing(r.facing);
            Ok(())
        }),
    ]));
    out.push(heading("Tissue and traits"));
    let chosen = run
        .bears
        .as_deref()
        .map(|k| map.regions.get(k).map_or(k, String::as_str));
    if let Some(k) = chosen {
        let info = cosmograph::catalog::catalog().part_by_key(k);
        out.push(words(info.map_or_else(
            || format!("Attached tissue · {}", small(k)),
            |part| {
                format!(
                    "{} · {} mg · {} · {}",
                    small(k),
                    part.cost_mg,
                    small(&part.process),
                    part.traits.join(", ")
                )
            },
        )));
    }
    let mut bank: Vec<String> = cosmograph::bodied::default_kinds()
        .into_keys()
        .filter(|key| genesis.rules.kinds.contains_key(key))
        .collect();
    if let (Some(rep), Some(line)) = (&genesis.rules.repertoire, genesis.lineages.get(&p.lineage)) {
        bank.extend(
            rep.kinds
                .iter()
                .filter(|(kind, entry)| entry.root_of.is_none() && rep.reaches(&line.kingdom, kind))
                .map(|(_, entry)| entry.part.clone()),
        );
    }
    let last = bank.len().saturating_sub(1) / 8;
    let page = p.bank_page.min(last);
    let mut choices: Vec<_> = bank
        .into_iter()
        .skip(page * 8)
        .take(8)
        .map(|k| {
            action(format!("Attach {}", small(&k)), move |p| {
                p.bind(k.clone());
                Ok(())
            })
        })
        .collect();
    if page > 0 {
        choices.push(action("Previous tissues", move |p| {
            p.bank_page = page - 1;
            Ok(())
        }));
    }
    if page < last {
        choices.push(action("More tissues", move |p| {
            p.bank_page = page + 1;
            Ok(())
        }));
    }
    out.push(row(choices));
}

fn next_facing(facing: Facing) -> Facing {
    match facing {
        Facing::Above => Facing::Right,
        Facing::Right => Facing::Below,
        Facing::Below => Facing::Left,
        Facing::Left => Facing::Front,
        Facing::Front => Facing::Back,
        Facing::Back => Facing::Above,
    }
}

fn watch(p: &Panel, out: &mut Vec<Child>) {
    let state = p.session.sim.state();
    let rules = &p.session.sim.genesis().rules;
    out.push(heading(format!(
        "Seed {} · tick {}",
        p.session.sim.genesis().seed,
        state.tick
    )));
    let move_time = |label, delta: i64| {
        action(label, move |p| {
            p.seek(p.session.sim.state().tick.saturating_add_signed(delta))
        })
    };
    out.push(row(vec![
        action(
            if p.playing() {
                "Pause time"
            } else {
                "Watch time pass"
            },
            |p| {
                p.next_tick = if p.playing() {
                    None
                } else {
                    Some(std::time::Instant::now())
                };
                Ok(())
            },
        ),
        move_time("Back 12 ticks", -12),
        move_time("Back one tick", -1),
        move_time("Advance one tick", 1),
        move_time("Advance 12 ticks", 12),
        button(
            if p.original.is_some() {
                "Save edited"
            } else {
                "Save projection"
            },
            |b| {
                if let Err(e) = b.store_projection(Version::Edited, false) {
                    b.founding.as_mut().unwrap().notice = e;
                }
            },
        ),
        button(
            if p.original.is_some() {
                "Enter edited"
            } else {
                "Enter this world"
            },
            |b| {
                if let Err(e) = b.store_projection(Version::Edited, true) {
                    b.founding.as_mut().unwrap().notice = e;
                }
            },
        ),
    ]));
    worldgen::atlas(p,out);
    let sites: Vec<_> = state.sites.keys().copied().collect();
    let index = sites.iter().position(|s| *s == p.site).unwrap_or(0);
    let previous = sites[(index + sites.len() - 1) % sites.len()];
    let next = sites[(index + 1) % sites.len()];
    out.push(row(vec![
        action("Previous site", move |p| {
            p.site = previous;
            p.specimen_id = None;
            p.part_id = None;
            Ok(())
        }),
        heading(format!("Site {}", p.site)),
        action("Next site", move |p| {
            p.site = next;
            p.specimen_id = None;
            p.part_id = None;
            Ok(())
        }),
    ]));
    let mut forecast = vec![action("Forecast this ecology", Panel::forecast), action("Visit selected lineage", Panel::visit_lineage), words(format!("Selected {}", p.lineage))];
    if let Some((_, specimen)) = p.specimen()
        && specimen.lineage != p.lineage
    {
        let lineage = specimen.lineage.clone();
        forecast.push(action(format!("Select {}", lineage), move |p| {
            p.lineage.clone_from(&lineage);
            p.run = 0;
            Ok(())
        }));
    }
    out.push(row(forecast));
    for line in p.outlook_words() {
        out.push(words(line));
    }
    if let Some(original) = &p.original {
        super::comparison::display(original, &p.session, p.site, out);
        out.push(row(vec![
            button("Save original", |b| {
                if let Err(e) = b.store_projection(Version::Original, false) {
                    b.founding.as_mut().unwrap().notice = e;
                }
            }),
            button("Enter original", |b| {
                if let Err(e) = b.store_projection(Version::Original, true) {
                    b.founding.as_mut().unwrap().notice = e;
                }
            }),
        ]));
    }
    if let Some(site) = state.sites.get(&p.site) {
        if let Some(skel)=&p.session.sim.genesis().rules.skeleton {
            out.push(words(format!("Ground elevation {} · relief {} · water level {}",
                site.conditions[&skel.elevation],site.conditions[&skel.relief],site.conditions[&skel.water])));
        }
        for (k,n) in &site.conditions {
            if let Some(id)=k.strip_prefix("habitat:body-") && let Some(body)=p.habitats.get(&format!("lineage:{id}")) {
                let [elevation,relief,water]=body.contribution(*n);
                out.push(words(format!("World-class lineage {id} here · elevation {elevation:+} · relief {relief:+} · water level {water:+}")));
            }
        }
        for (k,n) in &site.accounts {
            if state.spreads.identities.contains(&cosmograph::spread::Identity::Surface(k.clone())) {
                out.push(words(format!("Microbial surface {k} · {n} mg here")));
            }
        }
        out.push(words(format!(
            "Soil {} · light {}",
            site.accounts.get("world:soil").unwrap_or(&0),
            site.accounts.get("sim:energy").unwrap_or(&0)
        )));
    }
    if let Some((id, body)) = p.specimen() {
        out.push(row(vec![
            heading(format!(
                "Specimen #{id} · {} · {} · {}",
                body.lineage,
                if body.alive { "living" } else { "dead" },
                body.resident_site()
                    .map_or_else(|| "travelling".into(), |s| format!("site {s}"))
            )),
            action("Next specimen", |p| {
                p.next_specimen();
                Ok(())
            }),
        ]));
        if let Some(parent)=state.relations.iter().find(|r|r.subject==id && r.kind=="sim:parent") {
            out.push(words(format!("Born at tick {} · parent #{}",body.born,parent.object)));
        }
        if let Some(graph) = super::diagram::view(p) {
            out.push(graph);
        }
        let mut processes = std::collections::BTreeMap::new();
        for flow in &p.flows {
            let cosmograph::flows::MadeBy::Process(key) = &flow.made_by else {
                continue;
            };
            // The tissue in passage is the one leg that names both partners.
            if flow.from.1 == cosmograph::ecology::IN_PASSAGE && flow.to.1 == flow.from.1
                && !flow.internal()
                && let (Some(payer), Some(drawer)) = (flow.from.0.body(), flow.to.0.body())
                && (id == payer || id == drawer)
            {
                let bodies = (state.population.get(payer), state.population.get(drawer));
                let terms = match bodies {
                    (Some(a), Some(b)) => cosmograph::ecology::bargain(rules, a, b),
                    _ => None,
                };
                let [paid, drawn] = terms.map_or_else(
                    || ["tissue".to_owned(), "what the ground holds".to_owned()],
                    |legs| legs.map(|goods| goods.join(" or ")),
                );
                out.push(words(format!(
                    "#{payer} paid #{drawer} {} mg in {paid} for {drawn} at tick {}",
                    flow.amount, flow.tick,
                )));
            }
            for (end, direction) in [(flow.from.0, "out"), (flow.to.0, "in")] {
                if end
                    .body()
                    .is_some_and(|first| id >= first && id - first < flow.count)
                {
                    *processes.entry((key, direction)).or_insert(0u128) += u128::from(flow.amount);
                }
            }
        }
        for ((key, direction), amount) in processes {
            out.push(words(format!("#{id} · {key} · {amount} mg {direction}")));
        }
        // Who this body could trade with where it stands, read from the
        // organs each has now; one line for each other lineage.
        let mut partners = std::collections::BTreeMap::new();
        for group in state.population.groups.values() {
            let other = &group.entity;
            if !other.alive || other.lineage == body.lineage || other.resident_site() != Some(p.site) {
                continue;
            }
            if let Some([paid, drawn]) = cosmograph::ecology::bargain(rules, body, other) {
                partners.entry((&other.lineage, true)).or_insert_with(|| format!(
                    "Could pay {} in {} for {}", other.lineage, paid.join(" or "), drawn.join(" or ")));
            }
            if let Some([paid, drawn]) = cosmograph::ecology::bargain(rules, other, body) {
                partners.entry((&other.lineage, false)).or_insert_with(|| format!(
                    "Could draw {} for {}, paid in {}", drawn.join(" or "), other.lineage, paid.join(" or ")));
            }
        }
        for line in partners.into_values() {
            out.push(words(line));
        }
        if let Some(id) = p.part_id.and_then(|n| u32::try_from(n).ok())
            && let Some(part) = body.parts.get(&cosmograph::schema::PartId(id))
        {
            let cells = part
                .cells
                .iter()
                .map(|(k, v)| format!("{v} {}", small(k)))
                .collect::<Vec<_>>();
            out.push(words(format!("Part {id} · {}", cells.join(", "))));
            let part_id=cosmograph::schema::PartId(id);
            if let Some([run,seg,slot])=body.situs(part_id)
                && part.graft_origin.is_none()
                && let Some(dev)=state.lineages.get(&body.lineage).and_then(|l|l.development.as_ref())
                && let Some(run)=dev.recipe.tagmata.get(usize::from(run))
                && (slot==0 || run.chain.is_empty())
                && let Some(kind)=run.kind_at(seg,slot)
                && let Some(bank)=&p.session.sim.genesis().rules.repertoire {
                let suffix=format!("-port-{}",body.lineage.trim_start_matches("lineage:"));
                if let Some(entry)=bank.kinds.get(kind.strip_suffix(&suffix).unwrap_or(&kind))
                    && let Some(named)=cosmograph::catalog::catalog().part_by_key(&entry.part) {
                    out.push(words(format!("Lineage recipe · {} · {}",small(&named.key),named.traits.join(", "))));
                }
            }
            if body.lives(part_id)
                && let Some(port)=cosmograph::mosaic::ports::active(part) {
                let goods=rules.repertoire.as_ref().map_or(&[][..],|r|&r.goods);
                let abilities=super::organ_reading(port,goods);
                if !abilities.is_empty(){out.push(words(abilities.join(" · ")));}
            }
        }
    }
    let mut forms = std::collections::BTreeMap::new();
    for group in state.population.groups.values() {
        let e = &group.entity;
        if e.resident_site() != Some(p.site) || !e.lineage.starts_with("lineage:") {
            continue;
        }
        let counts = forms
            .entry((&e.lineage, &e.soma, &e.absent))
            .or_insert([0u64; 3]);
        if e.alive {
            counts[0] += group.count;
            if e.born > 0 {
                counts[1] += group.count;
            }
        } else {
            counts[2] += group.count;
        }
    }
    out.push(heading("Inhabitants"));
    for ((line, runs, absent), [living, young, dead]) in forms {
        let build = if runs.is_empty() {
            "unmapped".into()
        } else {
            let lengths = runs.iter().map(u8::to_string).collect::<Vec<_>>().join("/");
            let missing = absent
                .iter()
                .map(|(run, seg)| format!("{}.{}", u16::from(*run) + 1, u16::from(*seg) + 1))
                .collect::<Vec<_>>()
                .join(", ");
            if missing.is_empty() {
                format!("segments {lengths}")
            } else {
                format!("segments {lengths}, absent organs {missing}")
            }
        };
        out.push(words(format!("{line} · {build} · {living} living · {young} born since founding and alive · {dead} dead here")));
    }
    if state.tick > 0 {
        out.push(heading("Matter moved this tick"));
        match p.transfers.get(&p.site) {
            Some(moves) => {
                for (process, amount) in moves {
                    out.push(words(format!("{process} · {amount} mg")));
                }
            },
            None => out.push(words("No matter moved at this site this tick.")),
        }
    }
}
