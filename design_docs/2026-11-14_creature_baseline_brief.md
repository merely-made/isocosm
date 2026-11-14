# A baseline of creatures: one authored set the three games seed from

**Status:** 2026-11-14: brief for Mark to rule on. No code. Questions open (§8).

**For:** Mark's 2026-11-13 idea that the VTT, with optional generative aspects
that prompt a table as a GM might, could be the wing's commitment, and that a
baseline of monsters, creatures and humanoids would seed Mesocosm's ecologies
and Eponym's factions and settlements. It lays out what exists, what is
ruled, what a baseline would be, what it changes in the catalog build, and
what only Mark can decide. "Baseline" is Mark's word; "creature sheet" and
"coarse layer" are working words, not names (Naming 3).

## 1. The idea, in Mark's words

> "i wonder if, like, the scope of the wing idea isn't basically impossible to
> do and if i might do better just making a nice vtt with optional generative
> aspects that can suffice for prompting a table like a gm might, which is
> already a holy grail of dnd-ing."

> "if scoped, the work to make the vtt feel alive and reactive is the same
> work required to do the other two. having a baseline of
> monsters/creatures/humanoids would let us seed mesocosm's generated
> ecologies and eponym's factions/settlements."

## 2. What exists today

- **A statted bestiary, in a pack.** `packs/5e-srd/monsters.toml` holds 334
  SRD 5.1 monsters (CC BY 4.0), read by `crates/isocosm-system/src/bestiary.rs`
  into `Monster`: key, name, size, kind, alignment, armour class, hit points,
  one speed, six abilities, challenge rating, actions as text with a to-hit
  and a damage expression, and a sprite key. 87 are beasts and 10 swarms;
  6 plants, 4 oozes; 50 humanoids. Of the 97 beasts and swarms, 48 have a
  bite and 13 a poison.
- **What the stat block does not hold:** diet, habitat, climate, senses, and
  fly, swim or burrow speeds (the trimmed shape kept one speed). Nothing in
  it says who eats whom or where a creature lives.
- **Appearance is a placeholder.** Every monster's sprite is one of four
  rigs (orc 115, goblin 112, wolf 87, skeleton 20).
- **A parts catalog, in the sim.** `shared/cosmograph/datasheets/catalog/`:
  272 parts, 100 hand-written (fauna 36, flora 25, myco 25, micro 14) and
  172 generated; 194 expressible, 62 waiting on a mechanic, 16 on a
  proposed process. Provisional; working keys.
- **Five optional ecology rules** (2026-11-12 to 14): spoilage and
  categories, defence, mutualism, dung, nutrient axes. Each is tested as
  code. Measured over 20 seeds, one of D39's tests passes (toxin against
  detox); the others fail or cannot tell
  (`mesocosm/design_docs/2026-11-09_lineage_rounds_plan.md`, Progress).
- **Two findings from those measurements that a baseline bears on:** no line
  adopts a costly part at a boundary (0 of 154 turns took a defence), and
  decomposer lines have no recipe, so they cannot take parts at all. Every
  measurement had to plant its organs by hand at tick 0.
- **One authored quick-start world:** `datasheets/quickstart/mill-ford.toml`.

## 3. What is already ruled

- **Authoring displaces generation** (World 5): "authoring goes as deep as
  anyone likes, filling or displacing generated content."
- **A pack displaces generated content where the world meets its
  requirements, and waits where it does not** (vtt-overlay V2, rulings 89,
  190).
- **Quick-start founders are authored** (World 10): "A quick-start world is
  a modded, shared world whose founders are authored and enter by the
  founding door at its first tick." In a deep-time world the generator
  places no people.
- **All three games read the same bodies** (cases 1128 to 1131; denizen
  plan). Denizen body plans are a drawn family in the recipe grammar
  (case 1133), with named plan templates as data (P2, not built). A
  creator designs an individual that seeds a lineage (case 1134, pieces
  still to rule).
- **An open text is a reference; nothing is copied into the sim**
  (content-packs Constraints, Licensing 1). The SRD may ship in a pack.
- **Kingdoms are playstyles, and a role is read from what members did,
  never chosen** (Bodies 1).

So most of the idea has a door already. A baseline is authored content
entering by the founding door, in a pack, displacing what would be drawn.
What is not ruled is in §8.

## 4. What a baseline would be

Three things, each smaller than what is being built now.

**A creature sheet.** One record per creature that names catalog parts and
the few ecological facts a stat block lacks: kingdom, what it eats (the
catalog's `admits` words), where it lives, and its parts by key. It lives in
a pack beside the ruleset's stat block and joins it by key, so the SRD text
stays in its pack and the sim reads only catalog keys. A sheet lowers to a
line's recipe by the repertoire's existing lowering.

**A baseline set.** Enough sheets to found a world that holds together: by a
rough count, twenty to forty across producers, grazers, predators, a
scavenger, decomposers and a few humanoids. The SRD's beasts are the obvious
first fill; its plants and fungi are too few (shrieker, violet fungus,
shambling mound and three more), so producers and decomposers would be
written fresh.

**A coarse layer the table reads.** Who lives at a place, who eats whom
there, and what shifts when one is removed. The VTT needs this as prompts
(an encounter, a rumour, a consequence of last session), not as a tick-level
ledger. The sim's coarse tier (Processes 6) is where it would be read from.

What each product takes:

| Product | Reads the baseline as |
| --- | --- |
| Isocosm (VTT) | tokens with a part-tree shape (case 1141), and prompts from the coarse layer |
| Mesocosm | founding lines with recipes and parts from tick 0, which the five rules can act on |
| Eponym | humanoid lines for settlements and factions (World 10's authored founders) |

## 5. What it changes in the catalog build

**It answers the two findings.** An authored predator arrives with its jaw
and a toxic fungus with its toxin, so nothing depends on lines adopting
costly parts, and an authored decomposer has a recipe. D39's tests become
"does this authored set hold together over N seeds", which a measurement
can answer.

**It gives the remaining mechanics a sort order.** 78 parts wait, on
sixteen mechanics and six proposed processes (a part may wait on more than
one, so the counts below overlap). Sorted by whether a baseline creature at
a table needs it:

| Needed by baseline creatures | Parts waiting | Why |
| --- | --- | --- |
| venom injection, bite mode | 3, 3 | 13 SRD beasts poison, 48 bite |
| flight | 1 | birds and bats (speeds dropped from the pack; to restore) |
| refuge, concealment | 8, 2 | burrowers, ambushers; D39 model 3 |
| signal reading | 11 | senses a GM describes; warning colours |

| Not visible at a table | Parts waiting |
| --- | --- |
| allelopathy | 10 |
| long-range dispersal, seed carriage | 8, 4 |
| host immunity, host transmission | 7, 7 |
| soil depth, disturbance | 7, 5 |
| gut symbiont, pollination | 2, 2 |
| nutrient fixing (the nutrient rule moves nutrients and makes none) | 3 |

The second table is where scope would come off. It is the larger half of
what waits, and most of the ecological depth of D37 to D39.

## 6. Options

**A. Baseline first, VTT as the commitment.** Write the sheet format and a
first set; re-measure the five rules on it; the VTT reads it for prompts.
The mechanics in the second table stop. Mesocosm and Eponym take the
baseline and otherwise wait.

**B. Baseline as a Mesocosm phase only.** The sheet and set are built as
the fix for the two findings, the catalog list goes on in full, and the
VTT's use of it is left for later. Smallest change of direction; no scope
comes off.

**C. Keep building the list, review after.** As ruled on 2026-11-13. The
baseline waits on the review.

**D. VTT without the sim.** Prompts from tables and the packs alone, the
sim off. Fastest to a table; the three games stop sharing their hardest
part, which is the reason for the wing.

## 7. A recommendation

A, in this order, each step a place to stop:

1. The sheet format, and five sheets by hand (two producers, a grazer, a
   predator, a decomposer) to find what the format lacks.
2. Found a world from them and re-run the existing measurements. If the
   authored set holds together where the drawn one did not, the approach is
   proven on what is already built.
3. Fill to a first set from the SRD's beasts, and restore the dropped
   speeds and senses in the pack.
4. The VTT reads a place's creatures and their links as prompts.

It reduces scope only if the second table in §5 really stops. Adding the
baseline while keeping the full list is a fourth project.

## 8. Questions for Mark

1. **Which option?** A is recommended.
2. **Does the second table in §5 stop, park, or go on?** This is the scope
   question itself.
3. **Does a sheet declare what a creature eats?** Bodies 1 says a role is
   read from what members did, never chosen. A sheet naming a jaw and its
   `admits` words fixes what the creature can eat without naming a role; is
   that within Bodies 1, or does it need refining?
4. **May the sim carry a baseline of its own?** Licensing 1 keeps SRD text
   out of the sim. Sheets in a pack are clean. A set written fresh (no SRD
   names or text) could live in the sim's datasheets as the default
   founders; should it?
5. **Humanoids:** the denizen plan's templates (P2) are the body side. Does
   the baseline wait on P2, or carry humanoids as plain stat blocks until
   then?
6. **Is "the VTT is the commitment" a ruling?** It would reorder the three
   products' plans, and is yours to state in the doctrine's words.
