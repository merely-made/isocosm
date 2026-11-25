// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

pub(super) fn settings(p: &Panel, out: &mut Vec<Child>) {
    if let Some(dist) = p
        .draft
        .repertoire
        .as_ref()
        .and_then(|r| r.kingdom_draw.as_ref())
    {
        out.push(row(vec![
            input("Minimum kingdoms", |b| {
                &mut b.founding.as_mut().unwrap().kingdom_min
            }),
            input("Maximum kingdoms", |b| {
                &mut b.founding.as_mut().unwrap().kingdom_max
            }),
            input("Minimum lineages per kingdom", |b| {
                &mut b.founding.as_mut().unwrap().richness_min
            }),
            input("Maximum lineages per kingdom", |b| {
                &mut b.founding.as_mut().unwrap().richness_max
            }),
        ]));
        let mut pool = Vec::new();
        for k in cosmograph::generate::FoundingForm::ALL {
            let enabled = dist.pool.contains(k.key());
            pool.push(action(
                format!(
                    "{} {}",
                    if enabled { "Include ✓" } else { "Include" },
                    k.name()
                ),
                move |p| {
                    let dist = p
                        .draft
                        .repertoire
                        .as_mut()
                        .and_then(|r| r.kingdom_draw.as_mut())
                        .ok_or("No kingdom draw")?;
                    let previous = dist.pool.len() as u64;
                    if dist.pool.len() == 1 && dist.pool.contains(k.key()) {
                        return Err("Choose at least one eligible kingdom.".into());
                    }
                    if !dist.pool.remove(k.key()) {
                        dist.pool.insert(k.key().to_owned());
                    }
                    let size = dist.pool.len() as u64;
                    if let Ok(max) = p.kingdom_max.text().parse::<u64>() {
                        if max > size || max == previous {
                            p.kingdom_max = super::super::field(size);
                        }
                    }
                    if p.kingdom_min
                        .text()
                        .parse::<u64>()
                        .is_ok_and(|min| min > size)
                    {
                        p.kingdom_min = super::super::field(size);
                    }
                    Ok(())
                },
            ));
        }
        out.push(row(pool));
        match p.recipe().and_then(|f| {
            f.repertoire
                .as_ref()
                .unwrap()
                .kingdom_draw
                .as_ref()
                .unwrap()
                .draw(f.seed)
        }) {
            Ok(roll) => out.push(words(
                roll.iter()
                    .map(|(k, n)| format!("{n} {}", k.name()))
                    .collect::<Vec<_>>()
                    .join(" · "),
            )),
            Err(why) => out.push(words(why)),
        }
        world_bodies(p, out);
    }
}

fn world_bodies(p: &Panel, out: &mut Vec<Child>) {
    let Ok(draft) = p.recipe() else { return };
    let Ok(maps) = draft.world_body_recipes() else {
        return;
    };
    if maps.is_empty() {
        return;
    }
    let mut maps: Vec<_> = maps.into_iter().collect();
    maps.sort_by_key(|(k, _)| {
        k.trim_start_matches("lineage:")
            .parse::<u32>()
            .unwrap_or_default()
    });
    let slot = p.world_body_slot.min(maps.len() - 1);
    let (key, body) = &maps[slot];
    let body = *body;
    let last = maps.len() - 1;
    out.push(heading("World-class habitats"));
    out.push(row(vec![
        action("Previous habitat", move |p| {
            p.world_body_slot = slot.saturating_sub(1);
            Ok(())
        }),
        words(format!(
            "{key} · reach {} borders · elevation {:+} · relief {:+} · water level {:+}",
            body.reach, body.elevation, body.relief, body.water
        )),
        action("Next habitat", move |p| {
            p.world_body_slot = (slot + 1).min(last);
            Ok(())
        }),
    ]));
    let mut edits = Vec::new();
    for (label, changed) in [
        (
            "Extend habitat",
            cosmograph::generate::WorldBody {
                reach: (body.reach + 1).min(32),
                ..body
            },
        ),
        (
            "Shrink habitat",
            cosmograph::generate::WorldBody {
                reach: body.reach.saturating_sub(1),
                ..body
            },
        ),
        (
            "Raise ground",
            cosmograph::generate::WorldBody {
                elevation: (body.elevation + 1).min(64),
                ..body
            },
        ),
        (
            "Lower ground",
            cosmograph::generate::WorldBody {
                elevation: (body.elevation - 1).max(-64),
                ..body
            },
        ),
        (
            "Roughen",
            cosmograph::generate::WorldBody {
                relief: (body.relief + 1).min(64),
                ..body
            },
        ),
        (
            "Smooth",
            cosmograph::generate::WorldBody {
                relief: body.relief.saturating_sub(1),
                ..body
            },
        ),
        (
            "Raise water",
            cosmograph::generate::WorldBody {
                water: (body.water + 1).min(64),
                ..body
            },
        ),
        (
            "Lower water",
            cosmograph::generate::WorldBody {
                water: (body.water - 1).max(-64),
                ..body
            },
        ),
    ] {
        edits.push(action(label, move |p| {
            p.draft
                .repertoire
                .as_mut()
                .ok_or("No repertoire")?
                .world_bodies
                .insert(slot.to_string(), changed);
            Ok(())
        }));
    }
    edits.push(action("Restore generated habitat", move |p| {
        p.draft
            .repertoire
            .as_mut()
            .ok_or("No repertoire")?
            .world_bodies
            .remove(&slot.to_string());
        Ok(())
    }));
    out.push(row(edits));
}

/// The projected world's actual skeleton. On a planet the rows are the
/// native six-face atlas. Selecting a cell moves only the observer.
pub(super) fn atlas(p: &Panel, out: &mut Vec<Child>) {
    let g = p.session.sim.genesis();
    let Some(cosmograph::map::Layout::Grid(grid)) =
        g.founding.as_ref().and_then(|f| f.map.as_ref())
    else {
        return;
    };
    let Some(skel) = &g.rules.skeleton else {
        return;
    };
    out.push(action(
        if p.show_atlas {
            "Hide site map"
        } else {
            "Show site map"
        },
        |p| {
            p.show_atlas = !p.show_atlas;
            Ok(())
        },
    ));
    if !p.show_atlas {
        return;
    }
    let sites = &p.session.sim.state().sites;
    let peak = sites
        .values()
        .map(|s| s.conditions[&skel.elevation])
        .max()
        .unwrap_or(0);
    let low = sites
        .values()
        .map(|s| s.conditions[&skel.elevation])
        .min()
        .unwrap_or(0);
    for y in 0..grid.height {
        let mut cells: Vec<Child> = Vec::new();
        for x in 0..grid.width {
            let id = u64::from(y) * u64::from(grid.width) + u64::from(x);
            let Some(site) = sites.get(&id) else { continue };
            let elevation = site.conditions[&skel.elevation];
            let water = site.conditions[&skel.water];
            let (kind, color) = if elevation < water {
                ("water", "#69a8c4")
            } else if 3 * (elevation - low) > 2 * (peak - low) {
                ("high ground", "#cfb895")
            } else {
                ("low ground", "#93b28a")
            };
            let label = format!("Site {id}, {kind}, elevation {elevation}, water level {water}");
            cells.push(Box::new(focusable(clickable(
                el("button", text(id.to_string()))
                    .attr("aria-label", label)
                    .attr("style", format!("background-color:{color};color:#172422")),
                move |bench: &mut Bench, _| {
                    if let Some(p) = &mut bench.founding {
                        p.site = id;
                        p.specimen_id = None;
                        p.part_id = None;
                    }
                },
            ))));
        }
        out.push(row(cells));
    }
}
