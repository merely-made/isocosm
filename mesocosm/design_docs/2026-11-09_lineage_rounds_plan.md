# Lineage rounds: the loop, kingdoms and their play

**Status:** 2026-11-09: plan, cut from the
[lineage-rounds brief](2026-11-09_lineage_rounds_brief.md) (cases 1198 to
1243). Nothing here is built except the trophic rename (1222, logged in the
sim plan). The first slice is LR1 to LR4 and its headed check LR4h (D1). It
stands on the overlay plan's interim loop (M4i.1, M4i.2), which is built and
awaits certification.

**For:** building what the brief ruled, in ten phases: the round's own rules,
the boundary's drift, the player's draft, the systems sheet, discovery and
regard, the growth grammar and kingdom weights, abilities and their routes,
microbe and world-class play, shared lineages, and scale crossings with the
candidate kingdoms. Code paths are under `shared/cosmograph/` unless a
product is named.

Not here: the interim and full played loop, directing and the epoch
boundary's certification (`mesocosm-overlay`, whose EB3, EB6 and EB7 this plan
extends and cites); the sim's ecology phases and tuning (`sim`); speed bars
and the coarse step (`sim-execution`); the tagmata recipe's footing, senses
and binding (`bodies`); sessions between players (`session-network`, root).

## Decisions

Mark's answers, keyed `lineage-rounds D1` onward in the order he gave them.

- **D1, the first slice (2026-11-09), applies Mesocosm 6:** "The loop: rounds,
  drift, draft, sheet (Recommended)". LR1 to LR4 are built first, solo, on
  today's bodies and the interim loop, so a round is played, lost, drifted,
  drafted against a sheet and played again before the growth grammar exists.
  Offered against the growth grammar first and a second kingdom first.

  Links: area Mesocosm · draws on 1200, 1215, mesocosm.6 · summary The first slice is the solo loop on today's bodies: rounds, drift, the draft and the sheet, before the growth grammar.

- **D2, drift and draft together (2026-11-09), applies Ecology 1:** "Picks
  compete in the copy (Recommended)". The player's drafted remaps are
  candidates beside the averaged template; the forward copy scores each, and
  the player chooses with those scores in view, so selection can still say a
  pick is bad. Offered against picks applied after the drift, unscored, and
  against choosing per round.

  Links: area Ecology · draws on 684, 1200, 1208, ecology.1 · summary The player's drafted remaps are scored by the forward copy beside the averaged template, and the player chooses with the scores in view.

- **D3, the sheet's tolerance axes (2026-11-09), applies Mesocosm 6:** "Add
  temperature now". LR4 grows a temperature field in the sim, beside the
  light, water, soil, air and weather the world already has, since it is the
  axis players expect most. Offered against today's fields first and against
  light, water and soil only.

  Links: area Mesocosm · draws on 1215, mesocosm.6 · plans sim · summary The sheet's tolerance axes are today's fields plus a temperature field the sim grows for it.

- **D4, the order of heirs (2026-11-09), refines Mesocosm 1:** "You choose
  from the list". Heirs are listed descendants first, nearer generations
  first, then lineage-mates by fewest route steps from where the critter
  died, closer relatives breaking ties; the player may take any of them, and
  the first is the default. Offered against that order with no choice, and
  against relatedness before distance.

  Links: area Mesocosm · draws on 1204, 1217, mesocosm.1 · summary Heirs are listed descendants first, then lineage-mates by route distance with relatedness breaking ties; the player may take any, the first by default.

- **D5, the body a round starts in (2026-11-09), applies Mesocosm 3:** "1 or
  2", of a member already there with a journey to the fringe, or a critter
  planted from the lineage's budget; a member there only was the third. Left
  to the plan: LR1 takes the first, since the sim has no lineage budget yet.
  In range the player takes a living member at the place; one step past it
  they take the nearest member, which sets out on a journey there, paying
  upkeep on the way. Planting follows with the shared budget (LR9, 1221).

  Links: area Mesocosm · draws on 1211, 1221, mesocosm.3 · summary A round starts in a living member at the place, or in the nearest member journeying to a place one step past the range; planting waits for the lineage budget.

- **D6, the fringe start (2026-11-09), refines Mesocosm 3:** "some stuff
  probably can't move and you gotta take it where it is. Otherwise, 1, but
  then maybe nudge several, like a cohort of related critters". Asked because
  the sim has no command that sends a critter anywhere, only the nudge. A
  fringe start is offered only where a member can walk to the site; a body
  that cannot move starts where it is. The start nudges toward the site,
  the critter's going its own choice, and nudges a cohort of related
  critters with it. Offered against a guaranteed journey by a new command,
  and against no fringe until planting.

  Links: area Mesocosm · draws on 1211, mesocosm.2, mesocosm.3 · summary A fringe start exists only where a member can walk there; it nudges the taken body and a cohort of its kin toward the site, their going their own choice.

- **D7, three ways a round runs past its span (2026-11-09), refines Record
  7:** "I'd say 1 is default for co-op, 2 is a continuous play mode (if you
  want to keep playing for multiple spans), and 3 is a capped mode." Waiting:
  no lineage adapts until the players call the round, the default in
  company. Continuous: unplayed lineages take a boundary turn every span and
  only the player's own edit waits for their call. Capped: the round ends at
  the span unless everyone agrees to extend it by another.

  Links: area Record and history · draws on 1199, record.7 · summary A round runs past its span in one of three modes: waiting for the call (the default in company), continuous with others adapting each span, or capped at the span unless extended.

- **D8, the first condition (2026-11-09), applies Mesocosm 3:** "Beat your
  own forward copy (Recommended)". The condition is to hold more matter than
  the played lineage peaks at in a forward copy left to itself over the
  span. It is verified because the copy reached that mark; the round ends the
  moment the lineage passes it. Offered against growing by a set share and
  spreading to a new site.

  Links: area Mesocosm · draws on 1199, 1212, mesocosm.3 · summary The first round-ending condition is holding more matter than the lineage's own unsteered forward copy peaks at over the span.

## Constraints

- Verified by draws, on seeds nobody chose (ruling 15); every phase replays a
  seeded run to the same hash.
- New rules are optional (Schema 10): a world without a phase's rule hashes
  as it did before the phase, each phase's negative control.
- No stage grows its own engine (founding record §2). Rounds, drift and
  regard live in the sim; Mesocosm lowers envelopes and shows readings.
- A reading is a pure function of state and log, the same from a replayed
  run as from the live one.
- The slice's acceptance is the standing caveat: the game must make somebody
  want another run. LR4h puts it to Mark, headed.

## Phases

### LR1. Rounds: start, succession, end
A round over the interim loop (M4i.1) with the brief's rules. A player picks
a lineage they know and a start where it has members or one route step
beyond (1211). At a death the next life is the critter's offspring, else its
nearest kin left in the round, else the nearest living member of the line
however far (1204, 1217). A round ends when the players call it after a span
of world time agreed beforehand, or when a condition is met (1199); a player
whose line has no living member waits, and may offer to forfeit or end it
(1217). The edit lands on what survives (1213). The first cut offers one
condition: to hold more matter than the lineage's own forward copy, left to
itself, peaks at over the span (D8; 1212, EB3's stop rule). A round runs past
its span in one of three modes: waiting for the call, continuous, or capped
(D7). Solo, the agreement is the player's own call.
It is built in four steps:
- **LR1.1, succession.** One reading of heirs in the order D4 gives, shared by
  the sim's loop and Mesocosm's checkpoint, which today each keep their own
  (lowest id at the same site, then anywhere).
- **LR1.2, the start.** A pick of lineage and place, refused outside the
  range and its one-step fringe; the body is D5's. Today the lineage is fixed
  at founding and the first life is whichever member a habitable site holds.
- **LR1.3, the round's end.** The call after an agreed span, the first
  condition, waiting out with a forfeit, and D7's three modes. Today the
  boundary fires on the epoch's tick count and nothing else ends one, which is
  the capped mode without its extension. The sim's loop carries the epoch's
  ending first; Mesocosm's saved loop follows, with a save and replay path.
- **LR1.4, the edit on survivors.** Controls only, and owed: a revision already changes
  the lineage's recipe, not its members, and is refused when no body of the
  line is left.
- Done when, headless, by draws: a start outside the lineage's range and its
  one-step fringe is refused with a reason; a planted death hands on to
  offspring, to kin and to a far member in that order, one control each; a
  round ends at the call after the span and at a met condition, and does not
  end when near kin are gone while a far member lives; with no member left
  the player waits and a forfeit ends the round; an edit after local loss
  commits to a planted far population and to a dormant propagule, and with
  neither is refused; receipts replay.
- Needs: `mesocosm-overlay` M4i.1; `sim` S10.2 propagules (1197) for the
  dormant control; the known-lineage reading comes with LR5, so until then
  every lineage is known.
- Rulings: 1199, 1204, 1211, 1212, 1213, 1217, D4 to D8; Mesocosm 1, Mesocosm 3

### LR2. Drift: the success-weighted average
Before the forward copy weighs candidates, a lineage's template moves to the
average of its members' variation, each weighted by reproductive success
(1208). Members share one recipe today, so the variation averaged is what a
member already carries: its drawn soma (segment counts, absent appendages,
Bodies 6) and its policy draw (M3.5). Extends `lineage/boundary.rs` and the
overlay plan's EB7.
- Done when: with no variation among members a boundary adopts exactly what
  it adopts today (hashes equal, the negative control); a planted variant
  whose bearers leave more offspring moves the template toward it in
  proportion, and one whose bearers leave no more does not; the forward copy
  then scores candidates from the moved template; the played critter's
  weight is its own reproductive success, no more; cost is measured per
  boundary at a few hundred members.
- Needs: `mesocosm-overlay` M3.5 (built), EB7
- Rulings: 684, 1208; Ecology 1

### LR3. The player's draft
At the played line's review (`lineage/review.rs`), the offers are remaps
drawn from what the line incorporated that round, beside the status quo
(1200). What a meal yields follows the eater's kingdom, its kinship to the
eaten and the ingredient's familiarity to its recipe (1203). Picks are
bounded by an upkeep budget. The picks are candidates in the forward copy
beside the averaged template, and the player chooses with their scores in
view (D2). Creative mode offers any remap the kingdom's
bounds allow; until LR6 that bound is today's lexicon. `lineage/revise.rs`
already makes variants bearing a learned kind.
- Done when: after a round the review offers only kinds the line
  incorporated in it, an unplayed line gets no draft, and a line that ate
  nothing new is offered the status quo alone; each offer carries the forward
  copy's score, and a planted bad pick scores below the status quo; three planted controls show
  yield moving with kingdom, with kinship and with familiarity, each alone;
  picks past the upkeep budget are refused with the shortfall named; creative
  mode's wider offers appear only in creative mode; a committed draft
  replays.
- Needs: LR2
- Rulings: 1200, 1203, D2; Ecology 1, Mesocosm 6

### LR4. The systems sheet, with previews
A lineage's sheet is read off a grown body, never summed from the recipe
(1215): needs (upkeep by body size, particular matter), abilities (speed,
reach, force, senses, fecundity, dispersal) and tolerance curves. The
tolerance axes are the fields the world has (light, water, soil, air and
weather) and a temperature field this phase grows (D3). A field names its
consumer, dimension, cadence, sources and sinks, conservation rule and scale
reduction before it is admitted (this index's working principles). At the review each offer shows a diff, upkeep and mass as caps, and
each ingredient previews its contribution by growing the candidate body, not
by a forward copy.
- Done when: a sheet reads the same from a replayed run; it changes when the
  body's revision does and not otherwise (405); over draws, a preview's
  upkeep and mass equal the grown body's after commit; an offer past a cap
  shows which cap; a temperature field is admitted with its six facts named,
  a world without it hashes as before, and a body's performance follows its
  temperature curve in a planted warm and cold site; Mesocosm's host shows
  sheet and diff at the review.
- Needs: LR3; `sim` rates from body size (ecology brief 3.1) for upkeep;
  `place-graph-engine` field admission for temperature
- Rulings: 1215, D3; Mesocosm 6

### LR4h. The loop, played
Three rounds in the headed host, solo, on a seed nobody chose: pick, play
under the nudge, lose critters and continue, end a round, read the drift,
draft against the sheet, start again.
- Done when: the three rounds run end to end with receipts replaying, and
  Mark has played them and said whether he wants a fourth. His answer is
  recorded as a case and decides what follows.
- Needs: LR1 to LR4; `mesocosm-overlay` M4i.2 (the headed switch)
- Rulings: D1; the README's standing caveat

### LR5. Discovery, regard and the persona
A reading of the lineages a participant knows: outpaced in biomass,
incorporated, forked from their own, or met (1210). Any known lineage may be
watched. A lineage's regard for a participant rises with the benefit their
lineages give it and falls with harm; it sets each new critter's starting
bond, which served nudges then grow (1214). Regard is heritable within a
world-rule bound. The participant is keyed to the player's dramatis
identity, minted where absent (1227).
- Done when: the known-lineage reading is the same from a replayed run and
  each of its four routes has a control; benefit raises regard and harm
  lowers it, over draws; a new critter's bond starts from its lineage's
  regard, and with no regard and no bond a nudge moves nothing (the negative
  control); regard passes to descendants within the bound; a participant
  carries the same identity into a second world.
- Needs: `mesocosm-overlay` M3.1 (the bond); the dramatis facade; the
  Questions' regard item
- Rulings: 1210, 1214, 1227; Mesocosm 2, Directing 4

### LR6. The growth grammar and kingdom weights
The recipe becomes a growth grammar over one set of operators (repeat,
branch, differentiate, symmetry, fuse, switch, accrete, allocate), weighted
and bounded by kingdom (1202, 1206); a kingdom makes a function cheap or
dear and gates none (1232). The tagmata recipe is the segmented animal
preset. An ingredient adds or tunes a rule at a step of growth. This phase is
the largest; when it starts it may be cut into the bodies plan.
- Done when: the tagmata recipe, expressed as a preset, grows the eight
  authored bodies and a founding draw's bodies as it does today (hashes
  equal); a second preset, a plant grown from its tips, grows by repeat,
  branch and allocate; a function dear to a kingdom costs more and is never
  refused; a planted ingredient adds a rule at a step and a shift in when a
  rule fires changes the grown body.
- Needs: `bodies`; World 8's founding draws; the Questions' operators item
- Rulings: 1202, 1206, 1232; Bodies 1, Bodies 3, Bodies 6

### LR7. Abilities and their routes
An active move exists where the body's functions and systems can perform it
(1233). Ordinary activity is paid inside upkeep, and exertion past a
threshold the body sets spends matter or energy (1234). The ten routes to an
ability keep in three tiers: grown, borrowed and learned (1240 to 1243).
- Done when: a body with the parts performs a move and one without is
  refused with the lack named; activity below the threshold moves no account
  beyond upkeep and above it spends, the threshold rising with efficiency; a
  borrowed ability ends when its part or partner is lost; a watched act is
  imitated and an organ never is; a learned act passes by teaching at the
  share 1058 sets.
- Needs: LR6 for grown routes; `mesocosm-overlay` M3.6 for imitation; LR5 for
  symbiosis held by regard
- Rulings: 1233, 1234, 1240, 1241, 1242, 1243; Bodies 3, Ecology 7

### LR8. Microbe and world-class play
A microbe lineage is played by gene switches its population carries out and a
draft of one-shot plasmids from contact; its choices fall due at host and
patch events and its spread is watchable live (1223, 1224). Any strain may
decompose, infect, aid, join or disperse (1225), and inside another player's
critter either may help or harm (1226). A world-class lineage is played from
the aggregate view between rounds: time flow, cycles and events the sim's
conditions raise (1216).
- Done when: a switch changes what a strain expresses at every place and
  host it holds, at crowd grain; choices are raised at the named events and
  nowhere else; a plasmid is gained by contact and lost without selection; a
  mutualist strain raises its host lineage's regard and a pathogen lowers
  it; a world-class change to a cycle is a logged native command and its
  events are a reading of sim conditions; both replay.
- Needs: surfaces and strains (built at site grain); `sim-execution` E4 (the
  crowd); LR5; the Questions' microbe item
- Rulings: 1216, 1223, 1224, 1225, 1226; Bodies 1, Mesocosm 1

### LR9. Shared lineages and adopt or branch
Players may share a lineage or play their own (1221). Sharing one, they
plant its critters, add to its budget and add edits to its gene pool, and
individuating a played critter draws on that budget. Each edit proposes a
child revision: matching edits merge and conflicting ones fork the line, the
drift carrying a branch nobody claims (1205). Extends the overlay plan's EB6.
- Done when: two participants' matching edits commit one revision and
  conflicting ones leave two lines, each with its proposer; an unclaimed
  branch drifts; a shared budget is spent by individuation and refuses past
  empty; a player waiting out a round may offer to forfeit and the others'
  agreement ends it; all of it replays between two hosts.
- Needs: LR1, LR3; `session-network`; `mesocosm-overlay` EB6
- Rulings: 1205, 1217, 1221; Ecology 1, Mesocosm 6

### LR10. Crossings and the candidate kingdoms
A lineage changes scale by a major transition at a boundary (1209). The
colony (1228 to 1231) and the parasite or symbiont (1236 to 1239) are
designed; composite, replicator, idea, spirit and construct are candidates
only (1220). Each kingdom is cut into its own phases when it starts.
- Done when: not stated; owed, per kingdom
- Needs: LR6, LR7; LR8 for crossings from microbe
- Rulings: 1209, 1220, 1228 to 1231, 1236 to 1239; Bodies 17, Mesocosm 7

## Questions for Mark

From the brief's §8, each naming the phase it holds up.

- **Temperature's shape** (LR4): what drives it (latitude, height, season,
  weather) and at what grain, when the field is designed.
- **Regard's bound** (LR5): how far regard is heritable.
- **The operators** (LR6): whether eight is the set, and how a crossing
  unlocks another kingdom's.
- **The microbe's numbers** (LR8): which switches and plasmids exist and
  what each costs to carry.
- **Conditions** (LR1, later cuts): which kinds follow the first threshold
  condition, and how the generator is bounded (EB3).

## Findings

- 2026-11-09: the interim boundary skips the played line, whose turn is the
  review (`lineage/boundary.rs`, `lineage/review.rs`); other lines commit at
  once, from the most metabolically complex down. LR2's average therefore
  applies to both paths before either scores.
- 2026-11-09: `lineage/revise.rs` already offers a variant bearing a learned
  kind the recipe does not name, on one tagma. LR3's draft narrows that to
  kinds learned in the round and adds yield.
- 2026-11-09: members of a lineage share one `Development` recipe; what
  varies per member is the soma drawn by its seed and the policy's one drawn
  number (`directing/policy.rs`). LR2 averages those.
- 2026-11-09: journeys start only as a process's `Effect::Move` or
  `MovePaced`, through `journey::depart`; no `Command` moves a critter. A
  player's way to send one is `Command::Nudge` toward a site
  (`directing/mod.rs`).
- 2026-11-09: the world's fields in code are soil, ground, weather,
  habitable, food, water and air, with light as `sim:energy`. None is
  temperature.

- 2026-11-09, the interim loop as LR1 finds it: succession is
  `BorrowedInterim::next_life` (`directing/interim/borrowed.rs`) and,
  separately, Mesocosm's `succession::heirs`; `sim:parent` and `sim:child`
  are recorded at births and read by neither. The played lineage is fixed by
  `Founding.played`, and `first_life` takes a member at a habitable site or
  the first anywhere. There is no round type: `Pace` counts ticks and the
  boundary fires on the epoch's tick count. `Command::Revise` replaces the
  lineage's development, and `review::offers_using` refuses with "no body of
  the line is left". No dormant or propagule state exists in code.

## Progress

- 2026-11-09: LR1.3 built, in the sim and Mesocosm's saved loop. The sim's
  loop carries `interim::Ending` (wait, since, call, extend, forfeit, par):
  left at its default the boundary fires on the epoch's tick count as
  before; waiting holds it for a call once the span has passed; an extension
  skips one timed boundary; a forfeit or a met condition fires it at once
  (`SpanPassed`, `ConditionMet`). `boundary::par` grows a copy a span, left to
  itself, and returns the most matter the lineage holds in it (D8);
  `par_using` does so through a product's clock. Mesocosm's `LoopSettings`
  gain `rounds` (capped by default, waiting, continuous; D7) and `condition`,
  unserialized at their defaults so old saves read and write as before; the
  loop answers gain `Call`, `Extend` and `Forfeit`; the ending is saved with
  the loop and each round's source, and a saved boundary is validated by the
  rule that fired it (`Ending::due`) where it was by the tick count alone. In
  continuous mode the played line's review opens at the first boundary after
  a call. Controls: eight in `directing/interim/ending_tests.rs`, four in
  `tests/round_endings.rs` with every question revalidated after a save and
  restore, and the forfeit end to end in `continuation/ending_tests.rs`.
  Owed: the host's keys for the three answers (LR4h), and further kinds of
  condition (Questions).
- 2026-11-09: LR1.2 built, sim side and the host's flag.
  `directing::readings::starts` lists a lineage's valid starts (range sites
  with their lowest playable member, then fringe sites with the nearest) and
  `start_at` refuses any other site by name. `Start` carries an optional
  `pick`, absent in old saves and unserialized when absent; with none the
  first life is chosen as before. A fringe site is one a playable member can
  walk to in one route step, read from `readings::sources`' walk means, so a
  body that cannot move starts where it is (D6). A fringe start takes that
  body and nudges it to attend the site: the sim has no command that sends a
  critter anywhere, and the nudge is the only directive (Mesocosm 2).
  Mesocosm's host takes `--start W:E:SITE`. Eight controls in
  `directing/interim/start_tests.rs`. Owed: D6's cohort, since a nudge needs
  a bond and only the taken body has one until regard gives its kin theirs
  (LR5); and the pick in Mesocosm's saved loop, where a change of lineage
  already offers every living playable body, so a range start, and no
  fringe.
- 2026-11-09: LR1.1 built. `directing::readings::heirs` orders a lost
  critter's heirs (descendants by generation, then route steps from its
  site, then relatedness, then id), and `readings::route_steps` counts routes
  both ways; the sim's `next_life` takes the first and Mesocosm's
  `succession::heirs` lists the same order, any of which the player may take
  (D4). Five controls in `directing/interim/tests.rs`: a child before a
  same-site stranger; the nearer site first; a far member is an heir and the
  lineage is not ended; unreachable sites and unknown kin last; the reading
  the same from a replayed run. No existing test or pinned hash moved, and a
  replay re-applies the logged `Take`, so old saves replay as before.
- 2026-11-09: plan cut from the brief. The trophic rename (1222) landed the
  same day and is logged in the sim plan.
