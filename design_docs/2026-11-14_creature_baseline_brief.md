# A baseline of creatures: one authored set the three games seed from

**Status:** 2026-11-14: brief for Mark to rule on. No code. Mark answered the
same day (§9): the VTT is the first release target and the foundation stays
the three games'; the sort by what a table sees is withdrawn. Questions open
(§8).

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

**It does not sort the remaining mechanics.** 78 parts wait, on sixteen
mechanics and six proposed processes. The first draft of this brief sorted
them by whether a table would see them and proposed stopping the unseen
half. Mark withdrew that (§9): unseen things matter to a table (a plague is
host transmission, a fire is a disturbance), and cutting the sim to what one
product shows is the tailoring he does not want. The list stays whole. What
the baseline changes is its order, by what the foundation's own measurements
need next (§7).

## 6. Keeping the other two open

Mark's worry, in §9: a sim built toward the VTT's release becomes a sim
tailored to the VTT. What already holds against that, and what could be
added, so it rests on the structure and not on anyone's judgment in a
session:

- **Already so:** `shared/cosmograph` depends on no product crate. A game
  reaches the sim only by the overlay contract (case 154: "a game submits
  intents, the sim returns events and a read-only per-tick view, and settled
  outcomes return through the handoff"). The VTT's side of it is in
  `shared/cosmograph-overlay/src/vtt/`, outside the sim.
- **Already so:** the sim's measurements run in Mesocosm's shape (the played
  four-line world), and Mesocosm's suite runs on every sim change.
- **Proposed, for a ruling:** a sim change cites a sim case, a doctrine
  statement or a measurement as its reason. "The VTT needs it" is a reason
  to build something in the overlay or the VTT, and a reason to order sim
  work, never a reason to shape it.
- **Proposed:** the release target moves order only. Nothing leaves the
  catalog or a plan because the VTT does not use it; a thing may be parked
  with its reason, as plans already do.
- **Proposed:** what the VTT reads from the sim (a place's creatures and
  their links, as prompts) is a reading over the per-tick view, in the
  overlay. If it needs a fact the view lacks, the fact is added to the view
  for every game.
- **Proposed:** a baseline set is content in a pack, and a world without it
  founds and hashes as before, as every optional rule does.

## 7. A recommendation

In this order, each step a place to stop:

1. The sheet format, and five sheets by hand (two producers, a grazer, a
   predator, a decomposer) to find what the format lacks.
2. Found a world from them and re-run the existing measurements. If the
   authored set holds together where the drawn one did not, the approach is
   proven on what is already built. This step serves all three games and
   needs nothing of the VTT.
3. Fill to a first set, and restore the dropped speeds and senses in the
   pack.
4. The VTT reads a place's creatures and their links as prompts, through
   the overlay.
5. The waiting mechanics, in the order steps 2 and 3 show a need for them.

## 8. Questions for Mark

1. **Are §6's four proposed constraints right, and is one missing?** They
   are the answer to the worry in §9, and they are yours to state.
2. **Does a sheet declare what a creature eats?** Bodies 1 says a role is
   read from what members did, never chosen. A sheet naming a jaw and its
   `admits` words fixes what the creature can eat without naming a role; is
   that within Bodies 1, or does it need refining?
3. **May the sim carry a baseline of its own?** Licensing 1 keeps SRD text
   out of the sim. Sheets in a pack are clean. A set written fresh (no SRD
   names or text) could live in the sim's datasheets as the default
   founders; should it?
4. **Humanoids:** the denizen plan's templates (P2) are the body side. Does
   the baseline wait on P2, or carry humanoids as plain stat blocks until
   then?
5. **Does step 1 start?** Five sheets by hand and a measurement, before
   anything is built for the VTT.

## 9. Answered

Mark, 2026-11-14, on the first draft:

> "A visible versus not visible distinction is not where it's at. There are
> plenty of invisible things that would be important for the VTT. I do think
> that the VTT is the product that is most likely to be released soonest, so
> we can make that the target. But I don't want to make it the target in a
> way that precludes the possibility of doing the other wings, which is why
> I was working on the other wings, because I felt that was the foundation
> for doing all of them, right? So if we can make that foundation and
> release the VTT first, I'd rather do that. But I also don't necessarily
> have confidence in your ability to keep the possibility open by designing
> in a manner that can leverage a SIM that would then be tailored to the
> VTT, in effect."

Read as: the VTT is the first release target; the foundation is built for
all three games; the sort of mechanics by what a table sees, and the options
that stopped the unseen half or made the VTT the sole commitment, are
withdrawn. Whether this is a case for the design record is Mark's to say.
