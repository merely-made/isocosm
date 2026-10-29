# Functional generation F11: executable small set

2026-10-28, Windows, Rust 1.99.1. Base: `e6d8ede6` on `main`;
`git pull --ff-only` was already current, and a subsequent fetch found no
upstream commits. This is the follow-up to the catalogue-only F10 receipt,
which is retained unchanged.

Owner: `isometry-system::active_effects`. The substrate stores uninterpreted
system state; the plugin owns the accepted procedures and lifecycle. Uses
existing sheets, inventories, item IDs, token IDs, turn counters, dice,
terrain height visibility, body documents and native clock/sky reads.

## Automated and runtime evidence

Commands ran from the repository root using the existing target and
`CARGO_HOME=C:/t/cargo-homes/isocosm-foraging`. No isolated target, Cargo home
or worktree was created.

```text
cargo +1.99.1 test -p isometry-core --lib --offline --locked -j 3
54 passed

cargo +1.99.1 test -p isometry-system --lib --test effect_catalogue --test active_effects --offline --locked -j 3
50 library + 4 catalogue + 15 active-effect tests passed

cargo +1.99.1 check -p isometry-genet --all-features --all-targets --offline --locked -j 3
passed, including the edited desktop consumers

cargo +1.99.1 run -p isometry-system --example cast_effects --offline --locked -j 3
9 activated, saved/restored, suppressed/resumed and expired cases

cargo +1.99.1 run -p isometry-system --example inspect_effects --offline --locked -j 3 -- --json
54 entries; 15 reviewed related mappings; 147 unmapped; 9 executable bindings

scripts/line-ceiling.ps1 -Staged
scripts/pack-json.ps1 -Staged
responsa regen --stage
responsa lookup "functional-generation D1"
responsa check
passed; decision manifest current, doctrine within budget
```

The initial offline desktop check lacked `iroh 1.4.1`. A locked online check
fetched the existing lockfile dependencies and passed; the final offline check
then passed too. Cargo.lock adds only the existing `isometer-core` dev
dependency to `isometry-system`, with no version changes. Existing dependency
and dead-code warnings remain. Responsa reports the pre-existing unknown
`refines orthographic-voxel-presentation D1` link in the presentation plan.

The tests establish atomic refusal including RNG, idempotent requests,
source-definition snapshots, unsupported/tampered-state refusal, native solar
boundary reading without advancing the world, duration units, action/resource
costs, unwilling-target saves, concentration replacement/damage/incapacitation,
independent suppression, dispel causes, overlap priority and weaker-effect
resumption, visibility through actual height traces, exact part/joint scaling,
weapon dice/bonuses in actual D&D/PF2e attack resolution, preserved equipment
identity and later base edits surviving expiry.

## Retained output

- [Execution cases](2026-10-28_effect_procedures.json), 132,231 bytes.
  SHA-256 `023007225aba8d398a70e2cd544c766af66e5a97c965d5cb895286d637b9ad2d`.
- [Coverage](2026-10-28_effect_procedures_coverage.json), 49,731 bytes.
  SHA-256 `7a98e00269e2e8c23832b985b565f918bb9eb08f3cd45a45e751ffca2e890dee`.
  Logical catalogue revision:
  `fca9769880b85b7b22b2812ce0c73a7cd7528342dd41cf4818c8405fbbd0beb5`.

The execution board is authored, seed 73, one-second clock units, a declared
next Sun boundary at 21,600 and daily preparations at 86,400. Its small body
is demonstration geometry, not an evolved creature. A separate integration
test reads the actual native sky to establish the calendar adapter. The nine
casts each use a fresh board so the two D&D concentration spells do not mask
one another; a separate test exercises their replacement.

## Scope of acceptance

This qualifies the headless API and existing D&D/PF2e action readers. It does
not qualify headed casting controls, multiplayer cast admission, automatic
concentration events in the desktop host, cross-map migration, native magic,
scene geometry/lighting consumers or a complete edition port. Ars weapon
changes expose +2 Damage and Magic Resistance/penetration information; a full
Ars combat resolver is not implemented. Every cast receipt is experimental.

The [supported profiles and source credits](../../../../crates/isometry-system/data/effects/README.md)
name the remaining exceptions: D&D confinement, object/equipment geometry and
Strength advantage, the existing critical-damage approximation, PF2e group
casting, Ars stress/botch, full resistance and laboratory devices. These are
F11c work, not implied by catalogue bindings or API tests.

Retained new receipts total less than 0.2 MB plus this summary, below F10/F11's
100 MB cap. Previous receipts were not overwritten.
