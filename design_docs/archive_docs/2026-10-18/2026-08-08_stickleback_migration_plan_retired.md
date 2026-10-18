# Stickleback Consumer Migration Plan: what moved out on 2026-10-18

On 2026-10-18 Mark made the old numbered rulings cases (precedent, numbered 1
to 984 in the design record), kept new rule numbers for himself, and asked
every active doc to lose its stacked status banners, old progress entries,
stale counters and pasted copies. This file holds, verbatim, what left the live
[stickleback migration plan](../../2026-08-08_stickleback_migration_plan.md):
the 2026-10-17 status paragraph, the current-state section and the dated H7
notes under it, the original plan-status paragraph, and the progress entries of
2026-10-10 (Q1) and 2026-08-08. Relative links in the moved text are kept as
written, so they resolve from the live doc's folder, not from this one.

## Status banners

**Status, 2026-10-17:** K0 to K2 remain unimplemented. H7 uses JoinedSpace
for the separate single-host family accepted-record lane; `campaign_sync`
still assembles `LogSync` and `SyncedSpace` directly. The shipping Genet UI
now qualifies one native VTT action and an exact same-campaign ledger restart;
production peer admission/setup and power-loss durability remain open. The
no-second-runtime gate still governs campaign sync.

## Current state, 2026-10-17

`crates/isonetry/src/campaign_sync.rs` still assembles `LogSync` and
`SyncedSpace` directly. K0's JoinedSpace adoption is therefore open;
native simulation and the V2 assertion translation do not close it.

Campaign grammar, authorization, materialization and tactical sequencing
remain the VTT's; Stickleback supplies membership, log transport and sync.
K1 still requires concurrent and refused-operation receipts on the new
carrier. K2 still requires named dispositions for the older tiers.
The native campaign owner during migration is
`shared/isocosm/src/legacy/campaign.rs`; V2 will remove it after native
polities act, while campaign storage/collaboration are product-side.
That ownership change must not create a second live world runtime or
silently drop the authorization and privacy gates.

*2026-10-17:* H7's host-signed family-record lane uses JoinedSpace, but its
scope is single-host World replication. It does not implement this plan's
multi-writer campaign sync; K0 to K2 remain open.

*2026-10-17, H7 local ledger:* `FamilyHost::create_file` and
`FamilyHost::resume_file` now keep a bounded checkpoint and accepted-record
history for process restart. The core World commit gate replaces the synced
ledger before publishing a World change; a failed replacement or byte cap
leaves the World and journal unchanged. Restart verifies and replays the
stored suffix, and a signed-log control proves deterministic re-signing and
the next sequence/backlink. The caller must retain the original host key and
use the correct host-log backlink. Same-directory replacement and file sync
are qualified; the parent directory is not synced, so power-loss durability
is not claimed. Receipt:
`Code/testing/isometry/h7-2026-10-17/durable-ledger-receipt.json`.

*2026-10-17, H7 product restart/reconnect control:* the actual VTT endpoint
was reconstructed from the opt-in file ledger, then a fresh admitted
JoinedSpace lane installed the signed checkpoint and received the persisted
suffix. A further offered `vtt.leave` intent produced the next native family
record with the expected sequence and backlink. The focused family-network
gate passed 5/5. This reconstructs host and transports inside the test
process; it does not qualify production network process-kill recovery or
power-loss durability.
The headed Genet UI now exercises native endpoint authority with the durable
family ledger, but production invitation/bootstrap and peer setup remain open.
Receipt:
`Code/testing/isometry/h7-2026-10-17/durable-product-restart-receipt.json`.

*2026-10-17, H7 shipping native UI and file-ledger restart:* the headed Genet
app accepted one native VTT action at sequence 2/revision 2, preserved the
opaque VTT section, and verified an exact native suffix and ledger replica.
Reopening the same campaign resumed durable mode at the same sequence,
revision and native hash. The logs, binary binding, two inspected captures
and `shipping-native-ui-restart-qualification.json` are under
`Code/testing/isometry/h7-2026-10-17/`. This app-only dev opt-level 0 run is a
functional UI receipt, not a performance or peer-admission qualification.
The family stack still needs a production admission provider and shipping
peer setup; full-World authorization remains separate, and power-loss
durability is not claimed.

*The dated migration design and receipts below remain as written (793).*

**Status: plan.** Founded from the 2026-08-08 wing audit. The
[shared-authority plan](2026-07-09_shared_authority_and_collaborative_building_plan.md)'s
**sequencing gate stands and governs**: no second Isometry runtime. Its
earlier tiers (host-owned private stores, peer-side Lua revalidation,
regenerated secrets, commit-reveal mechanics) are **superseded by this
plan** and stamped so at the source.

## Progress entries

- **2026-10-10, Q1:** current ownership, gates and status checked against
  the lane tree under 793 to 802; earlier dated text retained. No new
  compile, test, draw or headed receipt claimed.

- **2026-08-08:** founded from the audit; shared-authority plan stamped.
