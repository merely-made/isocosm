# Private pack-authoring syntax backend

Vendored taplo 0.14.0 (MIT), upstream commit
`15963fcb555af6bb543e79caf664c3daa110f2ad`. The crates.io archive SHA-256 is
`c221a50eef1a5493074f11ca1ed62bef28c05a4d925002944cc686b2e783a5b3`.

Changes: pin Rowan 0.17.0; batch consecutive root/inline-table entries before
publication using unchanged per-entry conflict checks. Counter instrumentation
from the experiment is excluded. This local dependency is optional, unpublished,
and used only by the private pack-author command. It is not a shared stack API
or a new authoritative TOML decoder. Product format/schema validation stays in
the existing TOML/JSON loader. Publishing this tool feature requires a separately
qualified distribution of the fork; crates.io taplo 0.14 is not this backend.

The isolated baseline/differential/width receipts remain at
`Code/testing/retained-dom-batch-20261016`. Keep the upstream file layout so
updates and the focused batching patch stay reviewable. LICENSE.upstream retains
the upstream MIT notice.
