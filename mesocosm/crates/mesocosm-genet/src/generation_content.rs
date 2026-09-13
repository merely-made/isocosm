// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Shared content admission for generated starts and their command-line preview.

use mesocosm_core::{Founding, PartPalette, world::generation::Request};

/// Explicit structure uses the specimen bench's full shape vocabulary.
/// Earlier requests retain their original palette and saved seed streams.
pub fn palette(request: &Request) -> PartPalette {
    if request.criteria.structure.is_some() {
        Founding::SpacedRoster.palette()
    } else {
        Founding::Drawn.palette()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mesocosm_core::world::generation::Structure;
    use mesocosm_mesh::ContentPack;

    #[test]
    fn structure_admits_the_bench_palette_without_changing_earlier_requests() {
        let mut request = Request::default();
        assert_eq!(palette(&request), Founding::Drawn.palette());
        request.criteria.structure = Some(Structure::default());
        assert_eq!(palette(&request), Founding::SpacedRoster.palette());
        assert_ne!(palette(&request), Founding::Drawn.palette());
        let pack = ContentPack::generate(palette(&request)).unwrap();
        let recorded = serde_json::to_vec(&pack).unwrap();
        let restored: ContentPack = serde_json::from_slice(&recorded).unwrap();
        assert_eq!(restored, pack);
        assert!(restored.resolve_for(pack.palette).is_ok());
    }
}
