// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Request editing stays in the host; admitted recipes and matter stay in core.
use super::{
    state::{Bench, Specimen},
    view::Child,
};
use cambium::{TextInput, clickable, el, focusable, lens, text, text_field_typed};
use mesocosm_core::{
    Kingdom,
    world::generation::{Archetype, BodyPlan},
};
use mesocosm_mesh::content::ContentPack;

pub(super) struct Controls {
    pub open: bool,
    pub seed: TextInput,
    pub mass: TextInput,
    pub size: u8,
    pub base: Option<ContentPack>,
}

impl Controls {
    pub fn new(model: &Specimen) -> Self {
        Self {
            open: false,
            seed: TextInput::new(model.creator.request.seed.to_string()),
            mass: TextInput::new(model.creator.request.criteria.mass_mg.to_string()),
            size: 1,
            base: model.content.clone(),
        }
    }
}

impl Bench {
    pub fn apply_generation(&mut self) {
        self.commit_generation(|_| {});
    }

    fn commit_generation(
        &mut self,
        change: impl FnOnce(&mut mesocosm_core::world::generation::Request),
    ) {
        let seed = match self.generation.seed.text().trim().parse::<u64>() {
            Ok(seed) => seed,
            Err(_) => {
                self.notice = "Seed must be a whole number from 0 to 18446744073709551615.".into();
                return;
            },
        };
        let mass = match self.generation.mass.text().trim().parse::<u64>() {
            Ok(mass) if (64..=10_000).contains(&mass) => mass,
            _ => {
                self.notice = "Mass must be 64–10000 mg.".into();
                return;
            },
        };
        let mut request = self.model.borrow().creator.request.clone();
        request.seed = seed;
        request.criteria.mass_mg = mass;
        request.fixed_body = None;
        change(&mut request);
        if let Err(why) = request.validate() {
            self.notice = format!("Request refused: {why:?}");
            return;
        }
        self.restore = None;
        let mut model = self.model.borrow_mut();
        model.creator.request = request;
        model.creator.regenerate();
        model.replaced();
        self.notice.clear();
        self.events.push("generation-applied".into());
    }

    pub fn use_archetype(&mut self, archetype: Option<Archetype>, plan: BodyPlan) {
        self.commit_generation(|request| {
            request.criteria.archetype = archetype;
            request.criteria.body_plan = plan;
            request.criteria.role = archetype.map(Archetype::role);
            request.criteria.movement_organs = None;
            request.variation = 0;
        });
    }

    pub fn generation_role(&mut self, role: Option<Kingdom>) {
        self.commit_generation(|request| {
            if request
                .criteria
                .archetype
                .is_some_and(|a| Some(a.role()) != role)
            {
                request.criteria.archetype = None;
            }
            request.criteria.role = role;
        });
    }

    pub fn suggested_seed(&mut self, seed: u64) {
        self.generation.seed = TextInput::new(seed.to_string());
        self.commit_generation(|request| request.variation = 0);
    }

    pub fn random_seed(&mut self) {
        // Randomize locally, then retain the actual seed in the request and UI.
        let entropy = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |t| t.as_nanos() as u64);
        let seed = mesocosm_core::Rng::from_seed(entropy ^ self.model.borrow().epoch).next_u64();
        self.suggested_seed(seed);
    }

    pub fn generation_size(&mut self, size: u8) {
        let result = self
            .generation
            .base
            .as_ref()
            .ok_or("No admitted content pack.".to_owned())
            .and_then(|base| {
                base.resized(size)
                    .map_err(|e| format!("Size refused: {e:?}"))
            })
            .and_then(|pack| {
                pack.resolve()
                    .map(|volumes| (pack, volumes))
                    .map_err(|e| format!("Size refused: {e:?}"))
            });
        match result {
            Ok((pack, volumes)) => {
                self.restore = None;
                self.generation.size = size;
                let mut model = self.model.borrow_mut();
                model.creator.request.fixed_body = None;
                model.creator.replace_palette(pack.palette);
                model.content = Some(pack);
                model.volumes = volumes;
                model.replaced();
                self.notice.clear();
            },
            Err(why) => self.notice = why,
        }
    }
}

fn choice(label: &str, selected: bool, action: impl Fn(&mut Bench) + 'static) -> Child {
    Box::new(focusable(clickable(
        el("button", text(label.to_owned()))
            .attr("aria-label", label.to_owned())
            .attr("aria-pressed", selected.to_string())
            .attr(
                "class",
                if selected {
                    "generation-choice selected"
                } else {
                    "generation-choice"
                },
            ),
        move |state: &mut Bench, _| action(state),
    )))
}

pub(super) fn view(state: &Bench) -> Child {
    if !state.generation.open {
        return Box::new(el("div", ()));
    }
    let model = state.model.borrow();
    let criteria = &model.creator.request.criteria;
    let mut plans: Vec<Child> = [BodyPlan::Axial, BodyPlan::Branched, BodyPlan::Generated]
        .into_iter()
        .map(|plan| {
            choice(
                plan.label(),
                criteria.archetype.is_none() && criteria.body_plan == plan,
                move |s| s.use_archetype(None, plan),
            )
        })
        .collect();
    plans.extend(Archetype::ALL.into_iter().map(|a| {
        choice(a.label(), criteria.archetype == Some(a), move |s| {
            s.use_archetype(Some(a), BodyPlan::Axial)
        })
    }));
    let roles: Vec<Child> = [
        ("Any role", None),
        ("Producer", Some(Kingdom::Producer)),
        ("Consumer", Some(Kingdom::Consumer)),
        ("Decomposer", Some(Kingdom::Decomposer)),
    ]
    .into_iter()
    .map(|(label, role)| {
        choice(label, criteria.role == role, move |s| {
            s.generation_role(role)
        })
    })
    .collect();
    let sizes: Vec<Child> = (1..=3)
        .map(|size| {
            choice(
                &format!("Size {size}"),
                state.generation.size == size,
                move |s| s.generation_size(size),
            )
        })
        .collect();
    let seeds: Vec<Child> = [1, 7, 42]
        .into_iter()
        .map(|seed| {
            choice(
                &format!("Seed {seed}"),
                model.creator.request.seed == seed,
                move |s| s.suggested_seed(seed),
            )
        })
        .collect();
    let summary = model
        .creator
        .prepared
        .as_ref()
        .map_or_else(String::new, |p| {
            let draft = p.draft();
            format!(
                "{} admitted / {} attempts · variation {} · refusals: {}",
                draft.candidates.len(),
                draft.attempted,
                model.creator.request.variation,
                if draft.rejected.is_empty() {
                    "none".into()
                } else {
                    draft
                        .rejected
                        .iter()
                        .map(|(k, v)| format!("{k} ({v})"))
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            )
        });
    let mass = model.world().controlled().map_or_else(|| "No admitted specimen.".to_owned(), |o| {
        format!("Body {} mg · reserve {} mg · capacity {} mg. Size changes anatomy; mass changes tissue allocation and ecological accounts.", o.biomass_mg(), o.energy_mg, o.mass_ceiling_mg())
    });
    Box::new(el("section", (
        el("div", plans).attr("class", "toolbar"),
        el("div", (el("span", text("Seed")),
            el("div", lens(|input: &mut TextInput| text_field_typed(input), |s: &mut Bench| &mut s.generation.seed)).attr("class", "generation-input generation-seed").attr("id", "generation-seed"),
            el("span", text("Mass (mg)")),
            el("div", lens(|input: &mut TextInput| text_field_typed(input), |s: &mut Bench| &mut s.generation.mass)).attr("class", "generation-input generation-mass").attr("id", "generation-mass"),
            choice("Apply seed and mass", false, Bench::apply_generation),
            choice("Random seed", false, Bench::random_seed),
            choice("Save specimen", false, Bench::save_comparison),
        )).attr("class", "toolbar"),
        el("div", (el("div", seeds).attr("class", "toolbar"), el("div", sizes).attr("class", "toolbar"), el("div", roles).attr("class", "toolbar"))).attr("class", "generation-options"),
        el("p", text(mass)),
        el("p", text("Suggested seeds are reproducible starting points. Reroll changes the body variation while retaining the habitat seed. Size retains sensory and role-sensitive details at their admitted scale.")),
        el("p", text(summary)),
    )).attr("class", "generation-controls"))
}
