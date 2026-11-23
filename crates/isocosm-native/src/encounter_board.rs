// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A local table picture over admitted native positions. A player's live
//! terrain comes from its current eye; legacy positions without an eye show
//! the movement destinations the controlling seat's sheet already discloses.
use super::{Act, Built, Cx, sheets_pf2e};
use crate::native::{GameBindings, PositionBinding, eye};
use crate::ruleset::{self, Action, State, pathfinder::play};
use chirograph::{AdvertisedAction, IntentEffect};
use cosmograph::Session;
use cosmograph_host::endpoint::disclosure::{Item, NULL, action, card, value};
use cosmograph_overlay::Seat;
use isocosm_core::encounter_board::{BADGE, BoardCell, BoardToken, EncounterBoard, VALUE};
use isocosm_system::pf2e::{combat, records};
use std::collections::BTreeMap;

struct Board {
    site: u64,
    spacing: u32,
    cells: BTreeMap<[i64; 3], Option<String>>,
    tokens: Vec<BoardToken>,
    actions: Vec<AdvertisedAction>,
    acts: Vec<Act>,
}

impl Board {
    fn offer(&mut self, id: &str, label: &str, why: &str, act: Action) {
        self.actions
            .push(action(id, label, why, NULL, IntentEffect::DomainTruth));
        self.acts.push(Act::Ruleset(act));
    }

    fn stride(&mut self, actor: u64, at: [i64; 3], ruled: bool) {
        let [x, y, z] = at;
        let id = format!("vtt.pf2e-board-stride-{actor}-{x}-{y}-{z}");
        self.cells.insert(at, Some(id.clone()));
        self.offer(
            &id,
            &format!("Stride to ({x}, {y}, {z})"),
            if ruled {
                "Spend an action on a GM-ruled move."
            } else {
                "Spend an action to move within Speed."
            },
            if ruled {
                Action::RuleStride { actor, to: at }
            } else {
                Action::Stride { actor, to: at }
            },
        );
    }

    fn push(self, built: &mut Built) -> Result<(), String> {
        let board = EncounterBoard {
            site: self.site,
            cells_per_square: self.spacing,
            tokens: self.tokens,
            cells: self
                .cells
                .into_iter()
                .map(|(at, stride)| BoardCell { at, stride })
                .collect(),
        };
        let values = vec![value(
            VALUE,
            serde_json::to_string(&board).map_err(|e| e.to_string())?,
        )];
        built.push(
            Item {
                id: format!("pf2e-board:{}", self.site),
                card: card("Encounter board", values, &[BADGE]),
                actions: self.actions,
            },
            self.acts,
        );
        Ok(())
    }
}

pub(super) fn append(
    built: &mut Built,
    cx: &Cx,
    session: &Session,
    sections: &BTreeMap<String, Vec<u8>>,
    game: &GameBindings,
) -> Result<(), String> {
    if game.positions().is_empty() || !sheets_pf2e::plays(sections)? {
        return Ok(());
    }
    let state = State::read(sections)?.unwrap_or_else(|| State::new(session.sim.genesis().seed));
    let access = cx.cut.access;
    let observer = access.subject.map(|body| body.0);
    let player = access.seat == Seat::Player;
    let here = observer
        .and_then(|id| session.sim.state().population.get(id))
        .and_then(|body| body.resident_site());
    let mut sites = BTreeMap::<u64, Vec<&PositionBinding>>::new();
    for position in game.positions().values() {
        let site = position.placement.site;
        if cx.cut.visible.sites.contains(&site)
            && cx.cut.visible.shows(position.entity)
            && (!player || here == Some(site))
        {
            sites.entry(site).or_default().push(position);
        }
    }
    let active = state.encounter.as_ref().and_then(|e| e.current());
    let mut start = sheets_pf2e::start_encounter(&state, access.seat == Seat::Dm);
    let marked = game.positions().values().any(|p| p.spatial.is_some());
    for (site, positions) in sites {
        let eye = if player && marked {
            // A marked world requires this observer's actual admitted eye. A
            // missing/stale eye withholds the board; the table remains usable.
            let Some(observer) = observer else { continue };
            match eye::current(session, game, observer, site) {
                Ok(eye) => Some(eye),
                Err(_) => continue,
            }
        } else {
            None
        };
        let mut board = Board {
            site,
            spacing: state.cells_per_square.unwrap_or(1),
            cells: BTreeMap::new(),
            tokens: Vec::new(),
            actions: Vec::new(),
            acts: Vec::new(),
        };
        for p in &positions {
            let Ok(at) = p.current(session) else { continue };
            if Some(p.entity) != observer
                && eye
                    .as_ref()
                    .is_some_and(|eye| !eye.sees(at, Some(p.placement.patch)).unwrap_or(false))
            {
                continue;
            }
            board.tokens.push(BoardToken {
                body: p.entity,
                name: session.sim.state().population.get(p.entity)
                    .and_then(|body| body.authored.as_ref()).map(|author| author.name.trim())
                    .filter(|name| !name.is_empty()).unwrap_or(&p.placement.character).to_owned(),
                at,
                active: active.is_some_and(|t| u64::from(t.token) == p.entity),
            });
            board.cells.insert(at, None);
        }
        if board.tokens.is_empty() {
            continue;
        }
        if let Some((id, label, why, act)) = start.take() {
            board.offer(&id, &label, &why, act);
        }
        let active_id = board.tokens.iter().find(|t| t.active).map(|t| t.body);
        // Frame around the acting character for a truth seat, or the actual
        // observer for a player. Keep remote knowledge out of local geometry.
        let origin = if player {
            observer
        } else {
            active_id.or_else(|| board.tokens.first().map(|t| t.body))
        };
        if let Some(p) = positions.iter().find(|p| Some(p.entity) == origin)
            && let Some(ground) = play::ground(session, p)
        {
            if !player || eye.is_some() {
                let [x, _, z] = p.placement.at;
                for dx in -5..=5i64 {
                    for dz in -5..=5i64 {
                        let Some(x) = x.checked_add(dx * i64::from(board.spacing)) else {
                            continue;
                        };
                        let Some(z) = z.checked_add(dz * i64::from(board.spacing)) else {
                            continue;
                        };
                        if let Some(at) = ground.landing(x, z)
                            && eye
                                .as_ref()
                                .is_none_or(|eye| eye.sees(at, None).unwrap_or(false))
                        {
                            board.cells.entry(at).or_default();
                        }
                    }
                }
            }
        }
        if let Some(turn) = active
            && let Some(actor) = active_id
            && ruleset::controls(session, access, actor)
            && state.pending.is_none()
        {
            board.offer(
                "vtt.pf2e-end-turn",
                "End turn",
                "Pass to the next in initiative order.",
                Action::EndTurn(actor),
            );
            let token = ruleset::token(actor)?;
            let map = state.current_map(session)?;
            if turn.actions > 0
                && !play::down(&map, token)
                && let Some(p) = positions.iter().find(|p| p.entity == actor)
                && let Some(build) = state
                    .drafts
                    .get(&token)
                    .and_then(|d| d.finished(records::builtin()).ok())
            {
                let gm = access.seat == Seat::Dm;
                let gear = play::gear(&state, token);
                let speed = play::speed(&build, &gear) as u64;
                let ruled = state.cells_per_square.is_none();
                // The previous, unmarked player path contains no terrain eye.
                // Retain just its established compass Strides in that case.
                let choices: Vec<_> = if (player && eye.is_none()) || ruled {
                    sheets_pf2e::strides(&state, session, sections, actor, token, gm)
                        .into_iter()
                        .map(|(_, at, _)| at)
                        .filter(|at| eye.is_none() || board.cells.contains_key(at))
                        .collect()
                } else {
                    board
                        .cells
                        .keys()
                        .copied()
                        .filter(|at| combat::feet(p.placement.at, *at, board.spacing) <= speed)
                        .collect()
                };
                if let Some(ground) = play::ground(session, p) {
                    for at in choices {
                        if at != p.placement.at
                            && ground.landing(at[0], at[2]) == Some(at)
                            && ground.fits_eye(p, at)
                            && at.iter().all(|v| i32::try_from(*v).is_ok())
                        {
                            board.stride(actor, at, ruled);
                        }
                    }
                }
                let weapon = gear
                    .weapon
                    .cloned()
                    .unwrap_or_else(isocosm_system::pf2e::play::fist);
                for target in board.tokens.clone() {
                    let Some(sheet) = ruleset::token(target.body).ok().and_then(|t| map.sheet(t))
                    else {
                        continue;
                    };
                    if target.body == actor
                        || sheet.int("dead").unwrap_or(0) != 0
                        || (ruled && !gm)
                        || play::strike_reach(
                            &state,
                            session,
                            sections,
                            access,
                            actor,
                            target.body,
                            &weapon,
                            ruled,
                            &[],
                        )
                        .is_err()
                    {
                        continue;
                    }
                    let (prefix, label, act) = if ruled {
                        (
                            "vtt.pf2e-rule-strike",
                            "Rule within reach and Strike",
                            Action::RuleAttack {
                                actor,
                                target: target.body,
                            },
                        )
                    } else {
                        (
                            "vtt.pf2e-strike",
                            "Strike",
                            Action::Attack {
                                actor,
                                target: target.body,
                            },
                        )
                    };
                    board.offer(
                        &format!("{prefix}-{}", target.body),
                        &format!("{label} {}", target.name),
                        "Spend an action to attack this character.",
                        act,
                    );
                }
            }
        }
        board.push(built)?;
    }
    Ok(())
}
