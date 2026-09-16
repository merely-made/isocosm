# Equipment persistence receipt (retired)

A two-process check: one run of the retired `body_sheet` binary attached a
dressing and published a save, a second run loaded it and drew the restored
attachment. `run-1788911337963/` keeps the published save and the two
composited captures from 2026-09-08.

The binary was retired on 2026-09-15 under the
[isomere plan](../../../mesocosm/design_docs/2026-09-15_isomere_plan.md)'s M5;
the rationale is in
[the genet document host plan](../../design_docs/2026-09-13_genet_document_host_plan.md),
under "Retiring the body sheet".

What survives of the claim: the store itself is
`paredros_client::equipment_store`, and
`disk_roundtrip_restores_attachment_into_a_fresh_game` proves that a published
save restores the same accepted game with its attachment intact, over the
session's own world. That a *second process* reads it is no longer proved
anywhere — a stated cost of the retirement, and these files are the last
evidence of it.
