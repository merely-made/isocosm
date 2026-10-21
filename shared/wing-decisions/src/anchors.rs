// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The anchor check: does the code a decision governs still exist? Reads a
//! rust-analyzer SCIP index (protobuf, decoded by hand: `Index.documents` 2,
//! `Document.relative_path` 1 and `occurrences` 2, `Occurrence.range` 1,
//! `symbol` 2, `symbol_roles` 3, Definition = 1) and resolves qualified
//! anchors like `isocosm::stage::Simulation::stage`.

use regex::Regex;
use std::collections::BTreeMap;
use std::sync::LazyLock;

/// Definitions by crate-qualified path segments.
#[derive(Default)]
pub struct Index {
    /// `isocosm::stage::Simulation::stage` -> [(file, line)]
    pub by_path: BTreeMap<String, Vec<(String, u32)>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Found {
    /// At the anchor's own path.
    Present(Vec<(String, u32)>),
    /// Not at its path; the same owner and name, or the same name, elsewhere.
    Moved(Vec<String>),
    Gone,
}

static IMPL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"impl#\[([^\]]+)\](?:\[[^\]]+\])?").unwrap());
static GENERICS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^<>]*>").unwrap());

impl Index {
    pub fn load(bytes: &[u8]) -> Result<Index, String> {
        let mut index = Index::default();
        for (f, _, doc) in fields(bytes)? {
            if f != 2 {
                continue;
            }
            let mut path = String::new();
            let mut occs = Vec::new();
            for (g, _, v) in fields(doc)? {
                match g {
                    1 => path = String::from_utf8_lossy(v).replace('\\', "/"),
                    2 => occs.push(v),
                    _ => {}
                }
            }
            if path.starts_with("tests/") || path.starts_with("benches/") {
                continue;
            }
            for o in occs {
                let (mut sym, mut roles, mut line) = (None, 0u64, 0u32);
                for (h, w, v) in fields(o)? {
                    match (h, w) {
                        (1, Wire::Bytes) => line = varint(v, &mut 0).map(|n| n as u32 + 1).unwrap_or(0),
                        (2, Wire::Bytes) => sym = Some(String::from_utf8_lossy(v).into_owned()),
                        (3, Wire::Varint) => roles = varint(v, &mut 0).unwrap_or(0),
                        _ => {}
                    }
                }
                if let Some(s) = sym.filter(|s| roles & 1 == 1 && !s.starts_with("local ")) {
                    if let Some(p) = qualified(&s) {
                        index.by_path.entry(p).or_default().push((path.clone(), line));
                    }
                }
            }
        }
        Ok(index)
    }

    pub fn resolve(&self, anchor: &str) -> Found {
        if let Some(at) = self.by_path.get(anchor) {
            return Found::Present(at.clone());
        }
        let segs: Vec<&str> = anchor.split("::").collect();
        let krate = segs[0];
        let tail = |n: usize| -> Vec<String> {
            let want = segs[segs.len().saturating_sub(n)..].join("::");
            self.by_path
                .keys()
                .filter(|k| k.starts_with(&format!("{krate}::")) && (k.ends_with(&format!("::{want}"))))
                .cloned()
                .collect()
        };
        for n in [2, 1] {
            if segs.len() > n {
                let hits = tail(n);
                if !hits.is_empty() {
                    return Found::Moved(hits);
                }
            }
        }
        Found::Gone
    }
}

/// A SCIP symbol as `crate::seg::seg`, or None for symbols outside a crate.
/// `rust-analyzer cargo isocosm 0.1.0 stage/impl#[Simulation]stage().`
/// gives `isocosm::stage::Simulation::stage`.
pub fn qualified(symbol: &str) -> Option<String> {
    let mut parts = symbol.splitn(5, ' ');
    let (_, _, krate, _, desc) = (parts.next()?, parts.next()?, parts.next()?, parts.next()?, parts.next()?);
    let desc = IMPL.replace_all(desc, "$1/");
    let mut desc = desc.replace('`', "");
    while GENERICS.is_match(&desc) {
        desc = GENERICS.replace_all(&desc, "").into_owned();
    }
    let segs: Vec<&str> = desc
        .split(['/', '#', '.', '(', ')', '[', ']'])
        .filter(|s| !s.is_empty() && *s != "crate")
        .collect();
    let krate = krate.replace('-', "_");
    Some(std::iter::once(krate.as_str()).chain(segs).collect::<Vec<_>>().join("::"))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Wire {
    Varint,
    Bytes,
    Other,
}

fn varint(b: &[u8], i: &mut usize) -> Option<u64> {
    let (mut n, mut shift) = (0u64, 0);
    loop {
        let c = *b.get(*i)?;
        *i += 1;
        n |= u64::from(c & 0x7f) << shift;
        if c < 0x80 {
            return Some(n);
        }
        shift += 7;
        if shift > 63 {
            return None;
        }
    }
}

/// The fields of one protobuf message: (number, wire type, payload). A varint
/// payload is its own encoded bytes.
fn fields(b: &[u8]) -> Result<Vec<(u64, Wire, &[u8])>, String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let key = varint(b, &mut i).ok_or("truncated key")?;
        let (f, w) = (key >> 3, key & 7);
        let start = i;
        match w {
            0 => {
                varint(b, &mut i).ok_or("truncated varint")?;
                out.push((f, Wire::Varint, &b[start..i]));
            }
            2 => {
                let n = varint(b, &mut i).ok_or("truncated length")? as usize;
                let end = i.checked_add(n).filter(|e| *e <= b.len()).ok_or("length past end")?;
                out.push((f, Wire::Bytes, &b[i..end]));
                i = end;
            }
            1 => {
                i += 8;
                out.push((f, Wire::Other, &[]));
            }
            5 => {
                i += 4;
                out.push((f, Wire::Other, &[]));
            }
            _ => return Err(format!("wire type {w}")),
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbols_qualify_through_impls_variants_and_fields() {
        let q = |s: &str| qualified(&format!("rust-analyzer cargo isocosm 0.1.0 {s}")).unwrap();
        assert_eq!(q("stage/impl#[Simulation]stage()."), "isocosm::stage::Simulation::stage");
        assert_eq!(q("heap/impl#[Counting][GlobalAlloc]alloc()."), "isocosm::heap::Counting::alloc");
        assert_eq!(q("rules/effect/Effect#Sunlight#"), "isocosm::rules::effect::Effect::Sunlight");
        assert_eq!(q("simulation/Receipt#foregone."), "isocosm::simulation::Receipt::foregone");
        assert_eq!(q("lineage/impl#[`BodyRead<'a>`]part()."), "isocosm::lineage::BodyRead::part");
    }

    fn message(fields: &[(u64, &[u8])]) -> Vec<u8> {
        let mut out = Vec::new();
        for (f, v) in fields {
            out.push(((f << 3) | 2) as u8);
            out.push(v.len() as u8);
            out.extend_from_slice(v);
        }
        out
    }

    #[test]
    fn a_hand_built_index_resolves_present_moved_and_gone() {
        let def = |sym: &str, line: u8| {
            let mut o = message(&[(1, &[line, 0, 4]), (2, sym.as_bytes())]);
            o.extend_from_slice(&[3 << 3, 1]);
            o
        };
        let a = def("rust-analyzer cargo isocosm 0.1.0 stage/Stage#", 33);
        let b = def("rust-analyzer cargo isocosm 0.1.0 matter/impl#[Pool]drain().", 9);
        let doc1 = message(&[(1, b"src/stage/mod.rs"), (2, &a)]);
        let doc2 = message(&[(1, b"src/matter.rs"), (2, &b)]);
        let index = Index::load(&message(&[(2, &doc1), (2, &doc2)])).unwrap();
        assert_eq!(index.resolve("isocosm::stage::Stage"), Found::Present(vec![("src/stage/mod.rs".into(), 34)]));
        assert_eq!(index.resolve("isocosm::ecology::Pool::drain"), Found::Moved(vec!["isocosm::matter::Pool::drain".into()]));
        assert_eq!(index.resolve("isocosm::stage::Gone"), Found::Gone);
    }
}
