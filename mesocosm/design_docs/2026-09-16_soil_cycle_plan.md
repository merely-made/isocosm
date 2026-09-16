# Soil cycle plan

**Date:** 2026-09-16

**Status, 2026-09-16:** plan; step S1 dispatched. Opened from the
[isoscape family plan](2026-09-16_isoscape_family_plan.md) ruling 20 and its
§2.6 assessment. Deep time's heir entry (D7b) waits on this plan.

**Owns:** making the enclosure's matter cycle work in the worlds the
generation door hands over: far-tier movement, how decomposers reach, sense
and eat carrion, how carrion returns matter with or without them, and how
many decomposers are founded, where, and in what form.

**Does not own:** deep time itself (the isoscape family plan), the heir
entry (D7b there), the multi-anchor network body (the playable ecology
plan's representation ruling), or any other kingdom's feeding rules except
where a step names the effect on them.

---

## 0. Rulings (Mark, 2026-09-16)

Answering §2.6 of the isoscape family plan.

1. **Fix far-tier movement first, in this lane.** A far body inside its
   target's place steps toward the target instead of leaving it, and a far
   body moves at most its dispersal budget per tick, paying per tick what a
   near body pays.
2. **Measure deep time near and far after that fix, then rule.** Isoscape
   ruling 19, everything far during deep time, stands until the numbers are
   in. Its premise was corrected in §2.6: every founder starts near.
3. **Reach, all four:** a decomposer's bite reads its body's reach; a
   scavenger closing on carrion spends its dispersal budget; founding gives
   fauna a sense organ; and decaying carrion leaves a scent in the soil that
   scavengers can follow.
4. **Supply, all four:** carrion decay proportional to mass as a declared
   world rule; more founding decomposers; decomposers founded where corpses
   fall; and sessile or creeping decomposers.

## 1. What the steps start from

Verified in §2.6 of the isoscape family plan, with `path:line`:

- **Far movement**: `graph_step` (`organism/ecology/movement.rs:566-582`)
  jumps to a neighbour place's centre and always leaves the target's place;
  `pay_travel` (`flows/returns.rs:55-73`) charges the jump to body substance.
- **Approach**: `walk_grounded` stops at `reach + GRAZE_RANGE`
  (`movement.rs:551-553`).
- **Bite**: flat `DECOMPOSE_RANGE = 6` (`movement.rs:39,103-116`).
- **Sight**: `sight_for_body(8, sensor_span, ..)` (`rates.rs:287-290`); no
  founding decomposer measured has a sense part.
- **Decay**: 1 mg every `CARRION_DECAY_TICKS = 4` ticks per corpse
  (`rates.rs:89-94`, `flows/returns.rs:76-97`).
- **Founding**: the pyramid is two thirds producers, a quarter consumers,
  the rest decomposers (`world/genesis.rs:520-545`), placed uniformly
  (`:223-226`); the door founds 24 by default (`world/generation.rs:103`).
- **Travel without machinery**: only an actuator span or the producer
  exception lets a body move (`rates.rs:381-383`).

**Instruments**: `mesocosm-core/examples/decomposer_probe.rs` (the first
epoch tick by tick, three arms: deep, near, hand) and
`matter_ledger_probe.rs` (where matter sits at each epoch).

## 2. Steps

Sequential by default: every step changes ecological outcomes, and each is
measured with both instruments before the next, so an effect can be
attributed. Steps on disjoint files may run in parallel only where the
measurement of one does not depend on the other.

| Step | Builds | Where | Model |
| --- | --- | --- | --- |
| **S1** | Far-tier movement (ruling 1): no bounce inside the target's place; at most the dispersal budget per tick, paid per voxel moved | `organism/ecology/movement.rs` | opus |
| **S2** | Deep time near and far, measured after S1 on seeds 7, 1 and 42 over six epochs: survival by kingdom, soil, carrion, wall time. Ruling 19 goes back to Mark with the numbers | measurement only | orchestrator |
| **S3** | The scavenger's bite reads its reach, and a scavenger closing on carrion spends its dispersal budget (ruling 3) | `movement.rs` | sonnet |
| **S4** | Carrion decay proportional to mass, a declared world rule in `WorldRules` and its digest (ruling 4); the world-state pin moves once, with its reason | `rules.rs`, `flows/returns.rs`, `ecology.rs` | sonnet |
| **S5** | Founding: a sense organ for fauna, a decomposer floor at the door, and decomposers placed where corpses will fall (rulings 3 and 4) | `world/genesis.rs`, the drawn founding | opus |
| **S6** | Soil scent: decaying carrion deposits typed matter that spreads, and scavengers follow its gradient (ruling 3) | `flows/returns.rs`, `perception.rs`, `places/soil.rs` | opus |
| **S7** | Sessile and creeping decomposers (ruling 4), under the reading §3 asks Mark to confirm | `rates.rs`, `movement.rs` | sonnet |
| **S8** | Re-measure deep time and the matter ledger on all three seeds; hand D7b a world with a working soil cycle | measurement only | orchestrator |

## 3. Decisions still Mark's

- **S7's reading.** Ruling 4 chose sessile or creeping decomposers, and
  ruling 3 improved walking ones. Proposed reading: both forms exist, set by
  the body. A decomposer drawn without locomotion machinery is sessile and
  creeps when hungry, as producers do; a limbed one walks, with S3's reach.
- **S4's rate.** Decay proportional to mass needs a value; S4 measures
  candidates and brings them.
- **S5's numbers.** The decomposer floor, the sense organ's form and the
  placement rule each need a value; S5 measures and brings them.
- **Ruling 19**, after S2.

## 4. Done conditions

1. **S1.** A far body in its target's place approaches the target and does
   not leave; no far body moves more than its dispersal budget in a tick;
   travel paid equals voxels moved; in `decomposer_probe`'s deep arm no
   founding decomposer dies on a tick it hopped; far-tier cohort
   conservation still holds; every existing core and runtime test passes or
   a changed expectation is named with its reason.
2. **S2.** A table of near against far, per seed, per kingdom, with wall
   time; ruling 19 answered.
3. **S3 to S7.** Each lands with both instruments rerun and its effect
   stated against the previous step.
4. **S8.** On all three seeds, after six epochs of deep time, decomposers are
   alive, carrion holds less than a stated share of matter, and the
   generation door drafts candidates. The share is proposed with S8's
   numbers, not fixed in advance.

## Findings

- **2026-09-16.** Opened with §2.6's evidence; nothing new yet.

## Progress

- **2026-09-16.** Plan written; S1 dispatched.
