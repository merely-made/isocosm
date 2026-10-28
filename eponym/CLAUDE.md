# CLAUDE.md — Eponym

Derived, not authoritative (doctrine, Method 7): where the records are and
how to work here. Read `../CLAUDE.md` first; it loads with this file.

Eponym lives in `eponym/` with its own Cargo workspace; the native package is
`eponym-client`. Git operations from here affect the whole repository:
preserve other products' work and stage only the intended paths.

## Where the records are

- What Eponym is: `design_docs/PROJECT_DESCRIPTION.md` and the founding plan,
  `design_docs/2026-10-28_eponym_overlay_plan.md` (Paredros is Eponym's
  former name).
- Where it stands, and what each crate holds: `design_docs/DOC_README.md` and
  the overlay plan, `design_docs/2026-10-28_eponym_overlay_plan.md`.
- The wing's records (doctrine, design record, founding record): the root
  `design_docs/`, listed in `../CLAUDE.md`.

## Vocabulary

- character (this game's unit word: a faction-association on a denizen),
  succession, fili; the battle-frame noun stays unnamed: founding record §7
  and §8.
- denizen, sophont: Naming 2 (case 200).
- companion or peer, never "unit" or "party member": the design record.
- hagiograph, and the retired memorial sense of tulpa: the design record.

## Guardrails

- No real-time puppeteering of a party: configure, don't command (founding
  record §1, care granularity).
- Body switching is never a menu operation; control changes through death or
  a recorded world event, with consequences (founding plan, "One life, then
  continuation").
- No Nemesis system: rival-hierarchy promotion is patented to August 2036
  (founding record §1, game tastes).
- The social simulation ships with its legibility surface, and relationship
  drift has player levers (founding plan, "The social layer is the base
  layer"; founding record §2).
- No rollback netcode or universal CRDT world state speculatively (the
  shared-authority plan, root `design_docs/`).
