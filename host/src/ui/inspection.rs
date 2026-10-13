// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Local presentation selection, read afresh through the owned native window.

use cambium::{el, text};
use isocosm::{attention::spatial::{Crowd, InspectedBody, RegionReading}, schema::PartId};
use isomere::{ExaminerModel, ExaminerRow};
use serde::{Deserialize, Serialize};
use super::{model::{App, Defaults}, view::{Child, button}};

/// A saved view preference. These IDs supply no awareness, watch or control.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Selection {
    pub site: u64, pub body: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part: Option<PartId>,
}

struct Reading {
    far: Vec<Crowd>, region: Option<RegionReading>,
    selected: Option<InspectedBody>, selection: Option<Selection>, notice: Option<String>,
}

fn read(app: &App, key: u64) -> Result<Reading, String> {
    let local = app.windows.get(&key).ok_or("window is closed")?;
    let cut = local.native.projection()?;
    let window = local.native.handle()?;
    let defaults: Defaults = serde_json::from_slice(&local.native.binding().view_defaults)
        .map_err(|e| e.to_string())?;
    let examined = local.native.binding().attention.examined.map(|site| site.0);
    app.world.read(|session| {
        let far = crate::spatial::far(session, &cut, window)?;
        let mut notice = None;
        let region = match examined {
            Some(site) => match crate::spatial::region(session, &cut, window, site) {
                Ok(region) => Some(region),
                Err(why) => { notice = Some(why); None },
            },
            None => None,
        };
        let mut selected = None;
        if let Some(selection) = &defaults.inspection {
            if examined != Some(selection.site) {
                notice = Some("The saved selection is outside this window’s current close-up.".into());
            } else {
                match crate::spatial::inspect(session, &cut, window, selection.site, selection.body) {
                    Ok(body) => selected = Some(body),
                    Err(why) => notice = Some(why),
                }
            }
        }
        Ok(Reading { far, region, selected, selection: defaults.inspection, notice })
    })?
}

impl App {
    fn select_inspection(&mut self, key: u64, selection: Option<Selection>) -> Result<(), String> {
        let local = self.windows.get(&key).ok_or("window is closed")?;
        let cut = local.native.projection()?;
        let window = local.native.handle()?;
        if let Some(selection) = &selection {
            self.world.read(|session| {
                let body = crate::spatial::inspect(session, &cut, window, selection.site, selection.body)?;
                if selection.part.is_some_and(|id| body.body.document.part(id).is_none()) {
                    return Err("selected part is outside the current native body document".into());
                }
                Ok::<(), String>(())
            })??;
        }
        let local = self.windows.get_mut(&key).ok_or("window is closed")?;
        let mut defaults: Defaults = serde_json::from_slice(&local.native.binding().view_defaults)
            .map_err(|e| e.to_string())?;
        defaults.inspection = selection;
        let mode = local.native.binding().mode.clone();
        local.native.set_view_defaults(serde_json::to_vec(&defaults).map_err(|e| e.to_string())?)?;
        local.preferences.modes.insert(mode, defaults);
        self.refresh();
        Ok(())
    }
}

pub(super) fn panel(app: &App, key: u64) -> Child {
    let mut rows: Vec<Child> = vec![Box::new(el("h2", text("Native spatial readings")))];
    let reading = match read(app, key) {
        Ok(reading) => reading,
        Err(why) => {
            rows.push(Box::new(el("p", text(why)).attr("role", "status")));
            return Box::new(el("section", rows).attr("class", "panel"));
        },
    };
    rows.push(Box::new(el("h3", text("Far crowds"))));
    if reading.far.is_empty() {
        rows.push(Box::new(el("p", text("No crowds are visible through this window."))));
    }
    for crowd in &reading.far {
        rows.push(Box::new(el("p", text(format!("Site {} · {} · {} living members",
            crowd.site, crowd.lineage, crowd.count)))));
    }
    rows.push(Box::new(el("h3", text("This window’s close-up"))));
    if let Some(region) = &reading.region {
        rows.push(Box::new(el("p", text(format!("Site {}", region.site)))));
        if region.bodies.is_empty() {
            rows.push(Box::new(el("p", text("No native bodies are visible in this region."))));
        }
        for body in &region.bodies {
            let selection = Selection { site: region.site, body: body.id, part: None };
            rows.push(button(format!("inspect-body-{}", body.id), format!("Inspect body {} · {} · {}",
                body.id, body.lineage, if body.alive { "living" } else { "remains" }), key,
                move |app| app.select_inspection(key, Some(selection.clone()))));
        }
    } else {
        let notice = if reading.notice.is_some() { "The requested close-up is unavailable." }
            else { "Choose an Examine site control to open a native region." };
        rows.push(Box::new(el("p", text(notice))));
    }
    if reading.selection.is_some() {
        rows.push(button("clear-inspection".into(), "Clear body selection".into(), key,
            move |app| app.select_inspection(key, None)));
    }
    if let Some(notice) = reading.notice {
        rows.push(Box::new(el("p", text(notice)).attr("role", "status")));
    }
    if let (Some(body), Some(selection)) = (reading.selected, reading.selection) {
        rows.push(examiner(body, selection, key));
    }
    Box::new(el("section", rows).attr("class", "panel"))
}

fn examiner(inspected: InspectedBody, selection: Selection, key: u64) -> Child {
    let body = &inspected.body;
    let mut model = ExaminerModel::<App, ()>::new("Native body parts");
    model.class = Some("panel");
    model.readings = vec![("Body".into(), body.id.to_string()),
        ("Lineage".into(), body.lineage.clone()), ("Kingdom".into(), body.kingdom.clone()),
        ("State".into(), if body.alive { "living" } else { "remains" }.into()),
        ("Body revision".into(), body.revision.to_string())];
    for geometry in &body.document.parts {
        let mut row = ExaminerRow::new(u64::from(geometry.id.0),
            format!("Part {} · {}", geometry.id.0, geometry.shape));
        row.selected = selection.part == Some(geometry.id);
        row.state_class = geometry.severed.then_some("severed");
        model.rows.push(row);
    }
    if let Some(id) = selection.part {
        if let Some(geometry) = body.document.part(id) {
            model.readings.extend([("Part".into(), id.0.to_string()),
                ("Shape".into(), geometry.shape.clone()),
                ("Attachment".into(), geometry.attachment.as_ref().map_or("root".into(),
                    |at| format!("Part {} · offset {:?} · yaw {:?}", at.parent.0, at.offset, at.yaw))),
                ("Half extent".into(), format!("{:?}", geometry.half_extent)),
                ("Part state".into(), if geometry.severed { "severed" } else { "present" }.into())]);
            if let Some(part) = inspected.parts.get(&id) {
                model.readings.extend([("Traits".into(), part.traits.iter().cloned().collect::<Vec<_>>().join(", ")),
                    ("Functions".into(), part.functions.iter().cloned().collect::<Vec<_>>().join(", ")),
                    ("Allocated cells".into(), part.cells.iter().map(|(name, n)| format!("{name}: {n}")).collect::<Vec<_>>().join(", ")),
                    ("Part stocks".into(), part.matter.iter().map(|(name, n)| format!("{name}: {n}")).collect::<Vec<_>>().join(", ")),
                    ("Lost cells".into(), part.lost.len().to_string())]);
            } else { model.note = Some("This document part has no current native physiology record."); }
        } else { model.note = Some("The saved selected part is absent from the current native body document."); }
    } else { model.note = Some("Choose a part to read its current geometry and stocks."); }
    isomere::examiner(model, move |app: &mut App, id| {
        let result = u32::try_from(id).map_err(|_| "selected native part identity exceeds its bounds".into())
            .and_then(|id| app.select_inspection(key, Some(Selection { part: Some(PartId(id)), ..selection.clone() })));
        app.record(key, result);
    })
}
