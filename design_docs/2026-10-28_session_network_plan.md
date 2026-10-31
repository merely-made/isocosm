# The family session layer

**Status:** 2026-10-28: cut from `mesocosm/design_docs/2026-10-14_sim_host_plan.md` (H7),
`2026-08-08_stickleback_migration_plan.md`,
`2026-07-09_shared_authority_and_collaborative_building_plan.md` and
`2026-10-15_campaign_retirement_plan.md` (the H7 follow-on), which are archived
in their products' `archive_docs/2026-10-28/`. `isonetry` has begun its remake:
`family`, `family_network` and `source_time` modules exist, with a host-signed
record lane over `JoinedSpace` and in-process and loopback tests. Open are the
production admission and peer setup, the network gate, retiring the old VTT
event authority, the Stickleback adoption of `campaign_sync.rs` (K0 to K2) and
the demand-gated tiers of the shared-authority plan.

**For:** one session layer for every product: one host sequences each world and
replicas take the accepted log (H7); the multi-writer campaign carrier moving
onto Stickleback `JoinedSpace` (K0 to K2); the standing gate that no step mints
a second runtime; the deferred authority tiers and creative-mode designs, and
the open questions on them. Governing statements: doctrine Host 5 (the family
session) and Record 1 (consistency is chosen per domain).

Not here: how the VTT campaign reads the sim over this layer (`vtt-overlay V2`);
where the GM store and proposal lifecycle go on retirement
(`campaign-retirement R2b`); pack distribution over the carrier
(`content-packs`); every tick, frame and search-time bar (`sim-execution`).

## Decisions

- **D1, no rule against CRDTs (2026-10-28), refines Record 1:** Mark: "No CRDTs never made any sense. It was a stupid rule to begin with. And just a misunderstanding as well to begin with. So no, that rule is gone. Get rid of that rule." Which operation families want true CRDT types stays a design question (Questions); none is ruled out.

  Links: area Host and network · draws on record.1, host.5 · summary The no-CRDTs rule is withdrawn; whether an operation family needs a true CRDT is decided on its merits.

## Constraints

- **The no-second-runtime gate stands and governs** (shared-authority plan,
  re-scoped 2026-08-08; Stickleback plan stop rule). In one sentence: no step
  may create a second Isocosm actor, host composition or private-delivery path
  beside the existing host session and its tactical sequencer; tactical play
  keeps the existing sequencer, and a step that would do otherwise stops. The
  archived plan states it as "Do not add a second Isocosm actor or
  private-delivery path against the retiring host-composition seam." It is
  defined nowhere else, so this sentence is its definition here. Source:
  `archive_docs/2026-10-28/2026-07-09_shared_authority_and_collaborative_building_plan.md:4-14,326-330`;
  `archive_docs/2026-10-28/2026-08-08_stickleback_migration_plan.md:52-57`.
- **One host sequences each world; a replica never reruns a product adapter to
  decide an intent** (doctrine Host 5; rulings 959, 960, 962). Requests use the
  admitted stream and Graphshell requests; accepted records use Stickleback
  joined spaces; one host stays the sole accepting writer. Replicating the full
  world is a separate grant from seeing a projection or holding a seat. The
  stack supplies protected identity and admitted connections; games map a peer
  to participant and seat; products never manufacture keys, grants or trust
  roots (962).
- **Consistency is chosen per domain** (doctrine Record 1). Where order matters
  one sequencer accepts intents against a revision; where it does not, writers
  sign concurrent operations and a type-specific materializer keeps competing
  work. The sequencer is a role, a lease or a turn, separate from edit
  authority and from the network role. Mark (Record 1): "client/server,
  host/guest are networking and authority axes intersecting. and peers can sit
  anywhere on either axis ... there could be two players with GM authority".
- **A rejected intent advances no accepted sequence; pending client intent
  stays a proposal until its host record arrives.** Source:
  `mesocosm/design_docs/archive_docs/2026-10-28/2026-10-14_sim_host_plan.md:289-315`.
- **Version negotiation refuses, never degrades**, and no peer derives a
  consequence (the H0 and H1 laws that `vtt-overlay` keeps for the VTT
  protocol).
- **Signature verification happens once, in `AnyHeader::decode`** in the owned
  fork `mere-p2panda-net-0.7.2`; `Header::verify` is test-only, and mutating
  the in-memory `header.extensions` changes neither signed bytes nor
  `header.hash()`. K0 must not carry that assumption implicitly.
- **Dependency discipline:** `mere-p2panda-net =0.7.2` exact dep line and
  feature set repeated by every consumer; the fork tag kept in lockstep with
  mere's, not a branch.
- **Authority tier 3 votes do not govern downtime**: ruling 246, downtime needs
  each player's yes, not a majority.

## Phases

### H7. The family session layer
`isonetry`, remade, is the family's session layer (923, 924). One peer hosts
the core's `World`; others send intent envelopes and receive the accepted log.
The family stack owns authenticated peer identity and admitted connections;
each game maps an authenticated peer to its native participant and seat.
Projection or seat access does not grant full-World replication (a separate
gate). This is the route for the VTT and later the other games, not evidence
that all three have shipped peer play. Open: production stack admission and
peer setup. The sub-phases below are its gates.
- Done when: (draft, "to be set by that assessment") "a world hosted on one peer accepts intents from a second over the stack's transport through each product's endpoint, both peers reach the same world revision and state hash, and the VTT's table runs on it with the old DM-authority core retired."
- Needs: rulings 959, 960 (recommended paths chosen 2026-10-17); K0 to K2 below
  (`campaign_sync.rs` still assembles `LogSync` and `SyncedSpace` directly); 921
  (the GM store and proposal lifecycle stay in `isonetry` until V2, outside the
  new family API); mere transport (murm, `mere-transport`)
- Rulings: 921, 923, 924, 959, 960
- From: `mesocosm/design_docs/archive_docs/2026-10-28/2026-10-14_sim_host_plan.md:248-315`.

### H7a. Atomic replication boundary and serialized peer records
The first implementation gate: two in-process peers over serialized records
covering native plus section-only changes, unchanged bytes, refusal,
duplicate, gap and out-of-order delivery, checkpoint resume, and an unknown
section. Chosen design (959, 960): publish the accepted native log suffix and
atomic opaque section updates; the client applies authority without
re-deciding. Needed: explicit session and checkpoint identity, a monotonic
record sequence and parent digest, a native-log cut, a section digest and the
recorded revision at the checkpoint; section deletion is an explicit update;
reconnect installs one checkpoint and checks the contiguous suffix before
exposing the cut. Code and tests exist for much of this (see Findings); what
remains is checking each listed case against the done-condition.
- Done when: "Both peers must match accepted sequence, native hash, section digest and world revision."
- Needs: H7 assessment forks (959, 960); a live `World` revision resets when its save is resumed, so the checkpoint carries its own revision
- Rulings: 959, 960, 923
- From: `mesocosm/design_docs/archive_docs/2026-10-28/2026-10-14_sim_host_plan.md:289-315`.

### H7b. Network gate
The actual consumed transport and endpoint, before each product and the VTT
table claim acceptance.
- Done when: both peers match accepted sequence, native hash, section digest and world revision (as H7a), over the real transport
- Needs: H7a
- Rulings: 923, 924
- From: `mesocosm/design_docs/archive_docs/2026-10-28/2026-10-14_sim_host_plan.md:311-313`.

### H7c. The old VTT event authority retires
Until H7 lands, the Genet app still uses the old VTT event authority and
`isonetry` keeps its VTT job plus the GM store and proposals. The remake is a
separate change from the campaign retirement; this phase is where "the old
DM-authority core retired" is made true. VTT-only parts move to `isocosm-native`
(924).
- Done when: the last clause of H7's condition: the VTT's table runs on H7 "with the old DM-authority core retired."
- Needs: H7b; `campaign-retirement R2b`; `vtt-overlay V2`
- Rulings: 921, 923, 924
- From: `archive_docs/2026-10-28/2026-10-15_campaign_retirement_plan.md`
  (the H7 follow-on; harvest src `:71-77,91-94`).

### H7d. Production admission, setup and durability
The shipping Genet UI qualifies one native VTT action and an exact
same-campaign ledger restart. Production peer admission and setup, and
power-loss durability, stay open. H7's family-record lane uses `JoinedSpace`
only for single-host World replication; it does not implement multi-writer
campaign sync (that is K0).
- Done when: not stated; owed
- Needs: K0; H7c
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-08_stickleback_migration_plan.md:3-12`.

### K0. `JoinedSpace` adoption
`campaign_sync.rs` consumes `JoinedSpace`; direct `LogSync` and `SyncedSpace`
assembly is deleted. K0 removes the direct assembly, not endpoint binding:
`JoinedSpace::join` still takes the host's already-created p2panda `Endpoint`
and `Gossip`. The loaded-machine flake in the host bridge test belongs to the
bridge receipt, not to this. `CampaignSpace::to_operation` is already shaped
like gemot's `to_operation_seed`, so the rebase is a lift; the one
Isocosm-only shape is the three-field `CampaignExt` with its `parents` DAG.
Isocosm keeps its domain grammar, authorization, materializer and tactical
sequencer; Stickleback owns membership, log transport and sync. A K0 receipt
should show the decoded-only admission path explicitly (Constraints).
- Done when: "the existing multi-writer campaign receipts re-green on the new path and the deleted assembly has no survivors (grep receipt)."
- Needs: the no-second-runtime gate; the dependency discipline above
- Rulings: the gate; W1 2026-09-18 "keep"
- From: `archive_docs/2026-10-28/2026-08-08_stickleback_migration_plan.md:35-38,61-92`.

### K1. Authority receipts on the new carrier
Authorization, materializer and conflict handling proven unchanged over
Stickleback transport.
- Done when: "a two-writer concurrent-edit receipt and a refused-unauthorized-operation receipt both pass on `JoinedSpace`."
- Needs: K0
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-08_stickleback_migration_plan.md:40-44`.

### K2. Superseded-tier sweep
The old tiers' machinery (private stores, peer revalidation, secrets,
commit-reveal) is either deleted or explicitly retained with a named reason.
This is the same work as dispositioning Tiers 1 to 3 below.
- Done when: "the shared-authority plan's tier sections carry per-tier dispositions and the code matches them."
- Needs: K0, K1; the tier items below (T1 to T3)
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-08_stickleback_migration_plan.md:46-50`.

### AU1. Retraction before strangers co-author
`Retract(fact_id)` tombstone: views stop showing the fact and snapshot
compaction removes it from the active checkpoint. The promise is active-state
removal plus a clear local retention policy, not retroactive deletion. No
`Retract` exists in code.
- Done when: not stated; owed
- Needs: the compaction question (Questions)
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-09_shared_authority_and_collaborative_building_plan.md:285-293`.

### AU2. Migrate collaborative domains one at a time
Drafts and facts, maps, packs, dialogue, inventory and equipment, then world
simulation records. Each needs an operation grammar, validation, a
deterministic materializer, conflict-preserving UI, policy hooks, retraction
semantics and offline and convergence tests. Only the campaign proposal,
endorsement and recognition shape exists.
- Done when: not stated; owed
- Needs: K0 (the carrier first); the CRDT question (Questions)
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-09_shared_authority_and_collaborative_building_plan.md:295-302,331-333`.

## Deferred and parked
Demand-gated tiers of the shared-authority plan. K2 must say for each whether
it is superseded by the Stickleback plan, retained, or deleted. The peer-side
commit-reveal and regenerated secrets are marked superseded already. Parked
items here wait on demand, Personae or the V2 world; nothing here is parked
parked (`wing-organs` D1).

- **T1. Transferable host role (Tier 1)**: late-join in reverse: the outgoing
  host freezes intents, transfers `GameSnapshot`, sequence, log hash and the
  private `CampaignStore`, peers reconnect; secret custody moves with the role;
  sequencer role and edit-mode membership are separable transfers. Host-owned
  `CampaignStore` persistence through a Muniment checkpoint exists; live
  transfer and a Personae-backed signed host grant do not (the Personae
  capability layer is not a dependency). Done when: "a session survives a live
  host handoff with both peers' convergence hashes intact, the new host can
  reveal a secret the old host authored, and a rejected mid-handoff intent is
  retried successfully against the new host." Doctrine Record 1 already says
  the sequencer can be handed off. Waits on K2's disposition, a Personae signed
  grant and the host-handoff transport question. From:
  `archive_docs/2026-10-28/2026-07-09_shared_authority_and_collaborative_building_plan.md:61-98`.
- **T2. Mechanical authority (Tier 2)**: every peer validates every intent;
  turn structure as leader schedule; honesty model detectable and attributable.
  Done when: "a full combat resolves with sequencing rotating per turn, all
  peers validating locally, and an injected invalid event being rejected by
  every peer independently (not just by a privileged host)." Gated on demand
  and Personae; needs K2 and the tactical sequencer recast (T4). From: same
  file `:100-121`.
- **T3. The app runs the campaign (Tier 3)**: commit-reveal dice; rulings by
  table action; secrets committed now, audited later via seed-replay. Done
  when: "a four-player, no-DM one-shot completes: seed sealed at start,
  commit-reveal dice throughout, one voted ruling, one storylet whose reveal
  matches its commitment, and a post-game audit that reproduces the hidden
  layer from the unsealed seed." Gated on determinism discipline (the W2 /
  campaign-packs decision 12 discipline); "majority vote" for table actions
  conflicts with ruling 246 for downtime only. From: same file `:123-153`.
- **T4. Recast the host session as the tactical sequencer** (sequencing 5): a
  signed lease transfer and deterministic peer validation, without making the
  lease holder campaign owner. Waits on T1, T2. From: same file `:334-336`.
- **T5. Commit-reveal randomness and sealed or auditable secrets** (sequencing
  6) for tables whose configured policy requires them. Waits on T3. From: same
  file `:337-338`.
- **CR1. Creative mode as a graph-tending game**: per-channel, per-player modes
  as permission-plus-rulebook presets; mode flips as logged events; attachment
  rule as floor; pack-authored shape constraints per kind; gaps as the prompt
  deck; rounds, focus, palette, veto; tombstones as the X-card floor; playing a
  faction as survival over the faction-turn channel. Survival, edit and
  creative are the three modes; none of the creative machinery exists in code.
  Waits on `vtt-overlay V2` (the sim holds the graph; attachment rules need the
  place and relationship graph), pack data and AU1. Edit mode is on by default
  at every table (156, 245). From: same file `:155-248`.
- **CR2. Prep-time co-authoring**: the preview table goes multiplayer (any peer
  proposes and locks fields; commit gate DM in Tier 1, quorum in Tier 3); a
  draft channel (`WorldFact` kind "draft") baked into pack data on acceptance;
  attribution as provenance. Waits on CR1 and worldbuilding generator
  governance. From: same file `:250-265`.
- **CR3. Session-time truths**: Microscope-style history rounds; Fate-style
  declarations (cost validated by the system plugin, acceptance by the table's
  commit gate); player-authored secrets entering the GM layer. Waits on CR1.
  From: same file `:267-283`.
- **Murm items deferred by the 2026-07-12 Murm peer-runtime plan**: secret
  publication and receipt, payload sealing, cabal-key rotation after removal and
  desktop actor wiring. `murm-replication` must own accepted-operation
  processing, service lifecycle, retention, checkpoints and native-drop import
  before Isocosm rebases as a later domain consumer. Same-Moot founding
  conflicts use that Moot's admission context; unrelated founding Moots stay
  restricted. Waits on `murm-replication` (mere) and K0. From: same file
  `:313-330`.

## Questions for Mark
- **CRDT data types**: which operation families need true CRDT types beyond
  set-valued folds? Map cells, ordered dialogue choices, inventory uniqueness
  and counters must each name their conflict rule before AU2. From:
  `archive_docs/2026-10-28/2026-07-09_shared_authority_and_collaborative_building_plan.md:369-371`.
- **Host handoff transport**: does iroh let peers re-dial a new node cleanly
  mid-session (a new ticket over the old connection), or does Tier 1 v1 accept
  "everyone rejoins from the new host's ticket"? From: same file `:361-363`.
- **Retraction against the convergence hash**: does compaction after a
  tombstone need a resync checkpoint event so late joiners' hashes converge?
  The old plan's reading: "(Likely yes: compaction mints a new snapshot
  baseline, the same shape as a late join.)" From: same file `:372-376`.
- **Default Moot policy for recognizing a proposal**: the evaluator supports any
  eligible member, a fixed threshold, a fractional threshold and unanimity;
  explicit owner or grant policy waits on the Personae capability layer; packs
  or a signed Moot policy operation must select the default. From: same file
  `:364-368`.
- **Tier dispositions (K2)**: for each of Tiers 1 to 3, superseded by
  Stickleback, retained with a named reason, or deleted. The old plans mark the
  private-store, peer-revalidation, regenerated-secret and commit-reveal
  machinery superseded but leave the text until K2. From:
  `archive_docs/2026-10-28/2026-08-08_stickleback_migration_plan.md:46-50`.

## Findings
- 2026-10-28: `crates/isonetry/src/campaign_sync.rs` still imports
  `murm_replication::SyncedSpace` and `p2panda_net::LogSync` directly (lines 3,
  7); K0 is open. `JoinedSpace` appears only in `family_network.rs`
  (single-host family records).
- 2026-10-28: the H7a cases exist as tests: duplicate, gapped, reordered and
  checkpoint-resumed records are checked in `crates/isonetry/src/family/tests.rs`
  (`assert_converged` compares entries, tick, state hash and section digest);
  unknown format versions refuse in the same file; a loopback test over an
  admitted Graphshell stream and a host-signed lane
  (`family_network/tests.rs`) exists. What was not found: a run over the
  production transport on two machines, production admission, or the VTT table
  running on this layer.
- 2026-10-28: `PROTOCOL_VERSION` is 22 (`crates/isonetry/src/protocol.rs:26`);
  `isonetry`'s default feature is still `vtt`, which carries the old VTT API and
  `isocosm-campaign`.
- 2026-10-28: no `Retract`, host handoff or commit-reveal code was found by
  grep (checked against the code today).
- 2026-10-28: the shared-authority plan's sequencing items 1 and 2 are "landed
  locally 2026-07-11" but cite Moot, Murm and Personae pieces; the plan itself
  says Tier 1 still needs live transfer.
- 2026-10-28: the "no CRDTs" line is gone from the three CLAUDE.md files (D1).

## Progress
- 2026-10-28: cut from the sim host plan's H7, the Stickleback migration plan,
  the shared-authority plan and the campaign retirement plan's H7 follow-on.
