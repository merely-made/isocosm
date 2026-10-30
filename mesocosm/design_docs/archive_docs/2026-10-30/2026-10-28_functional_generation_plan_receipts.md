# Functional generation F12 receipts, 2026-10-30

Owner: the Magic Weapon desktop implementation lane. Governing plan:
[functional generation](../../2026-10-28_functional_generation_plan.md), F12.
Starting main: `4a715b02`; initial pull was current, and a subsequent fetch
still found no upstream changes. Total retained receipt cap: 100 MB.

## Scope and authority

D&D SRD 5.1 Magic Weapon in free play, using manually authored sheets and
explicitly unmarked table equipment. Native participants and the current
seat cut control the endpoint. Casting, attacks, external damage, effect
expiry and edition state use the existing family host's atomic acceptance.
System plugins remain the rules authority. This creates neither native
matter nor native physical equipment; it does not test full initiative,
native magical manufacture, class progression or cross-map item migration.

Physical attack admission reads native resident poses first, then admitted
VTT position bindings. A declared native-cell scale measures root distance
and an existing terrain sight ray through the sim's site geometry cache.
This is not articulated weapon reach. In the uncalibrated headed Bench,
the painted **Rule within reach and attack** button records the DM's ruling.

## Automated gates

PowerShell, repository root, Rust 1.99.1. Reused the existing repository
`target/` and `C:/t/cargo-homes/isocosm-foraging`; the latter remains owned
by the earlier shared Mesocosm foraging/build lane as its offline dependency
cache. This task created no isolated target, Cargo home or worktree.

```powershell
$env:CARGO_HOME='C:/t/cargo-homes/isocosm-foraging'
cargo +1.99.1 test -p isometry-core -p isometry-system -p isocosm-vtt --features isocosm-vtt/endpoint --lib --tests --offline --locked -j 3
cargo +1.99.1 test -p isometry-views --lib state::vtt --offline --locked -j 3
cargo +1.99.1 check --workspace --all-features --all-targets --offline --locked -j 3
```

Passed: 186 core/system/VTT tests and three existing view-routing tests.
The all-feature, all-target workspace check passed with existing warnings.
The new endpoint cases cover slot spending, stale-request refusal, actual
enhanced attacks, concentration faces/end causes, native time expiry,
player-versus-DM control, exact family checkpoint/record replay and measured
native reach/refusal. The D&D critical case rolls damage dice twice and adds
Magic Weapon's flat bonus once. PF2e's existing critical multiplier stays.

The physical fixture uses seed 11, a flat 2x2 native grid with four 64-cell
sites, paid generated bodies, declared full-site stands and native
`Command::Patch` admission. Earlier fixture attempts exposed invalid grid
parameters and the distinction between a declared stand and a currently
admitted terrain patch. The production adapter refuses an unadmitted pose;
the fixture now performs the same native placement admission as Mesocosm.

## Native desktop trials

Each trial started from a fresh synthetic Bench ledger, seed 11, caster 3,
target 5, two level-two slots, a wielded longsword and an explicitly empty
saved player roster. Fixture setup itself used advertised endpoint actions.
The selftest used retained-DOM painted rectangles, scroll and pointer events,
including the ordinary native downtime proposal/consent/advance path.

```powershell
cargo +1.99.1 build -p isometry-genet --bin isometry-genet --offline --locked -j 3
cargo +1.99.1 run -p isocosm-vtt --features endpoint --example magic_weapon_table --offline --locked -j 3 -- <fresh-case>/campaigns/Magic_Weapon.family
# Launch target/debug/isometry-genet.exe --campaign "Magic Weapon" with the
# fresh case as working directory and these inherited environment variables:
$env:ISOMETRY_MAGIC_WEAPON_SELFTEST='active' # or lost / expired
$env:ISOMETRY_CAPTURE_DIR='<fresh-case>'
```

| Case | Result | Slots left | Native tick | Accepted suffix records |
| --- | --- | ---: | ---: | ---: |
| active | Magic Weapon applies; enhanced attack uses +3 attack and +1 damage | 1 | 0 | 2 |
| lost | One external damage causes a failed concentration save and removes the enhancement | 1 | 0 | 3 |
| expired | After concentration loss, recast and accept one hour of downtime; second effect expires | 0 | 60 | 6 |

All three emitted `durable_replay=exact`: replay from the startup checkpoint
and persisted accepted suffix matched the current native world, opaque
edition section and revision. This is native headed host and saved family
replay evidence. No live network peer or production transport was exercised.

Initial PNGs, inspected after capture:

| Capture | Bytes | SHA-256 |
| --- | ---: | --- |
| [Active](magic_weapon_play/active/isometry_capture.png) | 496099 | `8806c8bca5e182fe95bba6f382dcce1f20180a668b8c0919e1cbd876f8c43368` |
| [Initial lost readout](magic_weapon_play/lost/isometry_capture.png) | 517119 | `dc77cd2f61a47e8cd445877dc1a19e6392070248c3ffc767bb15cecf5bbd87e1` |
| [Expired](magic_weapon_play/expired/isometry_capture.png) | 524734 | `6c0c1767624ea8e5c24f84c82688860645523f9c3bed998e789b1319b46722a2` |

The initial lost capture showed the correct ended state and concentration
cause but misleadingly retained 3600 seconds remaining. It is preserved as
the failed readout receipt. The display now reports zero for ended effects.
A fresh corrected trial and disk-reopen results are appended below.

The fresh [corrected lost capture](magic_weapon_play/lost_readout/isometry_capture.png)
was inspected: `State: ended`, `Seconds remaining: 0`, `End cause:
Concentration lost`; the log retains attack `1d20+3 = 13`, damage `1d8+1 = 7`
and concentration `+0 vs DC 10 = 2`, with their actual die faces. It is
515267 bytes, SHA-256
`5e7f9ddfaa35734ca2ede90d13edc285ac8749c421d88a02cc06fc20713a869a`.
Its headed PASS reported one slot left, native tick zero, one effect, one
damage check, three accepted suffix records and exact durable replay.

After each writer stopped, the fixture inspector reopened its actual file:

```powershell
target/debug/examples/magic_weapon_table.exe --inspect <case>/campaigns/Magic_Weapon.family
```

The family host verified and replayed the checkpoint/suffix before exposing
state. The inspector also required the **stored** effect data to agree with
the accepted native clock. All four reopens passed; the ledger bytes were
unchanged before/after inspection. Compact reports are retained publicly:

| Saved case | Sequence | Reopened state |
| --- | ---: | --- |
| [active](magic_weapon_play/active/disk_reopen.json) | 12 | one slot, active effect |
| [lost](magic_weapon_play/lost/disk_reopen.json) | 13 | one slot, concentration ended |
| [expired](magic_weapon_play/expired/disk_reopen.json) | 16 | zero slots, tick 60, second effect expired |
| [corrected lost](magic_weapon_play/lost_readout/disk_reopen.json) | 13 | one slot, concentration ended |

The final desktop build passed. The trial directory, including ignored
synthetic runtime stores, totals 17,941,456 bytes; this compact receipt adds
less than 10 KB, comfortably inside the 100 MB cap.

Staged checks passed: `responsa regen --stage`, `responsa lookup
"functional-generation D1"`, `responsa check`, `scripts/line-ceiling.ps1
-Staged`, `scripts/pack-json.ps1 -Staged` and `git diff --cached --check`.
Responsa retained the pre-existing unknown-link warning at the orthographic
presentation plan's line 51; its manifest, doctrine budget and declared
cross-repository links checked successfully. No new decision was invented
for the operational instruction to proceed.

Synthetic `.family` ledgers, host `.redb` stores, lock files and full local
logs remain ignored under `magic_weapon_play/` for replay. Public evidence
is the PNGs and this compact receipt. All application processes launched by
this lane are closed after capture; no existing evidence was removed.
