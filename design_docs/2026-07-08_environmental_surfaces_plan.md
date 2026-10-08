# Environmental surfaces: what a ruleset reads at a battlemap

**Date:** 2026-07-08, rewritten 2026-09-26 as a VTT note (wing design record
ruling 312).
**Status, 2026-09-26:** A VTT note since 2026-09-26 (ruling 312), not a lane. *(Brought current 2026-10-06 under ruling 618; the
earlier status line follows as written.)*

*Earlier:* **Status:** note beside the [VTT overlay plan](2026-09-25_vtt_overlay_plan.md)'s
V3, not a lane. Nothing has landed and nothing is scheduled here.

## What this was

The 2026-07-08 plan proposed surfaces (fire, water, grease, ice, poison) as a
per-tile layer of the VTT's substrate, spread by `flood_region`, painted by
area templates, with the interaction matrix (fire on grease, water plus
lightning, water plus cold) in the Lua system plugin as a ruleset dial from
faithful 5e to Larian-heavy. It cited Larian's Divinity: Original Sin and
Baldur's Gate 3 and Owlcat's Pathfinder games for the design space. A
2026-08-08 audit corrected its authority posture (core stores, the ruleset
resolves once), and W1 (2026-09-18) found the tier wrong: the sim holds the
environment, not one product's tile layer (record §3.1, §3.3; ruling 12).
Mark ruled it rewritten as this note rather than archived.

## What the sim owns

- **The field.** Environment, weather and climate, is a field on places,
  moved by agentless processes: "a rule over conditions on places running
  with nobody choosing, as field passes over the graph" (sim plan §2.4,
  §3.3; rulings 72, 98). Fire, flood, ice and spread are this shape; wild
  events and catastrophes are this shape and the third.
- **Diffusion is one mechanism.** Temperature, plague, reach and price
  diffuse over the place graph by the same passes (sim plan §2.4); a
  surface that spreads is that mechanism at the near rung, over bricks with
  an interior (ruling 12), not a flood fill over tiles.
- **Conditions live on places, never on things** (record §3.5). "Wet",
  "burning" and "prone on ice" are the sim's conditions on the cell a body
  occupies, under a content-addressed rules revision (ruling 32).
- **The battlemap is a far view of that volume** in the game's grid (VTT
  overlay plan §3, rulings 12, 18, 243), so a surface is drawn where the
  field says it is.

## What a ruleset reads

The ruleset never owns spread. At a battlemap it reads the environment field
and the conditions on the cells its actors occupy and adjudicates by its own
rules, returning the result through the handoff (ruling 154; V3):

| The ruleset asks | The sim answers from |
| --- | --- |
| what is on this cell | the environment field and the cell's conditions |
| what happens to a body standing there at the end of its turn | the ruleset's own rule, calibrated against the sim's processes (rulings 114, 189, 249) |
| whether a spell's area ignites, douses or freezes | an intent whose effect the ruleset declares; the sim runs the agentless process that follows (sim plan §3.3) |
| whether fire reaches the next cell | the field pass, at the sim's clock (ruling 257), never a table op |

The Larian interaction matrix is therefore a set of process definitions a
world declares (sim plan §3.1), not a ruleset dial: a world with fire that
spreads through grease has that process; a world without does not, and a
pack forces it where the world lacks it (rulings 89, 190). The 5e area
spells that paint surfaces are ruleset intents whose effects those processes
carry.

## Open, for V3

- The grain at which the sim's field is read at a battlemap: the cell, or
  the tile's five-voxel face (ruling 244's per-tick batches).
- Whether a ruleset may declare a condition the world has no process for,
  and what an uncalibrated one shows (rulings 189, 250).
- Rendering: surfaces as voxel appearance over the traced ground is the
  isometer family's, not this note's.

## Relation to prior docs

- The original plan's text is in this file's git history before 2026-09-26.
- The campaign packs plan's decision 12 (spread as a DM-authority op) is
  superseded for the sim by rulings 154 and 243: the table sends intents,
  the sim runs the process.
