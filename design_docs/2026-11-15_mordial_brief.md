# A rhythm game of territory, before Mesocosm: research brief (2026-11-15)

**Status, 2026-11-15:** research brief, opened by Mark in conversation. Not a
plan; nothing is scheduled and no code exists. "Mordial" is Mark's speculative
name, not banked in the naming ledger and not checked anywhere; this brief
uses it only as a working label. The answers below are in Mark's words and
are not yet numbered cases in the wing's record.

## The idea

Mark, 2026-11-15, condensed from the spoken pitch (his words where quoted):

- "A rhythm game" played "on a board, a giant board", with fog of war. "Your
  goal is to increase your territory of the board ... It's like Go in that
  way."
- "The world's tempo is set by how many things are in the world."
- Each kind of life has a base rhythm (one move a measure, two, three), which
  you specialize with "additional things that you do per measure ... maybe per
  4 bars even"; effects can be queued.
- Critters find resources on squares, "grow by exploring", and with what they
  find "create new versions of themselves" that "learn based on what you did
  your first round" and try to apply your strategy: "your progeny".
- "Little fights when two entities interact", or rock-paper-scissors by
  specialty. Examples given: a tree, a poisonous vine, a food that is eaten
  and carried as seed; a fungus that hijacks animals, grows under territory,
  or spreads in spore bursts "every so often, as opposed to incrementally".
- "2.5D ... isometric." Offered as a way to "limit the scope", under a
  different product identity or as "a pre-Mesocosm stage, primordial soup ...
  it can maybe provide seeds for creatures in Mesocosm."

## Where it meets the record

- Territory is Mesocosm's own win measure made visible: "growing your kind's
  share of the world's biomass" (`mesocosm/design_docs/PROJECT_DESCRIPTION.md`).
- Progeny that imitate the player are pillar 6 (dead is dead, lineage
  persists), and playing one body among them keeps pillar 1.
- Handing a founder to Mesocosm is "enrichment, never homework" (same file,
  Where it sits).
- The anti-Spore rule (founding record §2: no stage grows its own engine)
  holds only if this runs on Cosmograph's clock and the existing board.
- Mesocosm is the first-person vessel. A board seen from above is a different
  seat, which is why the placement answer below makes it a vessel of its own.
- "Flora" is spoken for platform-side and is never a game word
  (`mesocosm/CLAUDE.md`, Vocabulary). The pitch's kingdom names would need a
  naming round; the answers below drop kingdoms anyway.
- Cosmograph already has the vocabulary Mark reached for: `Part`
  (`shared/cosmograph/src/schema.rs`), `Effect` and the rules' `Process`
  (`shared/cosmograph/src/rules/`), and five part processes, `Contract`,
  `Intake`, `Sense`, `Fix`, `Secrete` (`shared/cosmograph/src/process/mod.rs`).
  `Kingdom` is a reading of the body, not a class.

## Answers so far

Each is the question, then Mark's answer as given (an option label where he
took one).

1. **Where it sits.** "New small vessel": a fourth, separately named game on
   Cosmograph's clock and the existing isometric board; Mesocosm stays as
   written and receives seeds.
2. **What the beat is.** Timing accuracy matters, "with generous ranges for
   latency (or we can cut accuracy for co-op/networked play, for balance)".
3. **Who you control.** "One body, progeny imitate": each progeny replays the
   player's recorded per-measure pattern with variation; on death the player
   continues as a descendant.
4. **Tempo.** Total population, capped, and at the cap: "an epoch shift, reset
   tempo to baseline, and zoom the map out so territories shrink relative to
   an expanded scale. All players upgrade their lineages, to specialize/change
   it, using one or some of the round's resources (or taking account of the
   consequences of the round's choices/accomplishments?). I would like the
   variation to be open ended and reflected in the critters… maybe i should be
   thinking in terms of parts, effects, processes, and roles, since this is
   primordial soup, not the meso scale of animals… kingdoms wouldn't be quite
   right for this. Really everything should range from the smallest life to…
   the starting point of mesocosm's kingdom individuation? Luca lines? Idk."
5. **Where rhythm comes from.** Read from the processes a body carries, and:
   "Maybe each trait has a time signature? What are the varieties of
   microbiota we might develop as predecessors to critters?"
6. **The epoch shift.** "Territory becomes your body": each lineage's
   territory collapses into one critter at the next scale, its shape and
   contents deciding that critter's parts.
7. **Upgrades.** "Deeds offer, resources pay": what the lineage did in the
   round decides which parts and processes are on offer; resources found on
   squares pay for them.
8. **Record.** This brief, here; "speculative name: mordial?"
9. **Meter.** Both a cycle length and a subdivision per trait, "because you
   toggle these on and off as you like as you navigate the board. Perhaps a
   trait like a stinger can apply to biota within 1 square but the sound it
   makes extends past the fog of war, so you wait until the creature is in
   range before you toggle it on and step close enough. Helps differentiate
   usual movement sounds from combat and the use of an ability."
10. **Small lineages at the shift.** "Play on, small": a small territory
    becomes a small critter at the new scale and the player continues.
11. **Epochs.** "Three, as a world setting": cell, colony, body by default,
    the count adjustable per world alongside board size.
12. **Cases.** "When a plan is cut": the answers stay here until this becomes
    work.
13. **What an active trait costs.** Missed beats cost, and being heard is the
    cost: "If you sound like the background you might be able to sneak up on
    something preoccupied. But attacks are rapid usually, which tips off
    people around you to what you are and can do."
14. **Who hears.** "Everyone hears; Sense sharpens": all bodies hear past the
    fog; a body with `Sense` also gets direction and distance, and hears
    further.
15. **Progeny.** "They copy the toggling": progeny switch traits on and off
    in the situations the player did.

16. **What the kept critter is.** "Record, with a .vox beside it": the source
    is the lineage record; a MagicaVoxel export is written next to it as a
    picture.
17. **What the record carries.** Body, history and rhythm, and: "I would say
    traits more than rhythm!" Read as: the record carries the traits, and a
    rhythm is what a trait has, not a separate thing to save.
18. **Edited records.** "Accepted, marked as edited": a record is signed by
    the run; an edited one still founds a lineage but shows as designed, not
    earned, and shared worlds may ask for earned ones.

19. **The start.** "Everybody beginning as the same one part cell or like
    smallest unit of life. That makes sense to me. And then you individuate?
    Yeah, that's pretty cool."

20. **Fights.** "Traits decide, the beat delivers": which trait beats which
    is a fixed relation, but a trait counts only on the beats it fires and
    the player hits. No separate fight mode.
21. **What a trait is.** "Cosmograph's, plus a meter": every trait is a
    Cosmograph trait or function; this game adds only its cycle length and
    subdivision; new traits go into the shared catalogue.
22. **The first difference.** On acting in epoch one: "it would be an
    additional protophase." On first find: "would not be reasonably
    distributed." On a pick: "works, but i wonder if we might not gamify
    that at the start to kick off the first round. But i don't know what
    would be a better sort than people picking. Maybe how quickly you want
    to make decisions; by acting once a bar, you would spread your territory
    with a range on that one beat, vs limiting your range to hit beats/move
    more frequently. If i was gonna give a choice, it would be that i guess;
    charge or tap tap tap tap?"

23. **Kingdoms.** "Mesocosm reads it from the record": this game never
    sorts; the record leaves kingdom empty and Mesocosm reads one from the
    grammar and processes when the lineage founds. Owed when a plan is cut
    (answer 12): a refinement of doctrine Bodies 6, that a kingdom can be
    read from a grammar as well as bound one. Doctrine is unchanged today.
24. **The first rule.** "Yeah, let people get a feel for moving around a
    test board and run a lil tutorial the first time they play, then they
    ready up, and the way they move in the opening 4 bars grants a trait
    related to their played tempo? Like how a metronome lets you do click
    timing to pick a bpm, no?"
25. **Deeds and the grammar.** "Yes, deeds are rules": each way of taking
    ground earns its operator, the offer at a shift is rules, the body
    unfolds from the grammar the territory earned, and the record carries a
    grammar.

26. **Uneven or absent opening bars.** "Even if somebody plays unevenly,
    they still produce a tempo. Like if you are clicking ... tap to tempo
    ... on any, normal, online metronome, you will, even if you click
    unevenly, you will still produce a tempo, right? And if you don't click
    at all, then you just get the lowest range of ... traits associated with
    the lowest range of tempos, essentially, which would probably be plant
    or sedentary or, like things that grow by not doing a lot and save their
    power in order to use it when they really, really need it, or to direct
    it all to growth, essentially, and not so much to, defense, except
    through characteristics of their bodies, like plants."

Unprompted, the same day (answered by 23 and 25): "I wonder if we could use the growth grammar to
organize the territory into recognizable body shapes at each tier." And: "I
wonder how we can sort things into the kingdoms. Or if we should try to do
that even. Should that be just like a mesocosm thing? A reading of this
game's export data? I don't know."

**The keepsake** (Mark, 2026-11-15, unprompted; answers 16 to 18 rule on it): "at the end of a
playthrough, you get that critter ... you can keep it, and it's data, and you
know it's explicit that it's like yours, and you can do whatever you want
with it. But you know, hopefully we'll be able to use it in our upcoming
game, Mesocosm ... Should this be like ... a Magicavox file or something? ...
maybe a voxel, a microvoxel presentation, kind of like squashed, like
perspective squashed, but you know, 3D modeled ... What is produced at the
end of this primordial soup?"

## Proposed, not ruled

**The arc.** Everyone starts as the same one-part cell and individuates
(answer 19). Each epoch is one major transition: cell, colony, body (answer
11). Proposed: the last
epoch ends where Mesocosm's founders begin, so a finished run is a founder.

**Meter** (answer 9 took both). Subdivision: bodies share the measure and
divide it differently (two against three). Cycle length: bodies share the
pulse and each trait repeats every N beats, so a 3-beat trait and a 4-beat
trait fall together every 12, and "something happens after 4 bars" falls out
of the arithmetic.

**What answer 9 implies.** Traits are voices the player switches on and off,
and a voice is heard further than it reaches. So sound is the sense that
crosses the fog: a body is known by its rhythm before it is seen, an active
trait announces itself, and silence is a choice. Movement and abilities
sound different, so the mix says what is nearby and what it is doing.

**Microbial forms**, each a way of making a living that an existing process
already names:

| Form | Process | How it takes ground |
|---|---|---|
| Light-eater (cyanobacteria) | `Fix` | Stands still and lays a mat; slow, every beat counts |
| Rock-eater (vent chemotrophs) | `Fix`, tied to mineral squares | Rich only where the square is; holds points, not areas |
| Absorber (bacteria, early fungi) | `Intake` of the dead and dissolved | Threads through what others leave; grows under territory |
| Engulfer (amoebae, ciliates) | `Intake` plus `Contract` | Moves and eats neighbours; ground follows the body |
| Swimmer (flagellates) | `Contract` plus `Sense` | Fast scout; finds resources first, holds little |
| Stinger | `Secrete` | Makes its ground costly to enter |
| Rider (viruses, parasites) | none of its own | Lives on a host's beats; spreads when the host does |

Forms of company, which are what the epoch shift acts on: film or mat
(contiguous ground), filament (lines), swarm that gathers on a cue (the slime
mould: a burst every N bars), and spore (sits out measures, survives a
reset). An engulfer that keeps what it swallowed instead of digesting it is
the origin of the mitochondrion and the chloroplast, and is kleptoplasty at
this scale: it gains the kept one's process and its rhythm. That is pillar 2
(every part used to be somebody) before there are parts.

Three roads lead to Mesocosm's three readings: mat-forming light-eaters to
producers, engulfers to consumers, thread-forming absorbers to decomposers.
Mixed roads are the interesting ones.

**What a run produces** (proposed against the keepsake). Bodies in the wing
are grown from a recipe, not modelled: a founder is a recipe, a method and a
development seed (`eponym/crates/eponym-play/src/founding/spread.rs`), a body
is a document plus parts carrying processes (`shared/cosmograph/src/
geometry.rs`), and voxels are what development and `isometer-mesh` make of
them. So the thing kept is a lineage record: the recipe and seed, its traits
(each with its meter), the lineage's descent and deeds, and the player's
recorded pattern. A MagicaVoxel `.vox` is a
picture of it, exported beside it; the VTT's appearance layers already read
`.vox` (`crates/isocosm-views/src/appearance.rs`). Doctrine World 8 already
admits the record: "a player's designed lineage" may found in a world, down
to one member. The same statement says micro life becomes "territories and
surfaces", which is this game's board.

Uses for the record: a founder in Mesocosm; a denizen or token on the VTT; a
stranger arriving in someone else's world (Mesocosm's project description,
Speculative); its rhythm as the lineage's motif wherever it appears.

**The growth grammar is the game** (proposed against Mark's question).
Doctrine Bodies 6 makes a lineage's recipe a growth grammar over one set of
operators (repeat, branch, differentiate by position, symmetry, fuse, switch
on a condition, accrete, allocate), where "an ingredient adds a rule or tunes
one at a step of growth". A `Grammar` with `Repeat`, `Branch` and `Allocate`
rules already unfolds a plant (`shared/cosmograph/src/bodied/tips.rs`);
fuse, switch, accrete and radial symmetry are not built. Each way of taking ground is one operator
played: a line is repeat, a fork is branch, a contiguous mat is accrete,
mirrored spread is symmetry, engulf-and-keep is fuse, toggling by situation
is switch, spending is allocate, playing regions differently is
differentiate by position. So at a shift the round's deeds are the
ingredients, the offer (answer 7) is rules, and "territory becomes your
body" (answer 6) means the body unfolds from the grammar the territory
earned, not from the territory's outline. That is what makes the shape
recognizable, and it makes the kept record's recipe a grammar Mesocosm can
keep growing.

**Charge or tap** (answer 22) is then the grammar's first rule and the
cell's first meter at once: one long-reaching act a bar (accrete), or a
short act on every beat (repeat). Answer 24 makes the opening four bars the
pick, as tap tempo: how often a player acts in them sets the first trait's
subdivision, so the choice is a range (one act a bar to one a beat or
finer), with reach falling as rate rises, not a pair. A test board and a
first-time tutorial come before the ready-up.

**Kingdoms.** Case 1219 retired the enum; a lineage's kingdom is now a key,
"empty where none is known" (`shared/cosmograph/src/meaning/mod.rs`, case
1232). So the record can leave it empty. Bodies 6 has kingdom weighting and
bounding the grammar; here the grammar comes first and a kingdom would be
read from it, which runs the other way and bears on that statement.

## Open

- The operators the simulator has not built: fuse, switch, accrete, and
  radial and no symmetry. Repeat, branch, differentiate, allocate and
  bilateral symmetry are in (`shared/cosmograph/src/rules/grammar.rs`).
  Building to eight is already ruled and owed
  (`mesocosm/design_docs/2026-11-09_lineage_rounds_plan.md`, D29, LR6); this
  game depends on that work and does not own it. The operators' meanings
  are "a reading until built" (the lineage rounds brief, §4).
- Mesocosm's microbes are already played by gene switches that cost upkeep
  while on (the same plan, D30, LR8). Answer 13 here prices a trait by
  missed beats and by being heard instead; the two need reconciling.
- The steps on the opening bars' rate range, and which traits sit at each
  (answer 26 settles uneven play and silence: any taps average to a tempo,
  and none gives the slowest, sedentary traits).

- What a trait is here against Cosmograph's own: a part's `traits` and
  `functions` keys (`schema.rs`), or something this game adds.
- What Mesocosm does with a trait's meter, having no beat of its own.

- What a fight is when two bodies meet on a beat.
- What "imitate with variation" records: inputs, or a pattern per measure,
  and whether progeny copy the toggling too.
- What an active trait costs while it is on (pillar 5's upkeep, here).
- How far each voice carries, and whether hearing is itself a trait (`Sense`).
- The name.
