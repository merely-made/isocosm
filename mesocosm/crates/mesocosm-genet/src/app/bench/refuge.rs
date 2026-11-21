// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{Bench, Child, sim::button};
use cambium::{el, text};
use mesocosm_runtime::refuge::{self, DAY, Observation, Pair, Patch};
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
    nudge: bool,
    error: Option<String>,
}

impl Panel {
    pub(super) fn running(&self) -> bool {
        self.worker.is_some()
    }

    fn start(bench: &mut Bench, nudge: bool) {
        let (send, receive) = mpsc::channel();
        let worker = std::thread::Builder::new()
            .name("meadow-comparison".into())
            .spawn(move || {
                let result =
                    refuge::compare(nudge, |pair| send.send(Message::Sample(pair)).is_ok());
                let _ = send.send(Message::Finished(result));
            });
        bench.refuge = Some(match worker {
            Ok(_) => Self {
                worker: Some(receive),
                nudge,
                ..Self::default()
            },
            Err(e) => Self {
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

fn patch(name: &str, a: &Patch, b: &Patch) -> Child {
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
        controls.push(button("Nudge toward refuge", |s| Panel::start(s, true)));
        controls.push(button("Leave both alone", |s| Panel::start(s, false)));
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
            text(
                "Seed 9. The meadow reopens on day 2. Both histories start with the same grazer. The control leaves it alone.",
            ),
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
                "Grazer {} · Alone {} · Trial {} · Nudge {}",
                p.grazer,
                location(&p.alone),
                location(&p.trial),
                p.trial.nudge
            )),
        )));
        if panel.nudge {
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
                    patch("Meadow", &p.alone.patches[0], &p.trial.patches[0]),
                    patch("Refuge", &p.alone.patches[1], &p.trial.patches[1]),
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
