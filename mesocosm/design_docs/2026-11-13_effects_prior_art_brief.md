# Prior art for the effects, traits and parts catalogs

**Date:** 2026-11-13

**Status, 2026-11-13:** brief, for Mark's reading. It changes no rule and
makes no ruling. It answers D38 and D39 of the
[lineage rounds plan](2026-11-09_lineage_rounds_plan.md): the catalogs of
traits, effects and the parts that result are wanted, and none of the effects
is novel, so each should have open implementations or data to learn from.

**Owns:** the per-effect map of open implementations and data, with each
source's license and a verdict. **Does not own:** the simulation-model prior
art (Madingley, DEB, Lotka-Volterra, Ecopath, the genre references), which is
the [sim prior art brief](2026-09-18_sim_prior_art_brief.md) §1 and is not
repeated; the rows of the catalogs themselves (owed, D37); any ruling.

**How it was checked.** Every license and content claim below comes from a
page, API record or source file fetched on 2026-11-13; the URL is given.
GitHub's license API and crates.io's API were used where the repo page did not
state a license. "Unverified" means the fetch failed or returned only a
search snippet; those rows say what is missing. Field names are quoted;
no data or text was copied.

**License frame** (doctrine Licensing 1 and 3; `LICENSES.md`): the sim is
MPL-2.0; original assets and the data-type core are CC-BY-SA-4.0; imported
material keeps its license and is recorded in `assets/ATTRIBUTION.md`; a
pack declares its own license and credit; an open text informing the sim is
a reference and is not copied in. Verdict words: **take** (license
compatible; attribution noted), **pattern** (read the shape, write our own
rows and code), **avoid** (reason given).

---

## 1. Toxins, venom, poison tolerance, status effects with onset and duration

| Project | URL | Lang | License | What it has (paths) | How it models it | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| Cataclysm: DDA | https://github.com/CleverRaven/Cataclysm-DDA | JSON data, C++ | CC BY-SA 3.0 for game code and content, per README; fonts Apache-2.0 and OFL-1.1 | `data/json/effects.json`, `disease.json`, `vitamin.json`, `species.json`, `mutations/mutations.json`, `monster_special_attacks/`, `monsters/` | Effect: `max_duration`, `int_dur_factor`, `max_intensity`, `int_decay_tick`, `base_mods` and `scaling_mods` (stat or rate keys such as `hurt_tick`), `resist_traits`, `resist_effects`, `removes_effects`, `rating`. Durations as strings with units ("15 m") or bare turns. Toxin as a counter: `vitamin.json` entries with `vit_type` of toxin or drug, signed `min`/`max`, `rate`, `disease_excess` bands, `decays_into`. Species tag carries `flags` such as `VENOM_IMMUNE`. Trait: `POISRESIST` | Pattern. Copyleft data: flag, do not take rows. The intensity-times-duration effect and the banded excess counter are the shapes worth reading |
| Space Station 14 | https://github.com/space-wizards/space-station-14 | YAML data, C# | Code MIT (LICENSE.TXT). Content mostly CC-BY-SA 3.0, some assets CC-BY-NC-SA, per README; per-asset metadata decides. Whether the YAML prototypes fall under the content grant is not stated on the pages read | `Resources/Prototypes/Reagents/` (`toxins.yml`, `biological.yml`, `medicine.yml`, `narcotics.yml`, `botany.yml`), `Resources/Prototypes/status_effects.yml` | A reagent has `metabolisms` keyed by stage (`Bloodstream`, `Digestion`), each with `metabolismRate`, a list of effects (`HealthChange` by damage type, `ModifyStatusEffect`, `Vomit`, `MovementSpeedModifier`...), gating `conditions` (reagent min or max, `MetabolizerTypeCondition` with `inverted`, `SatiationCondition`) and a `probability`. `plantMetabolism` and `reactiveEffects` (by `methods`) are separate keys. `Mechanotoxin` is gated by an inverted `Arachnid` metabolizer type: a venom that does not harm its own kind. `status_effects.yml` itself is declared legacy and thin | Pattern, strongly. Do not copy rows (license of the YAML unclear, SA likely) |
| Veloren | https://github.com/veloren/veloren | Rust | GPL-3.0 (FAQ) | `common/src/comp/buff.rs` | `BuffKind` enum (`Bleeding`, `Burning`, `Poisoned`, `Frozen`, `Regeneration`, `Hastened`...); `BuffData`: `strength`, `duration`, `delay`, `secondary_duration`; `BuffEffect` variants including `HealthChangeOverTime` with `tick_dur`, `DamageReduction`, `BuffImmunity`, `DeathEffect`; stacking rule per kind (`stacks`, `queues`, else strongest wins); `BuffCategory` for removal | Pattern only (GPL). Clean source of onset (`delay`) plus duration plus tick |
| bevy_alchemy | https://crates.io/api/v1/crates/bevy_alchemy | Rust | MIT OR Apache-2.0 (v0.6.0) | docs.rs README: effects as entities; `Delay` and `Lifetime` timers; `MergeMode` of Stack, Insert, Merge; same `Name` means same effect; a poison example holding `damage` | Effect as an entity related to its target, with timers | Take as dependency or pattern, if the sim is entity-based (not checked here). License fits MPL-2.0 |
| bevy_gauge / bevy_stat_query | https://crates.io/api/v1/crates/bevy_gauge, https://crates.io/api/v1/crates/bevy_stat_query | Rust | MIT OR Apache-2.0 (v0.6.0; v0.5.0) | Stat and modifier systems; `bevy_stat_query` uses add, multiply, min, max, or | Modifier stacks over a base stat | Pattern; take only if Bevy is in play |
| OpenMW | https://github.com/OpenMW/openmw | C++ | GPL-3.0 | `components/esm3/effectlist.hpp` | Effect record `ENAMstruct`: `mEffectID`, `mSkill`, `mAttribute`, `mRange` (0, 1, 2: self, touch, target per its comment), `mArea`, `mDuration`, `mMagnMin`, `mMagnMax` | Pattern only. The nearest published form of "an effect has a delivery range and a magnitude span". The game's own data is Bethesda's: avoid |
| ALIEN | https://github.com/chrxh/alien | CUDA, C++ | BSD-3-Clause (GitHub license API) | `resources/docs/cell-types.md`, `simulation-parameters.md` | Cell types Attacker (attack strength, food-chain color matrix, size protection, same-lineage protection), Injector, Defender (anti-attacker, anti-injector strengths), Detonator, Digestor, Sensor | Take as pattern now; code and numbers are takeable with the BSD notice. The defence-versus-attack pair is the closest open analogue to D39's defence and counter-defence |
| The Powder Toy | https://github.com/The-Powder-Toy/The-Powder-Toy | C++ | GPL-3.0 | `src/simulation/elements/` (one file per element, e.g. `ACID.cpp`, `CAUS.cpp`) | Per-element update code. Contents not read | Avoid (GPL); not read further |
| Sandspiel | https://github.com/MaxBittker/sandspiel | Rust to WASM, JS | MIT (GitHub license API) | `crate/src/species.rs` | `Cell` of `species`, `ra`, `rb`, `clock`; per-species update. Acid replaces a neighbor with a degraded copy; mites eat Plant, Wood, Seed | Take as pattern or code (MIT, WASM is the point). A cell-grid toy: little biology |
| Sandboxels | https://github.com/R74nCom/sandboxels | JS | R74n Content License 1.1 (GitHub license API; terms read at https://r74n.com/license.txt): reuse and modification allowed with credit, but commercial use needs the licensor's explicit permission and the licensor may request removal | `scripts/` | Not read: element definitions not located | Avoid (non-commercial, removal clause) |

## 2. Armour, shell, spines, thorns; tough tissue; mouthparts and digestion by food category

| Project | URL | Lang | License | What it has | How it models it | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| Cataclysm: DDA | as above | JSON | CC BY-SA 3.0 | `data/json/body_parts.json`, `bodypart_graphs/`, `monster_weakpoints/`, `materials.json`; monster `armor` object per damage type, `weakpoints`; trait field `integrated_armor`; body part `armor`, `unarmed_damage`, `effects_on_hit`, `hit_size` | Armor as a per-damage-type table on a part. Weak points as sets with coverage | Pattern. Flag SA |
| DCSS | https://github.com/crawl/crawl | C++ | GPL v2 or later for code, with some CC0 and public-domain files, tiles mostly CC0 (LICENSE) | `crawl-ref/source/mutation-data.h`, `dat/species`, `dat/mons` | `mutation_def`: `mutation`, `weight`, `levels`, `flags` (good, bad, anatomy, substance, scales, need_hands...), per-level text. Examples `MUT_POISON_RESISTANCE`, `MUT_SPINY` (3 levels), `MUT_TOUGH_SKIN` | Pattern only (GPL). The levelled trait with form-based flags is a clean precedent for "a trait is a row with levels and part prerequisites" |
| Luanti engine | https://github.com/Luanti-org/luanti | C++, Lua API | LGPL-2.1 (engine) | `doc/lua_api.md` | `armor_groups` and `damage_groups`: a table from group name to a rating, the percent of damage taken or dealt through that group. Groups are integer name-to-rating tables on items and nodes | Pattern. A small, general shape for "armour versus kind of harm" |
| Thrive | https://github.com/Revolutionary-Games/Thrive | C#, JSON | GPL-3.0 or later (LICENSE.txt, with an exception for dynamically loaded mods); no asset exception stated | `simulation_parameters/microbe_stage/organelles.json`, `enzymes.json`, `compounds.json`, `membranes.json` | Organelle: `Processes`, `Components`, `Enzymes`, `ToleranceEffects`, `Hexes`, `MPCost`, `UnlockConditions`, `IncompatibleMembraneNames`. Compound: `Digestible`, `Diffusible`, `IsAgent`, `DecayRate`. Enzymes gate which compounds a cell can digest | Pattern only. The best match to "digestion of a food category" as data: an enzyme per category, a part carrying it |
| CDDA | as above | JSON | CC BY-SA 3.0 | `monsters` fields `harvest`, `dissect`, `decay`; item fields `calories`, `vitamins`, `parasites`, `contamination` | Diet is not a field; eating is gated by item category and traits (`HERBIVORE`, `CARNIVORE`-like traits appear as mutation ids, not all read) | Pattern |

## 3. Carrion ageing, dung and litter, decay, scavenging

| Project | URL | License | What it has | How it models it | Verdict |
| --- | --- | --- | --- | --- | --- |
| CDDA | https://github.com/CleverRaven/Cataclysm-DDA (docs `doc/JSON/ITEM.md`, `doc/JSON/MONSTERS.md`) | CC BY-SA 3.0 | Item `spoils_in` (0 means never), `rot_spawn` (what the item becomes when it rots), `contamination` (diseases carried, with probabilities), `parasites` (one-in-x on eating); monster `decay` (harvest left when the corpse rots away), `biosignature` (droppings), `harvest` with `drop`, `type`, `base_num`, `scale_num`, `mass_ratio` | The nearest open model of D37's flesh ageing from fresh to toxic is `spoils_in` plus `contamination`; no source read turns age itself into a toxin dose | Pattern. Dung as `biosignature` is a drop on the creature, not a food category |
| Thrive | https://github.com/Revolutionary-Games/Thrive | GPL-3.0 | `compounds.json` field `DecayRate` | A per-compound decay | Pattern |
| ALIEN | https://github.com/chrxh/alien | BSD-3-Clause | `simulation-parameters.md`: cell life cycle includes a decay rate of dying cells | Dead cells decay at a rate | Take as pattern; thin |

## 4. Parasitism, disease, infection, epidemics, immunity

| Project | URL | Lang | License | What it has | How it models it | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| NetLogo models library | https://github.com/NetLogo/models | NetLogo | Per model; the Wolf Sheep Predation, Virus and Daisyworld Info tabs state CC BY-NC-SA 3.0; NetLogo itself GPL for non-commercial per those tabs; some models public domain | `Sample Models/Biology/` (Virus, Disease Solo, HIV, Wolf Sheep Predation, Daisyworld, Rabbits Grass Weeds, Invasive Plants, Fairy Circles, Rock Paper Scissors) | Virus: `infectiousness` (percent on shared patch), `chance-recover`, `duration` (weeks), fixed immunity of 52 weeks. Wolf Sheep: `grass-regrowth-time`, `sheep-gain-from-food`, `wolf-reproduce` (percent per tick); energy below zero kills | Avoid copying (non-commercial). Pattern for the numbers' shapes only; also the classic references the sim prior art brief already cites |
| Mesa examples | https://github.com/projectmesa/mesa-examples | Python | Apache-2.0 (LICENSE read; GitHub's API shows "Other") | `examples/pathogen_outbreak`, `virus_antibody`, `forest_fire`, `rumor_mill`, `axelrod_culture` | `pathogen_outbreak`: `spread_chance`, `death_chance`, states healthy, infected, immune, dead; contact only, no incubation | Take (Apache-2.0) as pattern; it is Python, so port the idea, not the code. Thin: one transmission mode |
| Mesa | https://github.com/projectmesa/mesa | Python | Apache-2.0 | framework | Agent-based modeling framework | Pattern |
| krABMaga | https://github.com/krABMaga/krABMaga | Rust | MIT (ISISLab, Salerno) | engine; examples repo https://github.com/krABMaga/examples has `wolfsheepgrass` (`src/model/sheep.rs`, `wolf.rs`, `state.rs`), `forestfire`, `virusnetwork`, `sugarscape`, `schelling`, `antsforaging` | Rust discrete-event ABM. The examples repo's workspace manifest carries no license field and the repo license endpoint returned 404: examples' license unverified | Engine: take as dependency or pattern (MIT). Examples: treat as pattern until a license is found |
| Agents.jl | https://github.com/JuliaDynamics/Agents.jl | Julia | MIT | framework; `examples/` folder not listed | ABM framework | Pattern |
| CDDA | as above | JSON | CC BY-SA 3.0 | `disease.json` entries: `min_duration`, `max_duration`, `min_intensity`, `max_intensity`, `health_threshold`, `symptoms`; item `contamination` | Disease as duration and intensity bands that trigger a symptom effect | Pattern |
| SS14 | as above | YAML | see §1 | reagent effect `CauseZombieInfection` | Infection as an effect fired by a metabolism | Pattern |

Transmission by ingestion is `contamination` and `parasites` on an item
(CDDA). Transmission by contact is the shared-patch rule in Virus and
`pathogen_outbreak`. Transmission by soil: nothing found.

## 5. Mutualism (mycorrhiza, gut symbionts, pollination, seed dispersal)

Nothing found as data or code. See §10 for what was read. The nearest hits:

| Project | URL | License | Relevance |
| --- | --- | --- | --- |
| Thrive | https://github.com/Revolutionary-Games/Thrive | GPL-3.0 | `organelles.json` fields `EndosymbiosisUnlocks`, `EquivalentInSpecializationTo`: symbiosis as organelle acquisition, not as exchange. Pattern only |
| Global Biotic Interactions | https://www.globalbioticinteractions.org/about | Code GPL-3.0 (GitHub API); data per dataset (§9) | Records which species pairs interact (pollinator-plant, parasite-host, predator-prey). It says who, not how much. Seeds the pairing tables, not the exchange rates |
| Sandspiel | see §1 | MIT | Seeds germinate on Sand, Plant or Fungus; Fungus consumes Wood. A toy, not a nutrient exchange |

## 6. Hiding, camouflage, burrowing, fleeing, size refuge, predator functional response, satiation

| Project | URL | License | What it has | Verdict |
| --- | --- | --- | --- | --- |
| ALIEN | https://github.com/chrxh/alien | BSD-3-Clause | Attacker `Size protection` and `Same lineage protection`; Sensor `Restrict to lineage`. A size refuge and a kin refuge as parameters | Take as pattern |
| CDDA | as above | CC BY-SA 3.0 | Species `fear_triggers` and `anger_triggers` (`HURT`, `FIRE`, `SOUND`, `FRIEND_DIED`), `scent_intensity` on traits, `visibility` on traits | Pattern: flight and detection as triggers and scalars |
| NetLogo Wolf Sheep Predation | see §4 | CC BY-NC-SA 3.0 | Fixed energy gain per kill; no functional response | Avoid copying; it is the baseline the sim already beats |
| Mesa, krABMaga, Agents.jl | see §4 | Apache, MIT, MIT | Wolf-sheep as a baseline only | Pattern |

No source read implements a Holling-type functional response or satiation as
data. Satiation appears in SS14 as `SatiationCondition` on an effect and as
`Satiate`, which is a nutrition meter, not predator satiation.

## 7. Competition for limiting resources, dormancy and dispersal, succession and disturbance

| Project | URL | Lang | License | What it has | Verdict |
| --- | --- | --- | --- | --- | --- |
| LANDIS-II core | https://github.com/LANDIS-II-Foundation/Core-Model-v8-LINUX | C# | Apache-2.0 (GitHub license API, core repo only; extensions' licenses unverified) | Forest landscape model: growth, competition, seed dispersal, succession and disturbance (fire, wind, harvest, insects, browsing) as extensions, per landis-ii.org text in search results (page itself not fetched) | Take as pattern; core is Apache-2.0. Its extension structure (succession and disturbance as plug-ins on a shared core) is the interesting part. Contents not read |
| Mesa examples `forest_fire` and krABMaga `forestfire` | see §4 | Apache-2.0; examples' license unverified | Fire spreading as contact between cells | Pattern |
| NetLogo Daisyworld, Invasive Plants | see §4 | CC BY-NC-SA 3.0 | Seeding onto adjacent patches by a temperature-dependent probability (Daisyworld, read) | Avoid copying |
| Sandspiel | see §1 | MIT | Seed falls, germinates on a substrate, stages by a counter; fire and lava ignite wood and plant | Take as pattern; the staged-counter growth fits a WASM grid |
| Luanti `farming` mod | https://content.luanti.org/packages/TenPlus1/farming/ | Code MIT; media CC-BY-SA-4.0 | Crops grow while no one is nearby; seeds drop from grass | Take as pattern; code MIT. Not read beyond the package page |

## 8. Metabolism and reagents, mutation catalogs, body-part catalogs, generation, growth grammars

| Project | URL | Lang | License | What it has | How it models it | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| Thrive | as above | JSON | GPL-3.0 | `compounds.json`, `bio_processes.json` (`Name`, `Inputs`, `Outputs` as name-to-amount maps, `IsMetabolismProcess`), `organelles.json`, `biomes.json` | Processes convert compounds at amounts; organelles carry process lists; biomes carry compound levels | Pattern. The closest open shape to our process definition plus a part catalog |
| SS14 | as above | YAML | see §1 | Reagents, `Recipes/` (reactions), `Body/`, `Species/`, `Hydroponics/`, `Traits/` directories | Metabolism by stage; reactions between reagents; plant growth chemistry in `Hydroponics/` (not read) | Pattern |
| CDDA | as above | JSON | CC BY-SA 3.0 | `mutations/` (27 files incl. `mutation_category.json`, `mutation_type.json`, `mycus.json`); mutation fields `prereqs`, `prereqs2`, `threshreq`, `changes_to`, `leads_to`, `cancels`, `types`, `category`, `vitamin_cost`, `enchantments`, `integrated_armor` | Mutation as a graph: category pools, prerequisites, thresholds, upgrade chains, mutual cancellation by type | Pattern. The best published mutation-catalog shape: the graph (`prereqs`, `cancels`, `changes_to`) is what a catalog of parts needs |
| ALIEN | as above | | BSD-3-Clause | `docs/genomes.md`, `evolution.md`, `cell-types.md`; meta-mutation groups (properties, cell identity, gene structure, genome structure) | Fifteen cell types as parts, constructors read genomes | Take as pattern |
| Veloren | as above | RON | GPL-3.0 | `assets/common/` (`abilities`, `entity`, `skillset`, `plugin_bodies.ron`) listed, not read | Not verified for body parts | Avoid (GPL), unread |
| Symbios | https://crates.io/api/v1/crates/symbios | Rust | MIT (v1.5.2) | Derivation engine for L-systems with arithmetic in rules, described as ABOP compliant | Parametric L-systems | Take as dependency or pattern. The repo already has a four-rule growth grammar of its own (lineage rounds plan); this is a comparison, not a replacement |
| rusty-systems | https://crates.io/api/v1/crates/rusty-systems | Rust | MIT (v5.0.0) | Context-free and stochastic L-systems, per docs.rs snippet | Pattern |
| lindenmayer | https://crates.io/api/v1/crates/lindenmayer | Rust | MIT (v0.3.0) | Memory-efficient L-systems | Pattern |
| openc2e | https://github.com/openc2e/openc2e | C++ | LGPL-2.1 (GitHub license API) | Reimplementation of the Creatures engine; its biochemistry was not read | Unverified contents. Creatures is already cited in the sim prior art brief as the deepest biochemistry; this is its open code |
| Avida | https://github.com/devosoft/avida | C++ | Unverified: the repo listing shows no LICENSE file; a search snippet says LGPL and other listings say GPL | Digital organisms | Avoid until the license is read |
| Biogenesis | https://sourceforge.net/projects/biogenesis/ | Java | GPLv2 per a search snippet; page not fetched | Segment-based organisms, "Genetic Laboratory" | Avoid (GPL), unverified |
| Framsticks | https://www.framsticks.com/a/al_download.html | | Unverified: the license agreement was not read; the download page labels some components open source | Stick-figure body genotypes | Avoid until read |
| The Bibites | | | Not fetched; believed closed | | Unverified |
| Dwarf Fortress raws | https://dwarffortresswiki.org/index.php/Syndrome | | Not open; the wiki fetch returned 429, so not read this session | Format precedent only, from memory: syndromes with injected, contact, ingested and inhaled routes and start, peak and end timing. Unverified here | Precedent, not a source |

Rust roguelike tutorials, rotLove (GitHub license API: NOASSERTION), Brogue CE
(AGPL-3.0 per the GitHub API; contents not read) and NetHack were not mined.
5e SRD conditions are already in the repo as `packs/5e-srd` (CC-BY-4.0, per
that pack's own header): a lawful, in-repo source for the status names
(poisoned, frightened, stunned, and so on), with no onset curve.

## 9. Real-biology datasets that could seed trait values

| Dataset | URL | What it has | License (as fetched) | Note |
| --- | --- | --- | --- | --- |
| EltonTraits 1.0 | https://api.figshare.com/v2/articles/3559887 | Diet and foraging stratum as scores, body size, activity time; 9,993 birds, 5,400 mammals | CC0 1.0 | Safest. Diet scores map to food categories (D37) |
| AmphiBIO v1 | https://api.figshare.com/v2/articles/4644424 | 17 natural-history traits (ecology, morphology, reproduction), over 6,500 amphibians | CC BY 4.0 | Attribution entry needed |
| PanTHERIA | https://esapubs.org/archive/ecol/E090/184/metadata.htm | 30 life-history variables for about 5,400 mammals (body mass, gestation, litter size, diet breadth, trophic level, home range) | Metadata: no copyright or proprietary restrictions and no cost; asks for citation; no formal license named | Cite; license ambiguous, treat as facts |
| TRY plant traits | https://www.try-db.org/TryWeb/Data.php | Plant traits | Per dataset: most "Public" with no license named, one categorical set CC BY 3.0; free registration; some data on request only | Use only a named-license dataset |
| BETYdb | https://www.betydb.org/ | Biofuel-crop traits and yields | ODC-By 1.0 for public data | Narrow scope |
| GloBI | https://zenodo.org/api/records?q=GloBI%20interactions%20dataset | Species interaction records (pollination, predation, parasitism) | Zenodo deposits read: "Unique Interactions" CC BY 4.0; review corpus ODbL (share-alike on the database); one bee subset CC0 | ODbL flagged |
| Wikidata | https://www.wikidata.org/wiki/Wikidata:Licensing | Taxon facts | CC0 | Structured data namespaces only |
| FishBase | https://www.fishbase.se/search.php | Fish traits | CC BY-NC 4.0 | Avoid (non-commercial) |
| AnAge | https://genomics.senescence.info/species/ | Longevity records, 4,671 entries | "HAGR's license terms", not read | Unverified |
| Add-my-pet (DEB) | https://www.bio.vu.nl/thb/deb/deblab/add_my_pet/ | DEB parameters per species, MATLAB structures | No license named on the page | Unverified; useful for the metabolism rates the sim prior art brief cites |

Not fetched: Open Tree of Life, Wikispecies.

---

## 10. The best sources overall, and the shape of their data against a catalog row

Ranked by how much of a catalog's shape they give and whether it can be
used.

1. **Cataclysm: DDA** (CC BY-SA 3.0; pattern). The only source with all of:
   an effect with intensity, duration and decay; a counter-style toxin with
   bands; trait prerequisites and cancellation; per-part armor; corpse decay;
   item contamination. Mapping: our *effect* row takes `max_intensity`,
   `int_dur_factor`, `int_decay_tick` and resistance by trait id; our *trait*
   row takes `prereqs`, `cancels`, `changes_to` and `category`; our *part* row
   takes the per-damage-type armor table. Do not copy rows.
2. **Space Station 14** (code MIT; content CC-BY-SA 3.0 mostly; pattern).
   Metabolism keyed by *stage* is the nearest match to "effects fire at
   transfer sites". `Digestion` is the meal; `Bloodstream` is the body after
   absorption; `reactiveEffects` with `methods` is contact; `plantMetabolism`
   is the soil or producer side. Gates (`conditions`, with `inverted` on a
   metabolizer type) express immunity and "harmless to its own kind" without
   a separate immunity table. Mapping: our *effect* row becomes
   site, rate, gate list, probability, outcome list.
3. **Thrive** (GPL-3.0; pattern only). Compounds with `Digestible`, `IsAgent`
   and `DecayRate`; processes as input and output maps; organelles carrying
   process lists, enzymes and tolerance effects. Mapping: a *part* row holds a
   list of processes and the enzymes (food-category keys) it carries; the
   category gate for D36 and D37 is an enzyme match.
4. **ALIEN** (BSD-3-Clause; code and parameters takeable with notice). The
   only permissively licensed source whose parts include attack, defence,
   injection and digestion with explicit counter-parameters (anti-attacker and
   anti-injector strengths, size and lineage protection). Mapping: our
   defence-and-counter-defence pair (D39's second model) as two strengths on
   two parts, plus refuges as protections on the target. Its docs also give a
   genome and meta-mutation vocabulary for the mutation catalog.
5. **Veloren's buff.rs** (GPL-3.0; pattern) with **bevy_alchemy**
   (MIT OR Apache-2.0; takeable). `delay`, `duration`, `tick_dur`, `strength`
   and a per-kind stacking rule is the onset, peak and duration shape in
   Rust. Mapping: the *effect* row's timing block. bevy_alchemy's merge modes
   (stack, insert, merge, by effect name) are the stacking rule in a license
   we can ship; whether the sim is entity-based enough to use the crate was
   not checked.

Takeable code and data, in one list (all license-compatible, attribution
noted): ALIEN (BSD-3-Clause; keep the notice), Sandspiel (MIT), krABMaga
engine, Agents.jl (MIT), Mesa and mesa-examples, LANDIS-II core (Apache-2.0;
keep NOTICE where one exists), bevy_alchemy, bevy_gauge, bevy_stat_query,
symbios, rusty-systems, lindenmayer (MIT or MIT OR Apache-2.0), and the
datasets EltonTraits (CC0), AmphiBIO (CC BY 4.0) and Wikidata (CC0).
Nothing there is a catalog of effects: they are engines, a few mechanisms
and trait values.

## 11. Gaps

Effects for which nothing usable was found, or nothing was read:

- **Mutualism exchange** (mycorrhizal nutrient trade, gut symbionts,
  pollination rewards, seed-dispersal payments): no open implementation
  found. GloBI gives pairs, not rates.
- **Camouflage, burrowing, hiding as detection reduction**: only trigger and
  scalar shapes (CDDA `visibility`, `scent_intensity`).
- **Predator functional response and satiation as data**: none found; the
  textbook equations in the sim prior art brief are the source.
- **Carrion that ages into toxin**: CDDA has spoilage time and contamination
  chance, not a dose rising with age. This is ours to define.
- **Dung and litter as a food category**: CDDA has `biosignature` on the
  producer; nothing models dung as food.
- **Soil as a transfer site for disease**: nothing found.
- **Competition for light, water and nutrients** with root depth, leaf area
  and storage: LANDIS-II is the candidate; its contents were not read. NetLogo
  Daisyworld and Invasive Plants are non-commercial.
- **Not verified**: Dwarf Fortress raws (429), Brogue, NetHack, the Powder
  Toy's element code, Framsticks, Avida and Biogenesis licenses, The Bibites,
  Veloren's body and species data, SS14's Hydroponics and Body directories,
  TRY whole-database terms, Add-my-pet and AnAge terms.

## 12. Open questions for Mark

1. **Posture on copyleft data.** CDDA and most SS14 content are CC BY-SA 3.0,
   and the assets are already CC-BY-SA-4.0. Options: (a) pattern only, write
   all rows ourselves, ship under the pack's own license (recommended, and
   the doctrine's reading); (b) take a few rows with attribution and let that
   pack carry share-alike, accepting that the 3.0 to 4.0 relicensing question
   was not checked here; (c) nothing from SA sources.
2. **GPL sources** (Thrive, Veloren, Powder Toy, OpenMW, DCSS, GloBI code):
   read for shape only, never port code. Confirm that reading them is fine
   as "pattern", or restrict to not reading source at all.
3. **Real-trait seeding.** Options: (a) derive trait ranges from EltonTraits
   and AmphiBIO offline, record the source in `assets/ATTRIBUTION.md`
   (recommended; CC0 and CC BY); (b) invent all ranges; (c) also GloBI, which
   adds an ODbL share-alike question.
4. **Effect shape.** Options: (a) SS14-like, one effect row keyed by transfer
   site with a gate list; (b) DDA-like, a status with intensity and decay
   applied at any site; (c) both, with (a) as how an effect arrives and (b)
   as what it becomes in the body.
5. **Timing block.** Take bevy_alchemy or write our own from the
   delay, duration, tick and stacking shape; this depends on whether the sim
   is entity-based, which was not checked here.
6. **A second pass.** Read LANDIS-II, SS14 Hydroponics and Body, openc2e's
   biochemistry and Brogue's bolts and lichen, to close the gaps in §11?
   It is the next cheapest step.
