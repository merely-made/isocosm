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

## Found a world in Mesocosm

Launch `mesocosm-genet` with no arguments, choose **World and lineage editor** on
the native bench, or press **E** at the game's **New World** screen. An explicit
launch is `mesocosm-genet --founding-editor --seed 104`.

**World conditions** sets the seed, population, cohort size, starting stock
range, climate preset and water level. The starting map is a four-site plane.
**Use a planet** folds a six-face grid into a sphere. You can also load a
Cosmograph founding TOML. **Lineage body maps** selects a lineage and a growth
run. Drag an attached run around its parent to change the direction it grows.
Add branches, edit repeats and variation, and attach native tissues and
admitted catalogue parts. **Load body map** takes a single lineage TOML.

**Project founding** grows the founders and opens the world at tick zero.
Select a site and **Watch time pass**, or advance and rewind by one or twelve
ticks. Select a joint on the developed specimen to inspect its tissue.
The population rows group the inhabitants by their realized segment counts
and omitted organs. The matter readings come from native process transfers.

**Save projection** writes the selected tick, the founding TOML and a reusable
TOML for each authored lineage under `~/Mesocosm/worlds/` (override with
`MESOCOSM_WORLD_DIR`). Paste its `entry.json` into **Template or save path**
and choose **Reopen projection** to scrub it again. **Enter this world**
opens Mesocosm watching the selected site and tick. The game writes its
continuation to `world.json` in that entry's directory. Open it with
`mesocosm-genet --watch <world.json>`.

## Found a plant lineage

**Compare frond forms** opens two ordinary foundings on a shared time slider.
The plant founders have the same IDs, bodies and starting stocks in both.
The second world also stocks grazers. Edit the voxel dimensions, then choose
**Grow worlds** to try a body map. Its partition rule must fit nine fixing
cells and the same total frond volume into one blade or three. At each tick
the charts show plant counts and the tables report exposed cells, cells
backed by local plant tissue, soil fixed, births and biomass eaten. Each
world can be saved at the selected tick. The default map is in the
[frond allocation template](../shared/cosmograph/examples/frond-allocation.toml).

The [browser comparison editor](tools/lineage-editor/) opens the ordinary
[optional-gland founding template](../shared/cosmograph/examples/optional-gland.toml).
The initial world lays four mapped sites and gives the grazer lineage the native
directing rules. Choose reactive grazers for a mapped ecology without directing, or
**Single site** to reproduce the original unmapped competition.
**Attachments** draws one numbered node per part the selected founder grew.
Edges follow the physical attachments. Select the same part in either view to
inspect its parent, tissue and growth rule. Drag a side stem to a base, middle
or tip socket, or drag a frond above or below its stem. The inspector offers the
same edits by keyboard. Edits apply to the lineage and regrow all founder bodies.
**Growth rules** shows how the template produces the selected anatomy.
The defensive organ offers plant tissues from the native catalogue with the
spine and toxin strengths read by the current rules. Repeat controls the stem
length, its seeded variation and whether initial bodies are drawn in pairs or
individually. Turning the specimen, flexing its fronds and playing the sway preview are view controls.

**Survival check** reads the selected founder's fixing tissue, canopy and site.
It reports the selected form's head count, births, soil converted, upkeep and
grazing over 35 native ticks. Three additional seeds sample the same edited
founding with different founders and habitats. In this template, moving all fronds
below the stem leaves the plant without exposed fixing tissue. Light-driven
growth then stops while upkeep and grazing continue. One exposed fixing plate
allows an act at the configured rate if the site can supply its light and soil.
The founding template enables this shared rule with `bodies.canopy_growth`.
The competition timeline scrubs the chosen world independently of these trials.
**Save world at tick** downloads a Cosmograph save at the selected tick.
**Export template** downloads the TOML for founding that world again.
**Enter in Mesocosm** opens a new game window in Watching at the selected tick.
Entering appends one native Join. The entry and the game save on exit have
separate paths under `~/Mesocosm/worlds/`; pass `--world-dir` to choose another.
The native New World editor and this browser comparison can both enter an ordinary founding. The browser editor
uses the native executable supplied to the launcher.

**Grow a resident pocket** opens a two-founder experiment. Switch the fungal
lining or the resident's exchange organ on or off before founding. Scrub both
paths together to inspect their anatomy, stocks and cumulative soil, sugar and
carrying costs. The departure removes their contact on one path. Both keep
living on the same patch. Either timeline can be saved or entered at its
selected tick, including the founding contact and any departure.

From the umbrella repository in the local Code workspace, run:

```sh
python3 /Users/markik/Code/tools/build_storage.py build --lane cosmograph-world-time \
  --cwd "$PWD/shared/cosmograph" -- build --release --locked --offline \
  --example lineage_preview --example pocket_preview --bin cosmograph-bench
python3 /Users/markik/Code/tools/build_storage.py build --lane mesocosm-scene-composition \
  --cwd "$PWD/mesocosm" -- build --release --locked --offline -p mesocosm-genet --bin mesocosm-genet
python3 mesocosm/tools/lineage-editor/serve.py --open \
  --game /Users/markik/Code/targets/lanes/mesocosm-scene-composition/artifacts/release/mesocosm-genet \
  --bin-dir /Users/markik/Code/targets/lanes/cosmograph-world-time/artifacts/release
```

Elsewhere, build the same two release targets in `shared/cosmograph` and pass
Cargo's release directory to the Python launcher. A game started independently
uses the running editor or looks beside its executable for the founding tools.
Set `MESOCOSM_EDITOR_BIN_DIR` if they are built separately. A bare founding
save can also be opened with `mesocosm-genet --enter-world <entry.json>`.
Resume the game's subsequent `world.json` with `--watch`. The editor runs on loopback
and requires Python 3 and a browser. Reopen a downloaded world with
`cosmograph-bench --load <save.json> --ticks 18 --output <continued.json>`.

### Root and litter trial

The seed 9 root trial divides 24 units of phosphorus between a plant's root
zone and litter accessible to the fungus. It forks at tick 6 and records each
partner through tick 24 with and without contact. From the repository root,
choose an empty output directory for the saves and report.

```sh
trial=/path/to/root-trial
python3 /Users/markik/Code/tools/build_storage.py build --lane cosmograph-world-time \
  --cwd "$PWD/shared/cosmograph" -- run --release --locked --offline --example root_patch -- "$trial"
python3 mesocosm/tools/lineage-editor/root_report.py "$trial"
```

Open `index.html` in that directory to scrub the forks, compare light and
phosphorus supply, and open either native save.

## Use

```sh
cargo run -p mesocosm-genet    # play in a window over a native Cosmograph session
cargo run -p mesocosm-genet -- --bench --seed 7 # the specimen bench over a native session
cargo run -p mesocosm-genet -- --help   # every flag and key
cargo test --workspace
```

Click or press E/Space to point something out. Right-click a visible body or
site to ask your critter to move toward it, move away, or stay near. Q opens
the same menu at the critter's site. Choose **Suggest an act** to see the
alternatives Cosmograph currently offers that creature and their targets.
Use Up/Down to choose and Enter to confirm. The creature weighs a request
by its bond to your player. By default, “near” means within one route hop,
and a direction lasts up to 60 simulation ticks or until you replace it.

In a newly founded playable world, each deliberative critter remembers where
it last ate and where it last got hurt. A hungry animal can head back to a
familiar feeding site. Once fed, it favors staying there. An injury gives
it a reason to leave and avoid returning. The animal weighs those memories
alongside its other needs even when nobody is directing it.

S speciates, arrows pan, Z/V rotate, C switches surface and section, Y cycles
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
without it and `--rot` is accepted as a no-op alias (like the other world rules, `--no-rot` is refused on a resumed save, spread, candidate or witness). Young grow from their line's current recipe in a new generated world, so a revision or drift a line took passes down to its young (they used to copy their parent, so it never did); `--no-inherit` founds the world with the copy and `--inherit` is accepted as a no-op alias (refused like the others on a resumed save, spread, candidate or witness; a saved world without the field keeps copying). `--spoilage` (off unless asked; refused like the others) makes a dead body's flesh fresh, then aged, then toxic with the time since it died, and lets a mouth eat what it does not tolerate (toxic flesh, a decomposer's flesh) to its harm instead of being refused. `--categories` (off unless asked; refused like the others) makes a plant's stalk, trunk and root tough, so a mouth with no grinding that bites them takes harm instead of being refused, and classes dead matter as litter, wood or carrion, which a mouth limited to some classes takes alone. `--guard` (off unless asked; refused like the others) gives defence and counter-defence: a part's armour resists a bite by the mouth's stronger attack (crush or pierce) and what it resists stays in the prey, a spined part wounds the mouth that bites it, and a body with glands harms what eats it, each unless the eater tolerates it or matches it; every one costs cells and a standing rent of matter, and with `--repertoire` lines reach armour, spine and gland parts by editing and drift. `--mutualism` (off unless asked; refused like the others) lets a producer line and a decomposer line that share a site trade soil for tissue, each paying only while the other does, so a line that stops giving stops receiving; with `--repertoire` the catalog's exchange parts lower as organs, though a decomposer line has no recipe to edit yet, so only plant lines take one. `--detritus` (off unless asked, needs `--categories`; refused like the others) makes an animal's meal leave a share of what it took at its site as dung, a dead stock of its own that a dead-stock mouth not limited to other classes (a decomposer's root, a lowered dung mouth) takes and that rot returns to the soil. `--nutrients` (off unless asked; refused like the others) makes the ground hold two nutrients that a site differs in, which a producer needs in a fixed proportion that differs by line, so it fixes only as far as the scarcer allows and declines by upkeep where one is short; what returns to the ground is sorted back in the proportion it was drawn, and with `--repertoire` the scarce-uptake organs lower as parts that draw the nutrient a site holds least of. `--repertoire` (off unless asked; refused like the others) lowers the runnable catalog parts that act at the meal (mouths, tolerance organs) into kinds the world declares, which lines reach by editing and drift, at the played line's review and the unplayed lines' boundaries; no line is founded with one. D calls the
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
