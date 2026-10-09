# Directing and the interim M4: Mesocosm's first played loop on Isocosm

**Date:** 2026-10-08

**Status, 2026-10-08:** plan. Assessment done and its forks ruled (681 to
691); no phase opened. It carries the
[Mesocosm overlay plan](2026-09-25_mesocosm_overlay_plan.md)'s M3 and the
interim M4 of ruling 680, and the full M4 stays there.

Rulings live in the [wing design record](2026-09-18_wing_design_plan.md);
this plan cites them by number and does not restate their reasoning.

## 1. What this plan does

It builds directing (194, the overlay plan's M3) on native Isocosm and then
the interim M4 (680): a first played loop at site grain, before places, with
rounds, births, boundaries and a collapse, its receipts replaying. The
interim M4 is the switch (681): genet's runtime moves onto a native `Session`
there, legacy driving ends (671), and in-site play (carve, deposit, voxel
sight) waits for the full M4.

Rulings carried:

| Ruling | What it settles here |
|---|---|
| 177, 178 | a critter follows its player by its bond; the bond's inheritance is a world setting, seeded by default |
| 194, 679 | directing only on Isocosm; the six driving intents are the critter's own acts, `Idle` goes |
| 216, 59 | standing orders grow from attention, as desire paths |
| 225, 226, 227 | a region collapses when a trophic level is gone; regions follow biomass; mood is read, never kept |
| 671 | a critter that is not sapient is directed, never driven |
| 680, 681 | the interim M4 at site grain is the switch |
| 682 | M3 plays a native generated lineage |
| 683 | a deliberative methodology makes the critter's choice |
| 684 | the interim boundary ports grow-a-copy scoring over `Session::fork_at` |
| 685 | a suggestion is a reading over `Receipt.foregone` and `knows` |
| 686 | the runtime translates contract envelopes into native commands |
| 687, 689 | regions group sites by biomass, derived each round |
| 688 | the bond is a weighted relation to a placeless participant entity, one per player |
| 690 | standing orders are read from the logged nudges and their outcomes |
| 691 | survival mode filters through `knows` and the reach field |

## 2. What exists (survey of main at `a02b680a`)

- No player, played entity or participant anywhere in native Isocosm.
  `isocosm-overlay` (1,256 lines) is depended on by nothing.
- The scheduler (`schedule/mod.rs`, 348 lines) runs every due process for
  every ready group and skips `Method::Inert`. `Method::Deliberative` and
  `Normative` exist (`schema.rs:30-31`) and nothing uses them.
  `Receipt.foregone` (`execute.rs:136`) already lists the Choice processes
  passed over.
- `rules/mind.rs` (36 lines) has `Mind` and `Need`; mood is read from needs
  (`meaning/mod.rs`). Generated worlds set `mind: None` (`generate.rs:285`).
  The `sim:attention` account (`AccountKind::Attention`) is created and
  never used.
- `Relation` is `(subject, kind, object)` with no strength (`schema.rs:194`).
  The world is already a placeless entity of `kingdom:world`
  (`generate.rs:108`).
- `history::Command` has `Act`, `Inspect`, `Release`, `Collect`, `Learn` and
  `PlaceMatter`; no nudge. `Session` saves v3, loads v1 to v3, forks at a
  tick and merges.
- `Effect::Move` along site routes writes `visits`; no generated process
  uses it.
- `DeepTimeSpan` and `deep_time_ceiling` (`rules/epoch.rs`) are validated
  only: no native deep-time runner.
- Genet's W4 bench (`mesocosm-genet/src/app/bench/`, 8,288 lines) already
  drives a native `Session` beside the legacy world. The legacy runtime is
  wired through 81 genet files importing `isocosm::legacy` and 77
  `Intent::` uses.

## 3. Phases

Each phase is verified in its own worktree (tests pass, controls fail where
they should, replay hashes check) before it touches main.

**D1. Participants, the bond and the nudge.** A placeless participant
entity; `Relation` gains a value (read as zero from older saves); one
weighted bond relation per participant and critter (688); a logged native
`Nudge` command (686) naming a place (a site at this grain, 690) or a
thing, to attend or to act; 178's three settings in the rules, seeded by
default. Done when a seeded run with nudges replays to the same hash, a v3
save from before D1 still loads, and in the bond's controls outcomes that
serve the critter raise the weight and outcomes that harm it lower it, over
draws.

**D2. The deliberative methodology (683).** A scheduler methodology on
`Method::Deliberative` that, for a deliberative critter, chooses one
Choice process by needs, mood, nudges and bond, the rest going to
`Receipt.foregone`. Generated worlds give the played lineage a mind. Done
when an un-nudged critter acts on its own needs; a nudge moves its choice
in proportion to the bond, and with a zero bond it moves nothing (the
negative control); every other methodology's runs are unchanged (their
hashes before and after match); and choice is deterministic under the
seed.

**D3. Readings.** Suggestions (685), standing orders (690), regions (687,
689, with each trophic level's slot capacity a world rule and regions
growing along site routes) and the survival filter (691), each a pure
function of state and log. Done when each reads the same from a replayed
run as from the live one, a planted fault in each is caught by its test,
and regions merge into one as a draw's biomass falls.

**D4. The interim loop on native.** Founding from a seed, a start the
player picks (179), rounds and epochs, a birth keeping the parent by
default (183), death handing the next life from the cohort (61), the
boundary where every lineage adapts by grow-a-copy scoring over `fork_at`
(684), a regional collapse (225) played on elsewhere, survival and
creative modes. Done headless, by draws: three epochs end to end from
seeds nobody chose, receipts replaying to the same hash, controls for a
collapse that does and one that does not happen.

**D5. The switch (681).** A runtime over `Session` in genet, translating
contract envelopes (686); the section, HUD, succession and review lanes
re-pointed to it; legacy driving removed (671, 679). Done when 680's
condition holds in the headed host: three epochs end to end at site grain
on Isocosm, both modes, receipts replaying.

## 4. Open, to be put when a phase reaches it

- D4's player-picked start needs a native deep-time runner and a
  habitability reading for the critter; neither exists.
- Playing on after a regional collapse needs some generated process to use
  `Effect::Move`; which acts move a body between sites is unruled.
- Wounds feeding mood (227) wait on checkpoint 10; D2 reads needs and
  mood without them.
- Whether a participant entity can be shared by games beyond Mesocosm
  (the VTT's players, Eponym's sophont player).

## Findings

- **2026-10-08:** the survey's counts above, spot-checked: no manifest
  names `isocosm-overlay` but its own; `Deliberative` and `Normative` occur
  only at `schema.rs:30-31`; `generate.rs:285` sets `mind: None`;
  `sim:attention` is created at `generate.rs:122` and read nowhere.

## Progress

- **2026-10-08:** plan drafted from the directing survey; rulings 681 to
  691 taken in three rounds.
