# CLAUDE.md — Isocosm repository

This file is derived, not authoritative (doctrine, Method 7): it says where
the records are and how to work here. A rule it mentions lives in a record,
linked.

## The repository

Three peer products over one simulator, Cosmograph: Isocosm (the VTT) at the
root, Mesocosm in `mesocosm/`, Eponym in `eponym/` (doctrine, Naming 1).
Each has its own Cargo workspace, kept separate to preserve source and patch
policy; read the product's `CLAUDE.md` before editing it. Shared sim and wing
crates are under `shared/`.

Use `./scripts/wing.ps1 isocosm <cargo arguments>` (or `mesocosm`/`eponym`)
to select a product. Tabletop-only machine overrides live in the ignored
`.cargo/tabletop-local.toml`, supplied explicitly by that command. Do not put
product-specific patches in `.cargo/config.toml`: children inherit it. Direct
Cargo commands from product directories also work. Runtime separation does
not prohibit shared libraries or atomic changes across products.

## Where the records are

| Record | Holds |
| --- | --- |
| `design_docs/doctrine.md` | What the wing holds and why, by area; read first |
| `design_docs/2026-09-18_wing_design_plan.md` | Every case (§0), Mark's words; `responsa lookup <n>` finds one |
| `design_docs/2026-07-30_games_wing_founding.md` | Vessels, care granularity (§1), the pipeline laws (§3), vocabulary (§7) |
| `design_docs/DOC_README.md` and each product's | Indexes, working principles |
| `design_docs/DOC_POLICY.md` | How docs and decisions are kept (one policy for all three) |
| each product's `design_docs/PROJECT_DESCRIPTION.md` | Goals and pillars (maintainer-owned) |
| each product's `LICENSES.md` | The licence grants (doctrine, Licensing 3) |

`responsa` (eidetic's decisions tool, run here through `shared/responsa`)
reads the records: `lookup <case | plan D-key | statement | path>`, `lookup
--area <name>`, `lookup --history <key>`. The pre-commit hook keeps
`design_docs/decisions.toml` and the case register current.

## Working here

- Rust: standard idioms. No `unsafe` without documented justification.
- Source files stay under 600 lines (doctrine, Method 3; the hook checks).
  A file over grows into a directory module with focused siblings (shared
  imports and fixtures at the roof, children on `use super::*`, visibility no
  wider than `pub(super)`), split along seams the code already has; a test
  file splits by the module boundaries of the subject it drives.
- Plans follow DOC_POLICY §8: dated, with done-conditions, in `design_docs/`.
  Never `.claude/plans/`.
- Check the Merely stack (mere, genet, netrender, woodshed, the wgpu-* repos,
  the sibling products) before writing a new module; name the owning layer
  before building anything app-local.
- Prefer runtime verification over long static tracing; surface a blocked
  diagnostic early.
- Names are not coined mid-session; naming rounds are deliberate (doctrine,
  Naming 3).
- Surface a scope change before adding features beyond the active plan's
  target.
- The three pipeline laws (founding record §3) bind every product.
- Licences: doctrine, Licensing 3; content that may ship: Licensing 1.

## The VTT

A pixel-art isometric virtual tabletop over the Cosmograph sim: a P2P map
editor and turn-based play substrate for tabletop systems. Product
description: `design_docs/PROJECT_DESCRIPTION.md`; vocabulary (map, tile,
token, tileset, campaign, system plugin, session): `README.md` and the board
plan. The substrate never knows what a hit point is; rules live in system
plugins (PROJECT_DESCRIPTION). The session model is DM authority plus an
ordered event log (`design_docs/2026-10-28_session_network_plan.md`).
The wing's default view is the 2:1 dimetric in quarter turns (doctrine,
Presentation 1).

```text
crates/
  isocosm-core/     Pure substrate model: grids, iso math, map documents,
                    session events. No I/O, UI, net or genet deps.
  isocosm-views/    Cambium view fns + CSS sheets (tilesets are stylesheets).
  isocosm-genet/    Native winit host; ISOMETRY_PROFILE=1 prints frame timers.
  isonetry/         DM-authority replication over a pure protocol seam, iroh
                    behind a feature.
  isocosm-system/   System plugins: schemas plus piccolo Lua rules, with the
                    5e SRD pack.
```

Voxel appearance lives in `shared/isometer/crates/isometer-mesh`. Keep
`isocosm-core` pure: no `wgpu`, `iroh`, genet crates or file I/O.

**Guard the feature-gated code.** Every campaign feature is `default = []`,
so a plain `cargo test` compiles none of them; run
`cargo check --workspace --all-features --all-targets` after touching a
sibling repo and before committing anything in `isonetry`. On 2026-07-17 all
four campaign features were found uncompilable behind the default build.

sem is described once in `Code/CLAUDE.md`.
