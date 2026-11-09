# Seed banks: what the producers' collapse asks of dormancy

**Status:** 2026-11-08: SC3's measured assessment (sim plan S10.2), opened by
case 1189 ("Seed banks next (S10.2)"). It changes no rules or biology. Its
questions (§6) were answered on 2026-11-08 as cases 1192 to 1197 (§6's
closing note).

**For:** the measured evidence and design questions behind the sim plan's
SC3: dormant propagules with conserved matter, reduced upkeep and a sensed
wake condition (ecology brief 3.4, cases 1146 and 1159). It also asks
whether dormancy, on its own, addresses the drain that ended seeds 5 and 9 on
the two-year bar. Code paths are under `shared/cosmograph/`.

## 1. The evidence and how it was read

- **Receipts:** the 2026-11-08 bar rerun after E7b
  (`Code/testing/isocosm/bar/2026-11-08/seed-<n>.json`, source `7f87d95e`),
  with monthly living counts, root and exposed tissue, soil and detritus.
- **Diagnostic runs:** `tests/world_time_year.rs`'s bar founding, seeds 9
  and 5, for 21 months. A new ignored instrument,
  `tests/seed_bank_assessment.rs`, advances one tick at a time with
  `Session::advance_tick_with_flows`. It sorts every matter move that touches
  a producer by process and by holder: root, exposed tissue, ledger, a site
  account, or another body by diet. At each month's end it also reads living
  and dead bodies by part, and every site's accounts.
  `COSMOGRAPH_PROFILE_ACTIVE=sum` gives each process's monthly acts.
- **Neutral:** the diagnostics changed no rules, and their monthly
  counts match the receipts. Seed 9 had 251 producers at month 8 and 300 at
  month 13; seed 5 had 332 at month 13.
- **Missing move:** `Effect::Regrow` records no matter move. Its root-to-exposed
  transfer is read as the residual of each month's root balance, after
  production into the root, upkeep, births and the root left at death.

## 2. Where the producers' matter goes

Months 12 to 21, both producer lineages together. Production is soil taken
into producers, in matter a month. "Into root" is the share of production
filed in the root store. "Grazed" is exposed tissue eaten by grazers and
omnivores. Upkeep is paid from the root.

Seed 9 (lineage 0's root fell from 3,565 at month 13 to 0 at month 20):

| Month | Producers | Root | Exposed | Production | Into root | Grazed | Upkeep | Births | Grow acts accepted / evaluated |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| 12 | 286 | 5,690 | 11,013 | 22,560 | 16,196 | 13,868 | 7,273 | 12 | 11,280 / 16,002 |
| 13 | 300 | 6,111 | 11,721 | 24,320 | 17,648 | 15,420 | 7,771 | 14 | 12,160 / 16,725 |
| 14 | 312 | 5,815 | 12,113 | 25,896 | 18,818 | 17,457 | 8,171 | 15 | 12,948 / 17,550 |
| 15 | 320 | 5,500 | 12,497 | 26,816 | 19,259 | 18,151 | 8,381 | 12 | 13,408 / 18,470 |
| 16 | 327 | 4,174 | 12,313 | 27,596 | 18,433 | 21,076 | 7,858 | 10 | 13,798 / 19,105 |
| 17 | 334 | 3,452 | 12,457 | 28,284 | 16,335 | 22,179 | 6,545 | 9 | 14,142 / 19,607 |
| 18 | 336 | 2,102 | 11,730 | 27,286 | 14,102 | 23,457 | 5,851 | 4 | 13,643 / 20,170 |
| 19 | 333 | 163 | 8,568 | 24,776 | 5,957 | 27,280 | 2,489 | 0 | 12,388 / 20,332 |
| 20 | 130 | 0 | 775 | 19,982 | 248 | 27,392 | 121 | 0 | 9,991 / 17,660 |
| 21 | 0 | 0 | 0 | 1,090 | 0 | 1,604 | 0 | 0 | 545 / 716 |

Seed 5:

| Month | Producers | Root | Exposed | Production | Into root | Grazed | Upkeep | Births | Grow acts accepted / evaluated |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| 12 | 318 | 4,910 | 12,372 | 23,006 | 17,477 | 14,863 | 8,500 | 9 | 11,503 / 18,337 |
| 13 | 332 | 4,724 | 12,786 | 24,890 | 17,944 | 15,998 | 8,501 | 17 | 12,445 / 19,317 |
| 14 | 333 | 4,379 | 12,752 | 26,304 | 17,377 | 18,680 | 7,710 | 6 | 13,152 / 19,476 |
| 15 | 341 | 3,977 | 12,821 | 27,464 | 17,152 | 20,214 | 7,515 | 9 | 13,732 / 19,881 |
| 16 | 353 | 2,963 | 12,619 | 27,754 | 15,499 | 22,220 | 6,684 | 13 | 13,877 / 20,959 |
| 17 | 354 | 2,002 | 12,076 | 27,800 | 12,106 | 24,145 | 5,017 | 4 | 13,900 / 21,286 |
| 18 | 348 | 198 | 8,392 | 26,372 | 5,258 | 29,460 | 2,184 | 0 | 13,186 / 21,369 |
| 19 | 311 | 0 | 2,446 | 24,980 | 503 | 30,814 | 189 | 0 | 12,490 / 20,759 |
| 20 | 0 | 0 | 0 | 5,782 | 0 | 7,615 | 0 | 0 | 2,891 / 3,578 |

What the numbers say:

- **Production plateaus near 27,700 a month.** The cap is crowding, not
  stock. Sun stood at 185,000 to 360,000 per site against a store of
  360,000, and soil never fell below 31,000. A quarter to a third of
  evaluated growth acts were refused as producers passed about 85 a site.
  The gates are filed, so what an evaluation can still fail on is the
  sensed light and soil shares.
- **Grazing kept rising.** Grazers and omnivores went from 148 to 314 (seed
  9, months 12 to 19) and from 171 to 350 (seed 5). Grazing passed
  production net of upkeep at month 16 on seed 9 and month 14 on seed 5,
  and passed production itself at month 19 and month 18.
- **The deficit came from the root through regrowth.** Regrowth moved
  9,300 to 11,700 a month from root to exposed tissue on seed 9 (months 13 to
  18), and 8,000 to 9,900 on seed 5 (months 12 to 17). That is more than the
  whole root stock each month, and the grazers ate it. With
  the root empty, production went to exposed tissue instead (seed 9: 6,672
  at month 13, 18,819 at month 19), which was grazed too.
- **Producers starved only when their roots were spent.** Seed 9 lost 202
  producers in month 20 and seed 5 lost 311. Every consumer starved within the
  following month, decomposers included.
- **Producer numbers held while their matter drained.** The consumers'
  breeding gate counts prey per head, not prey matter. Producers stayed at
  about 330 while roots fell to nothing, so the gate stayed open.
- **Births stopped before the crash.** A birth needs 70 in the body. As
  roots drained, births fell from 14 to 17 a month to none by month 19.

## 3. Stranded pools

- **Protected dead roots.** Dead producers' roots were never eaten: no move
  ever left a dead root. They held 816 at month 21 on seed 9 and 783 on
  seed 5, beside 102 and 80 of dead exposed tissue. The decomposers starved
  next to them. A root below the ground plane stays unexposed when its body
  dies, so no feed can bind it.
- **Site-dropped ingesta.** `world:ingesta` stood at 0 at every site at
  every monthly sample on both seeds. The detritus-to-soil step's skipped
  pool was not seen here. Ingesta still pending inside bodies was not read.
- **Detritus** stayed under 150 in most months, because decomposers kept
  pace. Soil rose from about 33,000 to 54,700 after the collapse.
- **Dead consumer tissue** was left uneaten once decomposers died: about
  330 to 400 grazer and 145 omnivore matter at month 21.

## 4. How propagules exist today

- **Who scatters.** A producer's birth (`ecology:birth-<i>`, every 90 days
  once its body holds 70) gives a child 10 from its root (case 1114's seed
  share). The lineage's drawn propensity, 50,000 to 150,000 in a million
  (1093), decides through the birth's risk whether the birth scatters. The
  medium's strength at the parent's site, `world:air` or `world:water` out
  of 100 (1092), decides whether it goes to a neighbour along a route
  (1089, `stage/staged/births.rs`). Decomposers scatter the same way. A
  walker's young do not; they wait on walking speed (1085).
- **How many.** Seed 9's producers made 242 births in 19 months, 23 of them
  scattered (9.5%). Seed 5's made 260, 25 scattered (9.6%).
- **What a propagule is.** The child is an active body from its first
  tick. It pays upkeep of 1 a day from its root, can be grazed, and grows
  only by the parent's own processes. Nothing is dormant: a 10-matter
  seedling would pay out its whole body in 10 days if it did not grow.

## 5. What a dormant stock would have had to do

- **Size.** Producers were founded at 131, 60 matter each. A bank able to
  refound them at seed size (10 each) would hold about 1,300 matter.
  Banking about half of all producer births from the start would have
  reached that by month 19 on either seed (2,420 and 2,600 matter born in
  all), if dormant upkeep were near nothing.
- **Upkeep.** At the active rate a seed lasts 10 days. To last a lean
  interval of 6 to 12 months it must pay at most about 1 in 20 to 40 days,
  or nothing with a seed death rate.
- **Waking.** A light-share wake (producers thin, so a seedling could
  grow) would have fired in month 20 on seed 9. Over 300 starving grazers
  and omnivores were still alive then, eating about 27,000 a month; a
  1,300 bank is about 5% of that. Waking safely meant waiting until month 21,
  when every consumer was dead.
- **The bar.** By then four of the five diets were lost everywhere.
  Case 1173 fails on them whatever the producers do. A producer seed bank
  can keep the producers' diet and lineages through this trough. It cannot
  keep the consumers fed, so on its own it cannot pass the bar on seeds 5
  or 9.
- **The other seeds.** On 11, 17 and 31 predators were lost everywhere
  (months 4, 23 and 2). That is not a producer trough, and a plant's seed
  bank does not reach it.

## 6. Questions for SC3

Each question gives options with a recommendation first.

**Q1. What wakes a dormant propagule?**
1. **Recommended: its site's light share reaching what a seedling needs to
   grow (`sense:light-share` at 3,000), with only a keyed fraction of a
   site's stock waking each season.** Existing sense, bet-hedged, so one
   flush cannot spend the whole bank into waiting grazers (§5).
2. Light share alone, every viable propagule at once.
3. A season's start (seasonal light, 973) after a minimum dormancy.
4. A new sense of grazing pressure: a new reading, so a rule question
   first.

**Q2. What does dormancy cost?**
1. **Recommended: reduced upkeep as a rate worked out when read, like
   strain (E7a), a stated fraction of the active rate accumulated
   deterministically.** A propagule dies when its matter runs out, and its
   matter goes to the site as detritus. This is S10.2's "reduced upkeep" and
   "fractional costs accumulated deterministically" as written.
2. No upkeep, with a rising seed death rate drawn once like E7b. Cheap, and
   conserved if the dead seed's matter returns.
3. No upkeep and no death: banks never decline. Simplest, but a stock
   could outlast any trough.

**Q3. What counts as alive?**
1. **Recommended: dormant stock is alive for extinction (a lineage holding
   only propagules is not extinct) and reported apart.** It does not count as
   a resident for case 1173's diets or for foreground membership. Living
   adults and viable stock are measured separately, as S10.2 says; the bar
   keeps measuring function.
2. Alive for both: a diet held only as seeds passes 1173's residency.
3. Alive for neither: a seed bank never prevents an extinction reading.

Dormant bodies are never posed or foreground in any option.

**Q4. How does dormancy fold into the cohort form (1163)?**
1. **Recommended: individual propagules now, as bodies carrying a dormant
   stage trait.** Gated processes skip them through the filed index; upkeep
   is a rate and waking a due event, so no pass visits them until they wake.
   They join 1163's stage pools when the cohort form is built. This uses
   existing machinery (traits, gates, due events) and keeps conservation and
   replay as they are.
2. A per-site, per-lineage pool now: a count and pooled matter, the first
   piece of 1163's form. Cheaper for large banks, but it adds a new
   container to conserve, save and replay.
3. Individuals, never pooled: simplest, but each seed stays an entity at
   every scale.

**Q5. Does dormancy alone address the drain?** The numbers say no (§2, §5).
The drain is consumer overshoot against a crowding-capped production:
grazing rose from about 15,000 to 27,000–31,000 a month, while consumers bred
on prey counts that did not fall. Regrowth lent the root store to exposed
tissue that was then eaten.
1. **Recommended: build dormancy as ruled (it carries producer lineages
   through troughs and meets S10.2's done-condition beside a no-dormancy
   control), and take SC1/SC2's consumer limit next, measured with dormancy
   on.** No dormant stock keeps consumers fed.
2. Dormancy plus a root floor regrowth may not lend (a bud bank): producers
   survive grazing, but consumers still starve when exposed tissue runs out.
3. Dormancy plus a consumer breeding gate on prey matter rather than prey
   counts. S10.1's `FoodPerHead` counts members whatever their matter.
4. Dormancy alone, measured against its control before deciding anything
   else.

Separately from SC3: decomposers cannot reach dead roots (§3), which
strands about 800 matter a run and starves decomposers beside it. It is a
reach or exposure question for the root-store contract (case 1114), not for
dormancy.

**Answered 2026-11-08.** Q5: "Dormancy, then consumers (Recommended)"
(1192). Q3: "Extinction only (Recommended)" (1193). Mark reframed Q1 and Q2
around seed mass ("maybe seeds can have a differing amount of mass, and the
ones with more need less sun"; "a player can choose to release a bunch of
cheap seeds or less, more robust seeds?"), and they came back as four:
seed size drawn per lineage and inherited (1194); waking light falls with a
seed's matter, a keyed fraction a season (1195); dormant upkeep a fraction of
the active rate, following it to the mass law (1196); Q4, individual
propagules now (1197).
