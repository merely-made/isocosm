//! Window-local fixture for a real owner and invited player. The player reads
//! only its rendered table, including on a second launch after rejoining.

use super::*;

enum Mode {
    Owner { wait_for_peer: bool },
    Player,
    Rejoin([i64; 3]),
}

pub(super) struct Peer {
    mode: Mode,
    actor: u64,
    target: u64,
    clicked: Option<u64>,
    owner_click: Option<(u64, u64)>,
    owner_observation: Option<(u64, u64, Option<u64>, String)>,
    destination: Option<[i64; 3]>,
}

impl Peer {
    pub(super) fn from_env(mode: &str) -> Option<Self> {
        let mode = match mode {
            "peer-owner" => Mode::Owner {
                wait_for_peer: std::env::var_os("ISOCOSM_PF2E_WAIT_FOR_PEER").is_some(),
            },
            "peer-player" => Mode::Player,
            "peer-rejoin" => Mode::Rejoin(
                serde_json::from_str(
                    &std::env::var("ISOCOSM_PF2E_DESTINATION").expect("rejoin destination"),
                )
                .unwrap(),
            ),
            _ => return None,
        };
        let body = |key| {
            std::env::var(key)
                .expect("peer fixture body")
                .parse()
                .unwrap()
        };
        Some(Self {
            mode,
            actor: body("ISOCOSM_PF2E_STRIDER"),
            target: body("ISOCOSM_PF2E_TARGET"),
            clicked: None,
            owner_click: None,
            owner_observation: None,
            destination: None,
        })
    }

    pub(super) fn drive(&mut self, app: &App, ctx: &mut Ctx<'_>, painted: &mut Painted) -> bool {
        if let Mode::Owner { wait_for_peer } = self.mode {
            let Some(table) = app.family_table.as_ref() else {
                return false;
            };
            if wait_for_peer && app.family_request_pumps.is_empty() {
                return false;
            }
            let turn = table
                .edition_state()
                .unwrap()
                .and_then(|s| s.encounter)
                .and_then(|e| e.current().map(|t| u64::from(t.token)));
            if turn == Some(self.actor) {
                if ctx.runner.state().vtt_selected_projection != TABLE {
                    painted.tab(ctx, TABLE);
                    return false;
                }
                eprintln!(
                    "[pf2e-peer] owner started encounter; player {} has the turn",
                    self.actor
                );
                return true;
            }
            if ctx.runner.state().vtt_selected_projection != TABLE {
                painted.tab(ctx, TABLE);
                return false;
            }
            let view = ctx
                .runner
                .state()
                .vtt_projections
                .iter()
                .find(|p| p.id == TABLE)
                .unwrap();
            let cut = (view.epoch, view.revision);
            let status = &ctx.runner.state().status;
            if self.owner_observation.as_ref().is_none_or(
                |(epoch, revision, last_turn, last_status)| {
                    (*epoch, *revision, *last_turn) != (cut.0, cut.1, turn) || last_status != status
                },
            ) {
                eprintln!(
                    "[pf2e-peer] owner ledger={} cut={cut:?} turn={turn:?} status={status}",
                    table.sequence()
                );
                self.owner_observation = Some((cut.0, cut.1, turn, status.clone()));
            }
            if self.owner_click == Some(cut) {
                return false;
            }
            let action = if turn.is_some() {
                "vtt.pf2e-end-turn"
            } else {
                "vtt.pf2e-start-encounter"
            };
            if painted.button(ctx, action, None) {
                self.owner_click = Some(cut);
            }
            return false;
        }

        assert!(
            ctx.runner.state().invited_table,
            "peer fixture needs the guest window"
        );
        let Some(view) = ctx
            .runner
            .state()
            .vtt_projections
            .iter()
            .find(|p| p.id == TABLE)
        else {
            return false;
        };
        let revision = view.revision;
        if ctx.runner.state().vtt_selected_projection != TABLE {
            painted.tab(ctx, TABLE);
            return false;
        }
        let (token, landings) = {
            let document = ctx.runner.dom();
            let dom = document.borrow();
            let at = |node| {
                dom.attribute(
                    node,
                    &Namespace::from(""),
                    &LocalName::from("data-native-at"),
                )
                .and_then(|s| serde_json::from_str::<[i64; 3]>(s).ok())
            };
            let actor = self.actor.to_string();
            let token = dom
                .all_with_class(dom.document(), "vtt-board-token")
                .into_iter()
                .find_map(|n| {
                    let body =
                        dom.attribute(n, &Namespace::from(""), &LocalName::from("data-body"))?;
                    (body == actor).then(|| at(n)).flatten()
                });
            let landings: Vec<_> = dom
                .all_with_class(dom.document(), "vtt-board-stride")
                .into_iter()
                .filter_map(|n| at(n).map(|a| (n, a)))
                .collect();
            (token, landings)
        };
        let Some(at) = token else {
            return false;
        };
        if let Mode::Rejoin(expected) = self.mode {
            if at != expected {
                return false;
            }
            eprintln!(
                "[pf2e-peer] rejoined player={} at={at:?} revision={revision}",
                self.actor
            );
            return true;
        }
        if let Some(destination) = self.destination {
            if at != destination {
                return false;
            }
            if let Some(previous) = self.clicked {
                if revision <= previous || ctx.runner.state().status != "Action accepted" {
                    return false;
                }
                eprintln!(
                    "[pf2e-peer] player={} at={destination:?} target={} strike accepted revision={revision}",
                    self.actor, self.target
                );
                return true;
            }
            if painted.button(ctx, &format!("vtt.pf2e-strike-{}", self.target), None) {
                self.clicked = Some(revision);
            }
        } else if let Some((node, destination)) = landings
            .into_iter()
            .min_by_key(|(_, to)| (0..3).map(|i| at[i].abs_diff(to[i])).max().unwrap())
        {
            if painted.click_node(ctx, node) {
                eprintln!(
                    "[pf2e-peer] player={} clicked Stride from={at:?} to={destination:?} revision={revision}",
                    self.actor
                );
                self.destination = Some(destination);
            }
        }
        false
    }
}
