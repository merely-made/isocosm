# One host for the three games: the idea against what is ruled and built

**Status:** 2026-11-12: brief, for Mark to rule on. It changes no code and no
record. Its questions (§7) are open. Its main finding is that most of the idea
is already ruled (cases 381, 442 to 445) and part of it is built; what is open
is narrower than the idea, and §3 and §7 say where.

**For:** the owner's 2026-11-12 idea of one platform under Isocosm, Mesocosm
and Eponym, with each game loaded on it and data linked across them. It lays
out what exists, what is ruled, what the idea could mean at four levels, three
options and one more, and what only Mark can decide. Names are Naming 1's
(case 1036): the products are Isocosm (the VTT), Mesocosm and Eponym; the sim
is Cosmograph. Naming 3 holds, so no new name is coined here: "the host", "a
game", "a world" and "a save" are descriptions, and naming is a question (§7,
Q6). Code paths are from the repository root.

Sources read: the root and product `CLAUDE.md` files, `doctrine.md`, the
founding record (§1, §3, §4, §7), `DOC_POLICY.md`, the three indexes, the sim
host plan (`mesocosm/design_docs/2026-10-28_sim_host_plan.md`), the lineage
rounds plan, the functional generation plan, the content packs plan and the
session network plan, and the crates named in §2. `responsa lookup` finds a
case by number or area; it found nothing for plain words (`cartridge`,
`persona`, `pack` and the like), so the words were searched in
`case_register.md` instead.

## 1. The idea, in Mark's words

2026-11-12, on whether a subagent could write a design brief for it:

> "maybe we can spawn a subagent with a design brief? for a sim host that can,
> like, play the games depending on which you load, i guess? or hmmm... idk. i
> guess i'm just thinking of a game system and cartridges, which feel
> intuitive to me. a sim that underwrites all three games kinda feels like the
> equivalent, platform-wise. linking the data across the games and whatnot."

From the same conversation, as relayed: he compared the sim to ALIEN
(github.com/chrxh/alien), where one simulator hosts many shared worlds through
a browser, and he wants worlds, lineages and creature learning to carry
between games. The records hold nothing on ALIEN (a search of the design docs
found no mention) and this brief did not fetch its repository, so ALIEN is
taken only as he described it. "Cartridge" is his analogy and is not used as a
term below.

## 2. What exists today

Line counts are Rust lines under each crate's `src/`, to show size, not
quality.

| | Isocosm | Mesocosm | Eponym |
| --- | --- | --- | --- |
| Host crate | `crates/isocosm-genet` (15,271) | `mesocosm/crates/mesocosm-genet` (17,478); host-neutral `mesocosm-runtime` (9,961) beside it | `eponym/crates/eponym-client` (15,237); its entry is the `session` bin, "a working name" (`bin/session.rs`) |
| Game logic | `isocosm-native` (11,543), `isocosm-system` (12,234) | in `mesocosm-runtime` and the sim | `eponym-play` (16,104), `eponym-motion`, `eponym-sortie` |
| Window, frame loop | `isomere::host::Assembly` over `cambium-desktop` | the same | the same |
| Input, controls | its own hooks (`hooks`, `dispatch`), board gestures in `isocosm-views` | its own (`input.rs`, `app/actions`) | its own (`session/`, `producer`) |
| Rendering | Cambium views and CSS sheets, board on isometer (plan in progress) | isometer slab scene, lens brick tracer (`borrowed_section`) | isometer slab scene (`borrowed`, `producer`) |
| Save and load | `campaign_store` (isocosm-campaign); the family ledger; `--campaign <name>` | `GameSave` lowered by `ImportSave` to the core's `WorldSave` (`mesocosm-runtime/src/runtime/world_save.rs`) | a `GameSave` that carries a `WorldSave` (`eponym-play/src/save_wire.rs`) |
| Section key | `"vtt"` (`isocosm-native/src/endpoint.rs:50`) | `"mesocosm"` (`runtime.rs:49`) | `"eponym"` (`eponym-play/src/endpoint.rs:33`) |
| Flags | `ISOCOSM_*` environment hooks, `--campaign` | `--replay`, `--watch`, `--scenario`, `--bench` (`main.rs`) | its own bins, several behind proof features (`eponym-client/Cargo.toml`) |
| World founding | campaign founding in `isocosm-native`; in-app founding from a seeded draw is owed (sim host H8) | `runtime/founding.rs`, behind the A1 witness | `eponym-play/src/founding.rs` |
| Rule choices | a system plugin per campaign (schema plus Lua), loaded from packs | `WorldRules` in `HostConfig`; the `mesocosm` pack (`mesocosm/packs/mesocosm/mesocosm-pack.toml`) | native recipes; crafts are keys a rules pack declares (Eponym 3) |
| Licence | MIT OR Apache-2.0 (root `Cargo.toml` `workspace.package`) | MPL-2.0 (`mesocosm/LICENSES.md`) | MPL-2.0 (`eponym/Cargo.toml`) |

What is shared already:

| Piece | Where | What it does |
| --- | --- | --- |
| The sim | `shared/cosmograph` | One world, history, forks, saves (`history::Saved`); no product engines (Stack 4) |
| The embeddable host | `cosmograph-host/` (a library, its own `[workspace]`, MPL-2.0) | `World`, `WorldSave { format, native, sections }` with one opaque byte section per mode, `Window` and attention binding, founding, clock and pacer, the inspection endpoint, atomic file save (`file_save.rs`). All three product hosts depend on it by path |
| The contract | `shared/cosmograph-overlay` | Intents, events, views and handoffs, with a module per game (`mesocosm/`, `eponym/`, `vtt/`) and a game-neutral core |
| The GUI layer | `shared/isomere` | `Assembly`: producer registration, scenario lane, capture, error line, profile line |
| Scene | `shared/isometer` | One orthographic slab scene and body inputs |
| Interchange | `shared/wing-formats` | Versioned body and chronicle records (`mesocosm.body/v0`, `mesocosm.chronicle/v2`) |
| Cross-product receipts | `shared/wing-integration` | A separate workspace that depends on `cosmograph-host`, `mesocosm-runtime`, `eponym-play` and `isocosm-native` at their public edges; has per-product family-network tests and a producer-to-reader chronicle test |
| Session layer | `crates/isonetry` | The family session: one host sequences each world (Host 5) |
| Content | `shared/wing-glyphs`, `wing-functions`; `packs/` (`5e-srd`, `ars5`, `pf2e-srd`), `mesocosm/packs/mesocosm` | Packs of rules and data |
| Identity | dramatis `personae`, `castellan-client` | A persona; used by `isocosm-genet`, `isonetry` and `wing-integration`, and not by `cosmograph-host` (see §3, finding 5) |

What is duplicated, or owned three times:

- The product host crates each own their flags, scenario and capture setup,
  founding path, section codec, and pacing policy (the last by case 927, on
  purpose).
- `[workspace]` roots: the root, `mesocosm/`, `eponym/`, `cosmograph-host/`,
  and each of `shared/{cosmograph, cosmograph-overlay, isomere, isometer,
  wing-formats, wing-functions, wing-glyphs, wing-integration}`. Patch rows
  (`[patch]` tables): root 25 plus two tables, `cosmograph-host` 13,
  `wing-integration` 15, `mesocosm` 4, `eponym` 3. The union was not computed.
- Mesocosm's `chrome.rs` and its five panel adapters survive as a
  hand-rolled layer; moving them off is sim host I1 (open).
- No test was found that holds all three games' sections in one world. The
  per-product family-network tests in `wing-integration` each insert an
  opaque sentinel section; `cosmograph-host/src/tests/save.rs` round-trips an
  uninstalled mode's bytes. Not every file there was read.

What does not exist: an executable that loads more than one game. The sim host
plan's own Findings (2026-10-28) say `cosmograph-host` has no `ui/` and no
desktop shell, and its `main.rs` offers only `--validate-save` and
`--endpoint`. Plan I3 ("Mode host acceptance") is open, and the plan's parked
note says the mode host carries only Mesocosm's mode until the VTT and Eponym
overlays run on the sim (a reading, not ruled).

## 3. What is already ruled

| Record | What it permits or forbids | Bears on |
| --- | --- | --- |
| Case 381, Host 1 | One host carries the three games as modes over one world save; each product is a build profile; players in different modes may share a trunk. Mark chose "One host, modes", over one product and over three sovereign products | The idea itself |
| Founding record, "Each vessel is a mode" (amended 2026-09-28, by 381 "applied at Mark's word by ruling 385") | A build may carry one mode or all three. What survives: no mode may require another mode's code to run; each game keeps its genre, verbs and care granularity. The older text (a vessel never shares a running world) is kept and annotated | What a host may and may not couple |
| Case 442, Stack 10 | The all-modes build lives in a fourth workspace holding only the host binary, depending on the products' crates by path under the union of their patches. Options "one workspace for all" and "the root workspace hosts" were offered and not chosen; the separate workspaces stay | Workspace separation: it stays, and the host is a fourth |
| Root `CLAUDE.md` | Separate workspaces "to preserve source and patch policy"; "runtime separation does not prohibit shared libraries or atomic changes across products" | The stated reason, which 442 keeps |
| Case 443, Host 4 | A window plays one mode and switches in place, keeping world, subject and embodiment; two modes at once are two windows | What the player sees |
| Case 444, Record 4 | One save holds the sim's history plus an opaque section per mode, carried unread and unchanged by a build without that mode. A mode's section branching and merging is the mode's own (a reading, not ruled) | Save format |
| Case 445 | A window's knowledge follows its participant's seat and subject, never its mode | Switching cannot leak |
| Case 815 | Each active window adds to one player's attention set (a union) | Two windows |
| Cases 884, 892, 898, 925 to 927, Host 1 | Three uniform product hosts over one embeddable core; all on `Assembly`; Mesocosm's session in the core's `World`; the core owns the clock, each product picks its pacing | The three hosts are meant to stay |
| Host 5, cases 959, 960 | `isonetry` is the family session layer; one host sequences each world; replicas take the accepted log and section changes and never rerun a product adapter | Many players on one world |
| Schema 9, Schema 10 | Old saves stay readable; new fields are optional; a feature gates operations, never state, so every build reads one save format | Feature gating |
| Founding §3, Laws A to C | What crosses games is choices under scarcity, not morphology (materials excepted); inheritance is pointable; no homework: the same seed format whether authored by play or RNG, each game standalone-complete | What may carry |
| Founding §4, "Graduated interoperability" | Five trust rungs: observe, import, contribute, participate, co-simulate; rung 5 is "the netcode dragon"; additive facts, opaque preservation, deferred interpretation | The levels in §4 |
| Case 1011, World 2, cases 126, 7, Record 4 | Related worlds relate by descent (forks) or neighbourhood, nothing merges, each keeps its log and clock; a crossing is recorded in both logs | Worlds across worlds |
| Case 1227, 1214 | A player's persona is their dramatis identity, minted if absent; bonds and regard key to it across worlds | Identity |
| Cases 1051, 1053, 1056 | A lineage's policy is a small net in the sim; learned deltas pass by imitation and teaching; a driven sapient creature learns from its player "in every game that drives" | Learning |
| Doctrine Stack 14, Stack 3 | Rungs: data, piccolo (rulesets and world packs), components, native crates; piccolo is the wing's one authoring language | Option C |
| Founding §4 | "A game's presence in a moot is pack-shaped: schema plus scripts plus a capability manifest, which is what Isocosm's system plugins already are" | Option C |
| Founding §1, "Facts cross; verbs never do" | Kinematics, tile-and-turn adjudication, renderers and control schemes do not cross | Option C |
| Licensing 1, 3; Eponym and Mesocosm `LICENSES.md` | The VTT is MIT OR Apache-2.0, the games, shared crates and promoted libraries MPL-2.0, assets CC BY-SA 4.0; code that changes layer takes the destination's licence; a ruleset ships when its licence allows, checked per product | Licences |
| Content packs plan; functional generation D4 | A pack is the unit content ships in, declaring `license` and `credit`; a world's magic typologies are data in packs, templates or stepwise generation | Packs |
| Naming 3 | One word, one sense; naming rounds are deliberate | No coined names |
| Method 8, 9 | Refuse, never invent; offer a thing only once it works | A game meeting a foreign world |

Findings in the records that cut against, or around, the idea:

1. **The idea is mostly Mark's own case 381.** The question this brief can
   still ask is not "one host or three" but how far 381 goes, when, and with
   what first step. Case 385, which the founding record cites as the one that
   applied 381, is "cited, but declared nowhere" in `responsa lookup 385`
   (it is 381 that is live). That is a record gap, not a conflict.
2. **The host is designed and its library is built; its executable is not.**
   §2. "Spawning a host" would duplicate sim host I3 unless it is the same
   work.
3. **LR9 and LR10 do not mean what the framing suggests.** In the lineage
   rounds plan, LR9 ("Shared lineages and adopt or branch", cases 1205, 1221)
   is several players sharing or splitting a lineage's gene pool inside one
   world, and LR10 ("Crossings and the candidate kingdoms") is a lineage
   changing scale or kingdom (colony, parasite and others). Neither carries
   anything from one game to another. Both are parked by lineage-rounds D16
   until hosts, a playable handle, body-size rates and members' learning are
   built. Crossings between worlds are cases 149 and 1011 (World 2); case
   1057 is listed as "a crossing is keyed by its route's border", and whether
   that is between sites or worlds was not determined.
4. **Founding §3's Law C and 385's guard limit how linked games may be.** A
   world a game did not found must still be playable by it; no game may
   require another's code. A linked world therefore cannot be a world only one
   game can open.
5. **D23 in the lineage rounds plan says `cosmograph-host` "already takes"
   `personae`; its `Cargo.toml` does not list it.** `isocosm-genet`,
   `isonetry` and `wing-integration` do. The dramatis repository has the
   `personae` and `djinn-custodian` crates (its READMEs describe a persona
   register and the custodian that holds the vault), but no `cosmograph-host`
   crate: that lives in this repository.
6. **No ruling was found on** a combined binary's licence, build time, how a
   per-mode section's own version is checked when another build opens the
   world (the envelope has `format: u32`; a section's version is the mode's
   own), how a learned policy moves between worlds, or hosting worlds for
   others to join through a browser.

## 4. The analogy taken seriously

**The platform** would be the sim and its host: `cosmograph` (world, history,
saves, forks), `cosmograph-host` (open world, window, attention, founding,
clock, inspection), the overlay contract, and the GUI assembly. The records
already name it this way: "one sim host under three product hosts" (Host 1),
and Host 2 says outside any game the endpoint-and-viewer workbench "reads,
analyses and edits entities and worlds' data and generation". The workbench
(`world-graph-viewer`, case 1038, behind `cosmograph-host`'s `viewer` feature)
is the nearest thing to a front door today.

**A game** is what the records call a mode: an overlay vocabulary
(`cosmograph-overlay/src/{mesocosm,eponym,vtt}`), a game crate, a section key,
a binding (controls, view defaults, attention: `binding.rs`), and its own care
granularity (species, individual, community). It owns verbs, camera and
controls and never the sim's rules (founding §1, Stack 4).

**A world** is the sim's history and genesis (`NativeSaved`), with a seed and
digests; a fork is a new world related by descent (Record 4).

**A save** is the world plus one opaque section per mode (case 444). It is
the closest thing to the owner's cartridge, but it is a file the games write,
not a thing the console reads from a game.

**What could be shared across games**, and where it lives today:

| Thing | Shared now? | Where |
| --- | --- | --- |
| A world | Yes, as the native part of a save | `WorldSave.native` |
| Lineages | Yes, they are sim facts, so they are in every game's view of the world; a lineage-as-identity across worlds is fili, reserved in founding §7 | the sim; Founding §4 provenance rule |
| A persona (the player) | Ruled (1227), not built in the games | dramatis `personae` |
| Learned policies | Inside the sim, so they travel with the world; between worlds nothing carries them | `shared/cosmograph/src/directing/policy.rs` (case 1051). How it serialises was not checked |
| Packs of content | Per product today (`packs/`, `mesocosm/packs`) | content packs plan |
| Records of a life | Chronicle and body records | `shared/wing-formats` |

"Linking the data across the games" can mean four different things:

| Level | Meaning | State |
| --- | --- | --- |
| 1. One save format | Every game reads and writes the same file, and keeps what it cannot read | Ruled (444), built in `WorldSave`, round-trip tested for unknown sections; Mesocosm and Eponym write it, the VTT's native endpoint reads and writes sections too, but its desktop app saves through its campaign store. Import acceptance for the VTT and Eponym is sim host H8, owed |
| 2. One world, games in turn | Found in one game, opened in another, the first's section carried unchanged | Ruled (381, 444); no cross-game round trip found; needs a game's reading of a foreign world (founding §4: "interpretation, not merging") |
| 3. One world, games at once | Two windows or two peers over the same world, in different modes | Ruled for one machine (443, 815) and for peers by Host 5; rung 5 of the trust ladder is the netcode dragon. Pacing differs: Mesocosm's rounds, Eponym's real-time motion, the VTT's consented downtime (927; sim host H8's pacing inventory) |
| 4. Identity and lineage across worlds | A persona, a lineage, a creature's learning following a player into another world | Persona ruled (1227); worlds relate by fork and crossing (1011); the law on what crosses is Law A (choices, and materials). Nothing ruled for policies |

Levels 1 and 2 need the games to read each other's worlds, which is a
presentation question under Law C: a game may refuse, fill in its own
defaults explicitly (Schema 10), or observe only (rung 1). The records do not
say which; see Q3.

## 5. Options

The costs below are from the records and the manifests; none was measured.

| | A. Three hosts, shared save and library | B. One host, games compiled in | C. One host, games as data and scripts | D. A plus a front door |
| --- | --- | --- | --- | --- |
| Shape | Keep the three binaries. Finish the shared save and the host library | A fourth-workspace binary with the three games as modes, chosen at launch or switched in a window (442, 443) | One generic host; a game is a pack of schema, scripts and a manifest | Option A, plus a small launcher and workbench that owns the world library and starts a game binary over a chosen world |
| What moves | Eponym and the VTT onto the core's founding and save (H8); Mesocosm's chrome off (I1) | Product host code becomes crates the host links; a mode selector and window lifecycle (I3) | The verbs, camera and controls of each game would have to become scripts | The viewer's founding and save handling grows into a world list; products take a world path |
| What breaks | Nothing new; the cross-game round trip must still be tested | The host workspace needs the union of three patch policies (442); one build graph carries all games | See below: Lua returns integers only; real-time motion and voxel tracing are native; the founding record says verbs never cross | A second program to keep in step with three products |
| Workspaces | Stay as they are | Stay, plus the host's (442 A) | Collapse toward one | Stay |
| Licences | Unchanged | A binary linking MIT OR Apache crates with MPL crates; no ruling found (Q5). `isocosm-genet` already links MPL crates (`cosmograph-host`, `isomere`) in its own build | A ruleset's licence still checked per product (Licensing 1) | Launcher and products ship separately |
| Patch policy | Per product, as now | Union in one place; 442 says Eponym's renderling had to retire first and no `renderling` or `spirv` entry is in `eponym/Cargo.toml` now | One graph | Per product |
| Build time, features | No change | Larger graph in one link; features must gate operations only (Schema 10); the VTT's campaign features are `default = []` and were once found uncompilable behind the default build (root `CLAUDE.md`); no build-time figure found | Smaller binary, a heavier script surface | No change |
| Player sees | Three apps; a world file opens in each | One app; a mode switch in place keeps world and subject (443) | One app; games are things you install | One home screen, three apps |
| Smallest first step | A cross-game test in `shared/wing-integration` (§6) | A mode switch between two games' bindings in a window over one `World`, using `Window` and `Binding` as built | One game's whole verb set as a pack; the existing evidence says this fails for verbs | The workbench opens a saved world and starts one product on it |

How far the Lua plugin system goes, from `crates/isocosm-system/src/lib.rs`:
a system is a schema plus Lua (via piccolo) that interprets the substrate's
`SheetData` into derived stats and dice expressions; "every script function
returns an integer", and dice expressions are assembled in Rust; the sandbox
runs under fuel. Packs carry rulesets and records, with a `license` and
`credit`. The Mesocosm pack holds fifteen process definitions as data, and
its Lua "proposes and never decides: the host prices what it asks for". So
the plugin lane covers rules and content, which is what the founding record
means by "pack-shaped" presence in a moot. It has no surface for windows,
input, rendering or motion, and no record or code found extends it to them.
C is therefore the right model for what a game's *rules* are (a ruleset,
a typology: functional generation D4) and not, on this evidence, for a game.

A fifth shape the code suggests, which this brief does not carry as an option:
making the games' hosts one process with several windows (Host 4 already says
two modes on one machine are two windows). It is B with a different launch.

## 6. A recommendation

*This is the writer's recommendation, not the owner's, and not a ruling.*

**Take D as the end shape only if Mark wants a home screen; otherwise keep
the ruled shape (381, 442 to 445, which is B) and spend effort on the part of
A that every option needs: a world that three games read and write.**

Reasoning:

- The part of the idea that is new is the linking, and linking is a property
  of the save, not of the launcher. B, C and D all depend on the same two
  things: a world each game can open, and sections each game keeps. These are
  built in the library and unproven across games.
- A single binary adds the costs that the three-host shape avoids (union of
  patches, one link, one feature graph) and adds nothing the player needs
  until two games can open one world. Method 9 says to offer a thing once it
  works.
- C has the wrong grain for the games' verbs. It has the right grain for
  rulesets, and that is already built and ruled.
- Mark's cartridge intuition fits a world file plus a host that opens it,
  which the records already describe (444).

The smallest step that would test it: one cross-game test in
`shared/wing-integration`, which already depends on `cosmograph-host`,
`mesocosm-runtime`, `eponym-play` and `isocosm-native`. Found one world
through `cosmograph-host`. Write Mesocosm's section through its `ImportSave`
adapter and Eponym's through its own. Save, resume under each game's reader,
and assert the other game's bytes and the native hash survive unchanged. Done
when: a world founded under a seed nobody chose (Method 1) holds two or three
modes' sections, and each game's reader opens it and returns the others'
sections byte for byte; and a deliberately wrong section version in one
section is refused by its game and carried by the others (the negative
control). That tests levels 1 and 2 of §4 without a new binary, a new name, or
a licence question. This brief names no code change; it would be a plan under
DOC_POLICY §8 if Mark rules for it.

## 7. Questions for Mark

Each can be answered with a letter.

**Q1. Is this brief's idea case 381?**
- A. Yes: 381 and 442 to 445 stand, the work is I3 and H8, and no new host is
  started.
- B. 381 stands, but narrower than built plans imply: modes share the world
  file first (level 1 and 2), and a single executable waits.
- C. Reverse it: three apps, a shared save format and no mode host (retire
  442 and 443).
- D. Wider: games as data and scripts (Option C above), accepting that rules
  and verbs would both need a surface.

**Q2. Which level of linking should come first?** (§4)
- A. 1, one save format readable by all.
- B. 2, one world opened by games in turn.
- C. 3, one world played by games at once.
- D. 4, identity and lineages across worlds.

**Q3. When a game opens a world it did not found, what does it do?**
- A. Refuse, with a stated reason (Method 8).
- B. Open it with its own defaults copied in explicitly (Schema 10).
- C. Open it for observation only (rung 1).
- D. A per-game choice.

**Q4. Does the platform have a front door?**
- A. No; each game launches itself and opens a world file.
- B. A launcher and workbench that owns a world library (Option D).
- C. The in-window mode switch only (443).

**Q5. A binary that links the VTT's MIT OR Apache crates with the games' MPL
crates: which ruling?** No record or ruling found; mere's licence posture brief
(`mere/design_docs/2026-08-22_license_posture_brief.md`) is cited by Licensing 3
and was not read here.
- A. The host binary is MPL-2.0, each crate keeps its layer's licence.
- B. The VTT stays out of the all-modes binary.
- C. Ask in mere first, since Licensing 2 leaves such releases to mere.

**Q6. Names for the platform, a game and a save?**
- A. Not now; descriptive words only.
- B. A naming round now.
- C. A naming round when a host binary exists.

**Q7. How far does a creature's learning carry?** (cases 1051, 1053, 1056)
- A. Within one world only, as ruled.
- B. With a lineage that founds a fork or crosses into another world.
- C. With a persona as a portable record (Law A's "choices", not a net).
- D. Not decided yet.

**Q8. Shared worlds through a browser, as ALIEN does?**
- A. Out of scope; the family session (Host 5) is the sharing model.
- B. After the family session's gates (session network H7) pass.
- C. Brief it separately; ALIEN's design is read first.

**Q9. Should the records be repaired?**
- A. Add 385's declaration to the case record and fix D23's `personae` line in
  the lineage rounds plan.
- B. Only D23.
- C. Neither.
