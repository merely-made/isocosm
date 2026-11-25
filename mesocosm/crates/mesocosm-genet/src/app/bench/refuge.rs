// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{Bench, Child, sim::button};
use cambium::{el, text};
use cosmograph_overlay::mesocosm::NudgeMeaning;
use mesocosm_runtime::refuge::{self, DAY, MeadowChange, Observation, Pair, Patch};
use std::sync::mpsc::{self, Receiver, TryRecvError};

enum Message {
    Sample(Pair),
    Finished(Result<(), String>),
}

#[derive(Default)]
pub(super) struct Panel {
    worker: Option<Receiver<Message>>,
    samples: Vec<Pair>,
    selected: usize,
    when: MeadowChange,
    request: Option<&'static str>,
    error: Option<String>,
}

impl Panel {
    pub(super) fn running(&self) -> bool {
        self.worker.is_some()
    }

    fn start(bench: &mut Bench, request: Option<(&'static str, NudgeMeaning)>) {
        let Some(when) = bench.refuge.as_ref().map(|panel| panel.when) else { return };
        let label = request.as_ref().map(|(label, _)| *label);
        let (send, receive) = mpsc::channel();
        let worker = std::thread::Builder::new()
            .name("meadow-comparison".into())
            .spawn(move || {
                let result =
                    refuge::compare_request_at(when, request.map(|(_, meaning)| meaning), |pair| send.send(Message::Sample(pair)).is_ok());
                let _ = send.send(Message::Finished(result));
            });
        bench.refuge = Some(match worker {
            Ok(_) => Self {
                worker: Some(receive),
                when,
                request: label,
                ..Self::default()
            },
            Err(e) => Self {
                when,
                error: Some(e.to_string()),
                ..Self::default()
            },
        });
    }

    pub(super) fn poll(&mut self) {
        let Some(worker) = &self.worker else { return };
        loop {
            match worker.try_recv() {
                Ok(Message::Sample(pair)) => {
                    if self.selected + 1 >= self.samples.len() {
                        self.selected = self.samples.len();
                    }
                    self.samples.push(pair);
                },
                Ok(Message::Finished(result)) => {
                    self.error = result.err();
                    break;
                },
                Err(TryRecvError::Disconnected) => {
                    self.error = Some("The comparison stopped before completion.".into());
                    break;
                },
                Err(TryRecvError::Empty) => return,
            }
        }
        self.worker = None;
    }

    pub(super) fn current(&self) -> Option<&Pair> {
        self.samples.get(self.selected)
    }
}

fn location(o: &Observation) -> &'static str {
    match (o.alive, o.site) {
        (false, _) => "died",
        (true, Some(0)) => "in meadow",
        (true, Some(1)) => "in refuge",
        _ => "travelling",
    }
}

fn patch(name: &str, a: &Patch, b: &Patch, alone_minutes: u64, trial_minutes: u64) -> Child {
    let row = |label: &str, left: String, right: String, role: &'static str| -> Child {
        Box::new(
            el(
                "div",
                (
                    el("span", text(label.to_owned()))
                        .attr("role", if role == "cell" { "rowheader" } else { role })
                        .attr("class", "patch-label"),
                    el("span", text(left))
                        .attr("role", role)
                        .attr("class", "patch-value"),
                    el("span", text(right))
                        .attr("role", role)
                        .attr("class", "patch-value"),
                ),
            )
            .attr("class", "patch-row")
            .attr("role", "row"),
        )
    };
    let mut rows = vec![row(
        "Reading",
        "Alone".into(),
        "Trial".into(),
        "columnheader",
    )];
    for (label, left, right) in [
        ("Resident grazers", a.grazers, b.grazers),
        ("Minutes this grazer spent here", alone_minutes, trial_minutes),
        ("Plant matter eaten", a.grazed, b.grazed),
        ("Eaten by this grazer", a.eaten_by_grazer, b.eaten_by_grazer),
        ("Shoot matter", a.shoots, b.shoots),
        ("Root matter", a.roots, b.roots),
        ("Dormant seeds", a.seeds, b.seeds),
        ("Seedlings emerged", a.emerged, b.emerged),
    ] {
        rows.push(row(label, left.to_string(), right.to_string(), "cell"));
    }
    Box::new(
        el(
            "section",
            (
                el("h2", text(name.to_owned())),
                el("div", rows)
                    .attr("role", "table")
                    .attr("aria-label", name.to_owned()),
            ),
        )
        .attr("class", "sim-site refuge-patch"),
    )
}

pub(super) fn view(panel: &Panel) -> Child {
    let mut controls = vec![button("Back to simulation", |s| s.refuge = None)];
    if !panel.running() {
        let switch = match panel.when {
            MeadowChange::Reopens => "Wait until day 6",
            MeadowChange::TurnsLean => "Return to day 2",
        };
        controls.push(button(switch, |s| {
            if let Some(p) = &mut s.refuge {
                let when = match p.when {
                    MeadowChange::Reopens => MeadowChange::TurnsLean,
                    MeadowChange::TurnsLean => MeadowChange::Reopens,
                };
                *p = Panel { when, ..Panel::default() };
            }
        }));
        for (label, meaning) in [
            ("Move toward refuge", NudgeMeaning::MoveToward),
            ("Move away from refuge", NudgeMeaning::MoveAway),
            ("Stay near refuge", NudgeMeaning::StayNear),
            ("Point out refuge", NudgeMeaning::Attend),
        ] {
            controls.push(button(label, move |s| Panel::start(s, Some((label, meaning.clone())))));
        }
        controls.push(button("Leave both alone", |s| Panel::start(s, None)));
    }
    if panel.selected > 0 {
        controls.push(button("Earlier observation", |s| {
            if let Some(p) = &mut s.refuge {
                p.selected = p.selected.saturating_sub(1);
            }
        }));
    }
    if panel.selected + 1 < panel.samples.len() {
        controls.push(button("Later observation", |s| {
            if let Some(p) = &mut s.refuge {
                p.selected = (p.selected + 1).min(p.samples.len() - 1);
            }
        }));
    }
    let mut rows: Vec<Child> = vec![
        Box::new(el("h1", text("Meadow and refuge"))),
        Box::new(el(
            "p",
            text(match panel.when {
                MeadowChange::Reopens => "Day 2 start. The meadow has reopened. Both histories start with grazer 15. The control leaves it alone.",
                MeadowChange::TurnsLean => "Day 6 start. The meadow has just gone lean. Both histories waited with grazer 15 untouched. Counts start here.",
            }),
        )),
        Box::new(el("div", controls).attr("class", "toolbar")),
    ];
    if let Some(p) = panel.current() {
        let weather = if p.trial.meadow_open { "open" } else { "lean" };
        let day = format!("Day {}", p.tick / DAY);
        let when = if p.tick % DAY == 0 {
            day
        } else {
            format!("{day} + {} min", p.tick % DAY)
        };
        rows.push(Box::new(el(
            "h2",
            text(format!("{when} · meadow {weather}")),
        )));
        rows.push(Box::new(el(
            "p",
            text(format!(
                "Grazer {} · Alone {} · Trial {} · {} [{}]",
                p.grazer,
                location(&p.alone),
                location(&p.trial),
                panel.request.unwrap_or("Leave alone"),
                p.trial.nudge
            )),
        )));
        if panel.request.is_some() {
            for (reason, minutes) in &p.trial.blocked_route_minutes {
                let reason = if reason == "missing requirement: MoodBelow { amount: 0 } = 0" {
                    "travel needs mood below zero and the grazer's mood was zero"
                } else {
                    reason.as_str()
                };
                rows.push(Box::new(el(
                    "p",
                    text(format!("Route closed for {minutes} min · {reason}.")),
                )));
            }
        }
        rows.push(Box::new(
            el(
                "div",
                vec![
                    patch("Meadow", &p.alone.patches[0], &p.trial.patches[0], p.alone.minutes[0], p.trial.minutes[0]),
                    patch("Refuge", &p.alone.patches[1], &p.trial.patches[1], p.alone.minutes[1], p.trial.minutes[1]),
                ],
            )
            .attr("class", "sim-sites"),
        ));
    } else if panel.running() {
        rows.push(Box::new(el("p", text("Opening the saved meadow…"))));
    }
    if let Some(e) = &panel.error {
        rows.push(Box::new(el("p", text(e.clone())).attr("role", "alert")));
    }
    Box::new(el("div", rows).attr("class", "bench sim"))
}
