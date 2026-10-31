// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The viewer's window: a panel of the world card's timeline and filters and
//! the disclosed clusters' actions beside the drawn graph. Every control
//! invokes an action the snapshot advertised; nothing here reads the world.

use std::cell::RefCell;
use std::rc::Rc;

use cambium::{AnyView, GenetCtx, GenetElement, clickable, custom_leaf, el, focusable, text};
use cosmograph_host::endpoint::graph::{CLOSE, Domain, OPEN};
use cosmograph_host::endpoint::{BIRTH, BRANCH, DEATH, FILTER, LINE, SCRUB, STEP};

use crate::board::{Board, LEAF};
use crate::client::{Client, Shown};

pub type Child = Box<dyn AnyView<Viewer, (), GenetCtx, GenetElement>>;
pub type Logic = fn(&Viewer) -> Child;

pub struct Viewer {
    pub client: Client,
    pub board: Rc<RefCell<Board>>,
    pub events: Vec<String>,
    pub graph_size: (u32, u32),
}

impl Viewer {
    pub fn number(&self, label: &str) -> u64 {
        self.client.world().and_then(|w| w.value(label)).and_then(|v| v.parse().ok()).unwrap_or(0)
    }
    pub fn filters(&self) -> Vec<Domain> {
        self.client.world().and_then(|w| w.value("Filters")).unwrap_or("")
            .split(", ").filter_map(Domain::parse).collect()
    }

    /// Invoke on `id` and redraw the board from the new snapshot.
    pub fn invoke(&mut self, id: &str, intent: &str, payload: &[u8]) {
        self.client.invoke(id, intent, payload);
        self.board.borrow_mut().sync(&self.client);
        let notice = if self.client.notice.is_empty() { "accepted" } else { &self.client.notice };
        self.events.push(format!("{intent} {id}: {notice}"));
    }
    pub fn advance(&mut self, ticks: u64) { self.invoke("world", STEP, format!(r#"{{"ticks":{ticks}}}"#).as_bytes()); }
    pub fn scrub(&mut self, tick: u64) { self.invoke("world", SCRUB, format!(r#"{{"tick":{tick}}}"#).as_bytes()); }
    pub fn line(&mut self, line: u64) { self.invoke("world", LINE, format!(r#"{{"line":{line}}}"#).as_bytes()); }
    pub fn toggle(&mut self, domain: Domain) {
        let mut kept = self.filters();
        if let Some(at) = kept.iter().position(|d| *d == domain) { kept.remove(at); } else { kept.push(domain); }
        let words: Vec<String> = kept.iter().map(|d| format!("\"{}\"", d.word())).collect();
        self.invoke("world", FILTER, format!(r#"{{"domains":[{}]}}"#, words.join(",")).as_bytes());
    }

    /// The shown site holding the most stored groups.
    pub fn busiest_site(&self) -> Option<String> {
        self.client.shown.iter().filter(|s| s.id.starts_with("site:") && !s.hidden)
            .max_by_key(|s| s.value("Stored groups").and_then(|v| v.parse::<u64>().ok()).unwrap_or(0))
            .map(|s| s.id.clone())
    }
    /// The first shown, living group offering `intent`.
    pub fn group_offering(&self, intent: &str) -> Option<String> {
        self.client.shown.iter().find(|s| !s.hidden && s.value("Alive") == Some("true")
            && s.actions.iter().any(|a| a.intent.0 == intent)).map(|s| s.id.clone())
    }
}

fn button(label: impl Into<String>, name: impl Into<String>, act: impl Fn(&mut Viewer) + 'static) -> Child {
    Box::new(focusable(clickable(
        el("button", text(label.into())).attr("class", "button").attr("aria-label", name.into()),
        move |s: &mut Viewer, _| act(s),
    )))
}

fn line(class: &str, words: String) -> Child { Box::new(el("div", text(words)).attr("class", class)) }

fn timeline(v: &Viewer) -> Vec<Child> {
    let (tick, head, at) = (v.number("Tick"), v.number("Head"), v.number("Line"));
    let mut rows: Vec<Child> = vec![
        line("title", format!("Tick {tick} of {head} · line {at} of {}", v.number("Lines"))),
        button("Advance 1", "Advance one", |s| s.advance(1)),
        button("Advance 10", "Advance ten", |s| s.advance(10)),
        button("Back 1", "Back one", move |s| s.scrub(tick.saturating_sub(1))),
        button("Back 5", "Back five", move |s| s.scrub(tick.saturating_sub(5))),
        button("Head", "To head", move |s| s.scrub(head)),
        button("Branch here", "Branch here", |s| s.invoke("world", BRANCH, b"null")),
    ];
    if v.number("Lines") > 1 {
        for other in (0..v.number("Lines")).filter(|l| *l != at) {
            rows.push(button(format!("Line {other}"), format!("Line {other}"), move |s| s.line(other)));
        }
    }
    let kept = v.filters();
    for domain in Domain::ALL {
        let mark = if kept.contains(&domain) { "on" } else { "off" };
        rows.push(button(format!("{} {mark}", domain.word()), format!("Filter {}", domain.word()),
            move |s| s.toggle(domain)));
    }
    rows.push(line("hash", format!("State {}", v.client.world().and_then(|w| w.value("State hash")).unwrap_or(""))));
    if !v.client.notice.is_empty() { rows.push(line("notice", v.client.notice.clone())); }
    rows
}

fn item(s: &Shown) -> Option<Child> {
    let mut row: Vec<Child> = vec![line("item-title", s.card.title.clone())];
    for action in &s.actions {
        let (intent, id) = (action.intent.0.clone(), s.id.clone());
        if ![OPEN, CLOSE, DEATH, BIRTH].contains(&intent.as_str()) { continue; }
        row.push(button(action.label.clone(), format!("{} {}", action.label, s.id),
            move |v| v.invoke(&id, &intent, b"null")));
    }
    (row.len() > 1).then(|| Box::new(el("div", row).attr("class", "item")) as Child)
}

pub fn root(v: &Viewer) -> Child {
    let mut items: Vec<Child> = vec![line("title", "Clusters".into())];
    items.extend(v.client.shown.iter().filter(|s| !s.hidden && s.id != "world").filter_map(item));
    let panel = el("div", (el("div", timeline(v)).attr("class", "group"), el("div", items).attr("class", "list")))
        .attr("class", "panel");
    let (w, h) = v.graph_size;
    let graph = custom_leaf::<Viewer, ()>(LEAF, w, h).attr("class", "graph").attr("role", "img")
        .attr("aria-label", "World graph");
    Box::new(el("div", (panel, graph)).attr("class", "frame"))
}

pub const SHEET: &str = "
.frame { display: flex; font-size: 13px; background: #0b1216; color: #e8dcc0; height: 100%; }
.panel { width: 300px; padding: 10px; overflow-y: auto; background: #111b21; }
.group { margin-bottom: 10px; }
.title { margin: 4px 0 6px 0; color: #f0dfb8; }
.button { padding: 3px 6px; margin: 2px; background: #22404f; color: #e8dcc0; }
.button:hover { background: #2f5a6e; }
.item { margin: 2px 0; }
.item-title { color: #b8c8cc; }
.hash, .notice { color: #8fa3a8; margin-top: 4px; }
.graph { flex-grow: 1; }
";
