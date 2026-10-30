use super::*;
use crate::native::GameBindings;
use isocosm::{operations::Window, spatial::Pose};

struct PhysicalPoint {
    pose: Pose,
    window: Window,
    places: isometer_space::places::Rules,
}

fn physical_point(
    session: &Session,
    game: &GameBindings,
    id: u64,
) -> Result<PhysicalPoint, String> {
    if let Some(resident) = session.sim.state().spatial_poses.get(&id) {
        let body = session
            .sim
            .state()
            .population
            .get(id)
            .ok_or("body is absent")?;
        if body.resident_site() != Some(resident.site)
            || body.arrived != resident.arrived
            || body.patch.is_none()
        {
            return Err("native pose belongs to a different residence".into());
        }
        let template = session
            .sim
            .genesis()
            .rules
            .spatial_templates
            .as_ref()
            .ok_or("native pose has no spatial declaration")?;
        return Ok(PhysicalPoint {
            pose: resident.pose.clone(),
            window: isocosm::spatial::embodiment::site_window(template, resident.pose.at),
            places: template.places,
        });
    }
    let position = game
        .positions()
        .values()
        .find(|p| p.entity == id)
        .ok_or("body has no native physical placement")?;
    let pose = position.current_pose(session)?;
    let window = &position.placement.window;
    Ok(PhysicalPoint {
        pose: Pose {
            at: pose.at,
            yaw: pose.yaw,
            ..Default::default()
        },
        window: Window {
            min: window.min,
            max: window.max,
            depth: window.depth,
        },
        places: position.placement.places,
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReachRuling {
    pub squares: u32,
    pub reason: String,
}

pub(super) fn reach(
    state: &State,
    session: &Session,
    sections: &BTreeMap<String, Vec<u8>>,
    access: OrdinaryAccess,
    actor: u64,
    target: u64,
    payload: &[u8],
) -> Result<(u32, String), String> {
    let a = session
        .sim
        .state()
        .population
        .get(actor)
        .ok_or("attacker is absent")?;
    let b = session
        .sim
        .state()
        .population
        .get(target)
        .ok_or("target is absent")?;
    if !a.alive || !b.alive {
        return Err("native bodies must be alive".into());
    }
    if a.resident_site().is_none() || a.resident_site() != b.resident_site() {
        return Err("attack requires bodies at the same native site".into());
    }
    if let Some(scale) = state.cells_per_square {
        let game = GameBindings::decode(sections.get("vtt").map_or(&[], Vec::as_slice))?;
        game.validate(session)?;
        let source = physical_point(session, &game, actor)?;
        let target = physical_point(session, &game, target)?;
        let start = source.pose.at;
        let end = target.pose.at;
        let span = (0..3)
            .map(|axis| start[axis].abs_diff(end[axis]))
            .max()
            .unwrap_or(0);
        if span > u64::from(scale) {
            return Err("target is beyond the weapon's five-foot reach".into());
        }
        let window = Window {
            min: [
                source.window.min[0].min(target.window.min[0]),
                source.window.min[1].min(target.window.min[1]),
            ],
            max: [
                source.window.max[0].max(target.window.max[0]),
                source.window.max[1].max(target.window.max[1]),
            ],
            depth: source.window.depth.max(target.window.depth),
        };
        let geometry =
            session
                .sim
                .site_geometry(a.resident_site().unwrap(), &window, source.places)?;
        let ray_range = (0..3)
            .try_fold(0u64, |sum, axis| {
                sum.checked_add(start[axis].abs_diff(end[axis]))
            })
            .ok_or("physical ray overflows")?;
        if !geometry
            .places
            .sees(&geometry.volume, start, end, ray_range.max(1))
        {
            return Err("native terrain blocks the weapon attack".into());
        }
        return Ok((
            u32::try_from(span.div_ceil(u64::from(scale))).map_err(|_| "distance overflow")?,
            format!("native root reach: {span} cells, scale {scale} cells per square"),
        ));
    }
    if access.seat != Seat::Dm {
        return Err("uncalibrated reach needs the DM's recorded ruling".into());
    }
    let ruling: ReachRuling = serde_json::from_slice(payload)
        .map_err(|_| "enter squares and a reason for the reach ruling")?;
    if ruling.reason.trim().is_empty() {
        return Err("reach ruling needs its reason".into());
    }
    Ok((
        ruling.squares,
        format!(
            "DM reach ruling: {} squares; {}",
            ruling.squares, ruling.reason
        ),
    ))
}

pub(super) fn authorize(
    session: &Session,
    access: OrdinaryAccess,
    actor: u64,
) -> Result<TokenId, String> {
    if !controls(session, access, actor) {
        return Err("this seat does not control the living native body".into());
    }
    token(actor)
}

pub(super) fn dm(access: OrdinaryAccess) -> Result<(), String> {
    if access.seat != Seat::Dm {
        Err("this edit requires the DM seat".into())
    } else {
        Ok(())
    }
}

pub(super) fn unmarked(
    sections: &BTreeMap<String, Vec<u8>>,
    inventory: &Inventory,
    item: &ItemId,
) -> Result<(), String> {
    let game = GameBindings::decode(sections.get("vtt").map_or(&[], Vec::as_slice))?;
    if !crate::native::items::unmarked_inventory(&game, inventory)
        .items
        .contains_key(item)
    {
        return Err("a native physical item requires its own edition binding; do not count it again as table equipment".into());
    }
    Ok(())
}

pub(super) fn hurt(
    state: &mut State,
    map: &mut MapDocument,
    session: &Session,
    target: TokenId,
    amount: u32,
    rolls: &mut Vec<RollRecord>,
) -> Result<String, String> {
    let sheet = map
        .sheets
        .get_mut(&target)
        .ok_or("damage target has no edition sheet")?;
    let hp = sheet.int("hp_current").ok_or("HP is undeclared")?;
    sheet.set_int("hp_current", hp.saturating_sub(i64::from(amount)).max(0));
    if let Some(check) =
        effects::concentration_check(map, target, amount, clock(session), &mut state.rng)?
    {
        rolls.push(RollRecord {
            by: format!("{} concentration", target.0),
            expr: format!("Constitution save {:+} vs DC {}", check.bonus, check.dc),
            dice: check.dice,
            total: i32::try_from(check.total)
                .map_err(|_| "concentration total overflows roll record")?,
        });
        Ok(format!("; concentration {} vs {}", check.total, check.dc))
    } else {
        Ok(String::new())
    }
}
