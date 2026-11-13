# Mesocosm

Mesocosm now lives in the [Isocosm umbrella repository](../README.md), under
`mesocosm/`. It retains its own Cargo workspace and package names. See the
[wing index](../design_docs/DOC_README.md) for all three products.

A first-person game of lineages.

You are a critter in an enclosed mid-scale ecosystem. You grow only by
incorporating other organisms; you die, and your line continues or it does
not. Your aim is to increase your kind's share of the world's biomass, which
no predator manages alone. One verb: **metabolize**. World into self, self
into world, and the world keeps what your generations did to it.

Wing-level design (doctrine, the design record, the founding record) lives
in the umbrella's root `design_docs/`. This folder's `design_docs/` holds
Mesocosm's plans and the plans for Cosmograph, the shared simulator.

## Status (2026-11-04)

Early implementation. Mesocosm now runs as an overlay on Cosmograph, the
wing's shared simulator (`shared/cosmograph`); the played loop (M4) does
not run end to end yet.

- `mesocosm-core` and its own `Intent` are deleted. Four crates
  (`mesocosm-runtime`, `-views`, `-phenotype`, `-genet`) run a native
  Cosmograph `Session` and lower `cosmograph_overlay::mesocosm` envelopes.
- Directing (M3) is built on native Cosmograph and the switch has landed.
  The epoch boundary, review, revision and speciation live in the sim
  (`shared/cosmograph/src/lineage/`).
- In the sim, bodies are certified at site grain; ecology phases S8 to S10
  and tuning are open.
- The host shows three views over one shared view policy, one of them a
  perspective section, switched with Y (2026-11-02).
- Open: certification of M3 and the interim M4 by draws, the headed
  three-epoch run, the shared hybrid presentation's product acceptance
  (case 1161), and the full M4, where chosen in-site acts and contact reach
  the same accepted world.

Current plans live in [design_docs/](design_docs/DOC_README.md): the
[overlay plan](design_docs/2026-10-28_mesocosm_overlay_plan.md) for the
game and the [sim plan](design_docs/2026-10-28_sim_plan.md) for Cosmograph.
The standing caveat outranks all of it: the game must make somebody want
another run.

## Use

```sh
cargo run -p mesocosm-genet    # play in a window over a native Cosmograph session
cargo run -p mesocosm-genet -- --bench --seed 7 # the specimen bench over a native session
cargo run -p mesocosm-genet -- --help   # every flag and key
cargo test --workspace
```

Click or E/Space attends, right click or Q chooses a current hint, S
speciates, arrows pan, Z/V rotate, C switches surface and section, Y cycles
the overview, over the shoulder and first person, +/- zoom, and Esc saves
the trace and quits.

Rounds (the [lineage-rounds plan](design_docs/2026-11-09_lineage_rounds_plan.md)):
`--rounds waiting|continuous` changes how a round runs past its span,
`--condition` sets a mark to beat, and `--rounds-rules` turns on the drift,
the draft and the sheet (`--drift`, `--draft`, `--sheet` singly). `--climate` adds
temperature, drought and dim light as upkeep outside a body's comfort. `--regard`
adds the lineages you come to know and their regard for you. `--exertion`
makes chosen acts past a body's ease spend matter. `--hosting` seeds a microbe you can hold
(it rots dead matter and enters bodies; 1 to 4 switch its rot, feed, leak and enter genes), `--turn N` lets a
holder of the world turn its temperature up to N degrees (needs `--climate`; 9 and 0 lower and raise the
shift), `--learning` lets members learn (it founds each deciding lineage a policy that scores as the hand-written score does, so members choose by policy, and weighs each kind of act as well as each need), and `--mass-law`
scales upkeep, meals, maturity, lifespan and birth spacing with a body's adult mass (the three-quarter and
one-quarter powers, held to a quarter and four times the flat rate), and
dormancy makes a producer's young lie dormant as seeds that cost a tenth of the upkeep and wake by season where
the site is not crowded, so a line a lean spell empties can come back from its seed bank. It is the crowding
brake a new generated A1 world founds with by default (since 2026-11-11, because without it one of the two
producers always dies in its boom and crash): `--no-dormancy` founds the world without it, and `--dormancy`
is still accepted as a no-op alias. Saves and candidate files made before keep the world they were made
with, and `--no-dormancy` and `--no-limits`, like the other world rules, are refused on a resumed save, spread, candidate or witness. Limits on
consumers are the second default (D27): each consumer, grazers, predators, omnivores and decomposers
alike, feeds only while it holds less than a set fullness and bears young only where fewer than a
ceiling of its line stand at its site, which stops a line outrunning its food. `--no-limits` founds the
world without them, and `--limits` is accepted as a no-op alias. Rot is the third default (owner's ruling, 2026-11-12): dead
bodies give a slow share of their matter (a fiftieth of what remains, at least one unit) to the site's soil, where producers draw it,
without waiting for a decomposer to eat them; it is the baseline, and decomposers do the same work faster (a decomposer feeds twelve times where it fed once, so they take about two thirds of the dead matter and rot the rest). `--no-rot` founds the world
without it and `--rot` is accepted as a no-op alias (like the other world rules, `--no-rot` is refused on a resumed save, spread, candidate or witness). Young grow from their line's current recipe in a new generated world, so a revision or drift a line took passes down to its young (they used to copy their parent, so it never did); `--no-inherit` founds the world with the copy and `--inherit` is accepted as a no-op alias (refused like the others on a resumed save, spread, candidate or witness; a saved world without the field keeps copying). `--spoilage` (off unless asked; refused like the others) makes a dead body's flesh fresh, then aged, then toxic with the time since it died, and lets a mouth eat what it does not tolerate (toxic flesh, a decomposer's flesh) to its harm instead of being refused. `--categories` (off unless asked; refused like the others) makes a plant's stalk, trunk and root tough, so a mouth with no grinding that bites them takes harm instead of being refused, and classes dead matter as litter, wood or carrion, which a mouth limited to some classes takes alone. `--guard` (off unless asked; refused like the others) gives defence and counter-defence: a part's armour resists a bite by the mouth's stronger attack (crush or pierce) and what it resists stays in the prey, a spined part wounds the mouth that bites it, and a body with glands harms what eats it, each unless the eater tolerates it or matches it; every one costs cells and a standing rent of matter, and with `--repertoire` lines reach armour, spine and gland parts by editing and drift. `--mutualism` (off unless asked; refused like the others) lets a producer line and a decomposer line that share a site trade soil for tissue, each paying only while the other does, so a line that stops giving stops receiving; with `--repertoire` the catalog's exchange parts lower as organs, though a decomposer line has no recipe to edit yet, so only plant lines take one. `--repertoire` (off unless asked; refused like the others) lowers the runnable catalog parts that act at the meal (mouths, tolerance organs) into kinds the world declares, which lines reach by editing and drift, at the played line's review and the unplayed lines' boundaries; no line is founded with one. D calls the
round, A extends a capped one, H forfeits when no body of the line is left,
and at a loss Tab moves among the heirs.

## License

Two-part boundary (see [LICENSES.md](LICENSES.md)):

- game code, repository documentation, and any promoted reusable library:
  MPL-2.0
- original game assets: CC BY-SA 4.0

The published `0.0.1` name reservation remains MIT OR Apache-2.0; the split
begins with `0.0.2`. A promoted reusable library no longer takes a separate
permissive grant, per Mark's 2026-09-03 ruling retiring the 2026-07-31
clause.

---

*This README was generated by AI and will be edited by the author upon
release.*
