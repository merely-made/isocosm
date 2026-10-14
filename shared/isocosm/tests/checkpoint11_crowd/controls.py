"""Finite compiled C11 source faults. Root owns compilation and receipts.

describe only reads. apply/restore require an exact current-file SHA, replace
one declared byte span, and print the complete inverse and resulting hashes.
This helper runs no subprocess and creates no directory or auxiliary file.
"""

import argparse
import hashlib
import json
from pathlib import Path

CRATE = Path(__file__).resolve().parents[2]
TRANSPORT = "src/probe/crowd/spreads/transport.rs"
FAULTS = {
    "zero-route": (
        TRANSPORT,
        "site.routes.iter().filter(|route| route.transmission != 0).collect()",
        "site.routes.iter().filter(|route| route.transmission <= 1_000_000).collect()",
        "independent_territory_split_preserves_zero_route_and_rounding_residues",
    ),
    "early-credit": (
        TRANSPORT,
        "if transfer.arrives > tick { pending.push(batch.clone()); continue; }",
        "if false { pending.push(batch.clone()); continue; }",
        "independent_surface_delivery_credits_only_after_authored_travel",
    ),
    "wrong-rate": (
        TRANSPORT,
        "let amount = (u128::from(offer) * u128::from(route.transmission) / 1_000_000) as u64;",
        "let amount = (u128::from(offer) * u128::from(route.transmission) / 1_000_000) as u64 + 1;",
        "counted_hosts_round_each_offer_before_multiplying_the_bin",
    ),
    "drop-pending-mass": (
        "src/probe/crowd/spreads.rs",
        "+ self.details.transit.iter().map(|batch|\n"
        "                u128::from(batch.transfer.amount) * u128::from(batch.count)).sum::<u128>()",
        "+ 0u128",
        "independent_territory_split_preserves_zero_route_and_rounding_residues",
    ),
    "foreign-strain-host": (
        "src/probe/crowd/spreads/strain.rs",
        "crate::meaning::credit(self.ledger_mut(at)?,&key,amount)?;",
        "let wrong = match at { Substrate::Host(id) => Substrate::Host(id+1), other => other };\n"
        "        crate::meaning::credit(self.ledger_mut(wrong)?,&key,amount)?;",
        "seeded_strain_forks_only_selected_host_and_retains_local_functions",
    ),
    "wrong-strain-seed": (
        "src/probe/crowd/spreads/strain.rs",
        "spread::strain_key(seed,parent,at,&traits)",
        "spread::strain_key(seed.wrapping_add(1),parent,at,&traits)",
        "seeded_strain_forks_only_selected_host_and_retains_local_functions",
    ),
}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("describe", "apply", "restore"))
    parser.add_argument("fault", nargs="?", choices=FAULTS)
    parser.add_argument("--expected", help="mandatory exact current SHA for mutation")
    args = parser.parse_args()
    names = [args.fault] if args.fault else list(FAULTS)
    if args.action != "describe" and (not args.fault or not args.expected):
        parser.error("apply/restore require one fault and --expected current SHA")
    rows = []
    for name in names:
        relative, original, changed, test = FAULTS[name]
        path = (CRATE / relative).resolve(strict=True)
        if not path.is_relative_to(CRATE):
            raise ValueError("control source is outside its native crate")
        before = path.read_bytes()
        row = {"fault": name, "path": str(path), "test": test,
               "source_sha256": digest(before), "original": original,
               "replacement": changed, "inverse": original,
               "original_matches": before.count(original.encode("utf-8"))}
        if args.action != "describe":
            if digest(before) != args.expected:
                raise ValueError("current source does not match the explicitly supplied SHA")
            old, new = (original, changed) if args.action == "apply" else (changed, original)
            old = old.encode("utf-8")
            new = new.encode("utf-8")
            if before.count(old) != 1:
                raise ValueError("control byte span is absent or ambiguous")
            after = before.replace(old, new, 1)
            path.write_bytes(after)
            if path.read_bytes() != after:
                raise ValueError("control write did not retain exact offered bytes")
            row.update(action=args.action, after_sha256=digest(after))
        rows.append(row)
    print(json.dumps(rows, indent=2))


if __name__ == "__main__":
    main()
