# Session host acceptance

Scenario acceptance for the `session` bin — the Cambium document host of one
played Paredros session (document-host plan P3, lane P3b). The bin is the
second consumer of `shared/wing-scenario` after Mesocosm's bench; everything
here is driven through that shared lane.

## Running

From `eponym/`, sequentially, artifacts wherever you want them:

```sh
cargo build -p eponym-client --bin session --target-dir target-contact

./target-contact/debug/session --scenario testing/session/acceptance.scenario \
  --receipt <out>/acceptance.json --capture <out>/acceptance.png \
  --frames 1800 --size 1240x820          # prints RESULT ok, exits 0

./target-contact/debug/session --scenario testing/session/failure.scenario \
  --receipt <out>/failure.json --capture <out>/failure.png \
  --frames 600 --size 1240x820           # prints RESULT fail, exits 1
```

`--scenario`, `--receipt`, `--capture`, `--frames` and `--size` mirror
Mesocosm's bench flags; `--help` prints them. With none of them the window is
interactive, and `PAREDROS_SESSION_SMOKE=1` still runs the P2 in-window smoke
instead (unchanged by this lane). A driven run with no `--receipt`/`--capture`
writes *scratch* names in this directory, so it can never overwrite a kept
acceptance artifact.

## What each scenario proves

`acceptance.scenario`

- The document presents: `ready`, no producer error, and the sheet, items
  and status panels are on the surface. Captures `opened`.
- Movement is drawn at once and reaches the accepted world at the minute
  (cases 1118, 1120): after one `act move-right` the drawn position differs
  and the step counter grew while the `GameState` hash is unchanged; after
  `act minute`, a fixture control that takes the round's minute now, a
  `motion-advanced` event was accepted and the hash differs.
- The charged volley through the keyboard-only vocabulary — aim, join part 2,
  charge, strike — severs the target's authored limb (`target-parts` shows
  `severed`), drops `target-vitality`, and changes the hash. Captures
  `volleyed`, and a `viewport-pixels-change` against `opened` proves the drawn
  viewport moved with it.
- Hagioglyph (F3b5): `act revise-canon` publishes one authored canon revision
  with a fixed seed — a fixture control, not a product action. Afterwards
  `canon-revision` has moved, the acquired glyph and its count have not, the
  strike's founding effect `paredros-fixture:reach` is still on the surface,
  and `glyph-last-live-effect` is the revised `paredros-fixture:fasten`.
- Treatment through the controls themselves: `Take item 69` (the dressing at
  the keeper's cell, in the Items panel) and `Rest` are clicked as buttons, through the host's own pointer routing, and afterwards
  `rested` is accepted and the played subject has one `lost-part` from the
  injury cut, its `wound` share 7: a severed part's whole lattice stays in
  it (the 2026-10-18 note to ruling 227; case 1132).
- A part button in the sheet selects the same identity a viewport pick would:
  `selected-subject` 701, `selected-part` 3.
- `Save` then `Load`, both clicked, round-trip one `GameSave` version: a move
  between them changes the hash, and the load restores it exactly
  (`same saved hash`). Captures `final`.

`failure.scenario` is the negative control: one deliberately false assertion
(`ready == impossible`). The run must exit 1 with `"ok": false` in the receipt
and still write a fresh capture.

## Artifact layout

- `receipts/<date>/` — the compact receipt JSONs kept as evidence. PNGs are not
  committed; they are large and regenerate from the same command.
- Scratch runs land as `scratch_session.json` / `scratch_session.png` here.

## Still open

Physical keyboard and mouse acceptance — a person at the window — is recorded
separately and remains open, as the plan's P3 done-condition says.
