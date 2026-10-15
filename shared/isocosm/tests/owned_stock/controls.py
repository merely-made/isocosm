"""Finite compiled owned-stock source faults. Root owns compilation and receipts.

describe only reads. apply/restore require an exact current-file SHA, replace
one declared byte span, and print the complete inverse and resulting hashes.
This helper runs no subprocess and creates no directory or auxiliary file.
Each row names the test target and test that must fail while it is applied,
and the extra cargo flags that run needs (the gate faults need the featureless build).
"""

import argparse
import hashlib
import json
from pathlib import Path

CRATE = Path(__file__).resolve().parents[2]
OWNERSHIP = "src/projects/ownership.rs"
CONSENT = "src/projects/consent.rs"
CAPTURE = "src/projects/debit/capture.rs"
DISPOSITION = "src/projects/debit/disposition.rs"
STAGED = "src/stage/staged/projects.rs"
FEATURELESS = "--no-default-features"
# Faults that only a build without the society feature can catch (rulings 886, 893).
FLAGS = {"gate-open": FEATURELESS, "command-ungated": FEATURELESS, "query-ungated": FEATURELESS}
FAULTS = {
    "claims-exceed-source": (
        OWNERSHIP,
        "if claimed > self.source_amount(lot.source, &lot.account)? {",
        "if claimed > u64::MAX {",
        "owned_stock", "ownership::ownership_names_exact_lots_that_never_exceed_actual_matter",
    ),
    "partial-lot-reserves": (
        OWNERSHIP,
        "if &sum != stocks { return Err(",
        "if sum.is_empty() { return Err(",
        "owned_stock", "ownership::reservation_requires_explicit_owned_lots_matching_the_recipe_exactly",
    ),
    "one-owner-grants": (
        CONSENT,
        "!project.owners.is_empty() && decisions.len() == project.owners.len()\n"
        "            && project.owners.iter().all(",
        "!decisions.is_empty()\n"
        "            && project.owners.iter().any(",
        "owned_stock", "consent::owners_of_the_reserved_inputs_grant_access_together",
    ),
    "maker-self-authorized": (
        CONSENT,
        "    pub fn project_authorized(&self, project: &Project, worker: Id) -> bool {\n"
        "        project.permissions",
        "    pub fn project_authorized(&self, project: &Project, worker: Id) -> bool {\n"
        "        worker == project.maker || project.permissions",
        "owned_stock", "work::reserve_work_and_output_count_every_milligram_once",
    ),
    "revoked-stays-granted": (
        CONSENT,
        "p.granted && self.jointly_decided(project, &p.proof)",
        "self.jointly_decided(project, &p.proof)",
        "owned_stock", "consent::owners_of_the_reserved_inputs_grant_access_together",
    ),
    "auto-allocate-debit": (
        CAPTURE,
        'None => return Err("owned native debit needs an explicit exact lot selection".into()),',
        "None => DebitSelection { request, lots: Vec::new(), unowned: amount },",
        "owned_stock", "debit::an_owned_debit_selects_an_exact_portion_of_a_named_lot",
    ),
    "portion-closes-lot": (
        CAPTURE,
        "if held.lot.amount == 0 { held.at = LotAt::Closed; }",
        "held.at = LotAt::Closed;",
        "owned_stock", "debit::an_owned_debit_selects_an_exact_portion_of_a_named_lot",
    ),
    "default-close": (
        DISPOSITION,
        'let [rule] = rules.as_slice() else { return Err("owned debit needs one exact authored operation disposition".into()); };\n'
        "                let disposition = self.referenced_disposition(&rule.disposition,&selected)?;",
        "let disposition = match rules.as_slice() {\n"
        "                    [rule] => self.referenced_disposition(&rule.disposition,&selected)?,\n"
        "                    _ => Disposition::CloseConsumed };",
        "owned_stock", "debit::consumption_without_an_authored_disposition_refuses",
    ),
    "unspecified-portion": (
        DISPOSITION,
        "if selected.iter().any(|(key, (_, amount))| allocated.get(key) != Some(amount)) {",
        "if false {",
        "owned_stock", "debit::consumption_without_an_authored_disposition_refuses",
    ),
    "output-closes-claim": (
        STAGED,
        "held.at = LotAt::Output(next);",
        "held.at = LotAt::Closed;",
        "owned_stock", "work::reserve_work_and_output_count_every_milligram_once",
    ),
    "step-loses-construction": (
        STAGED,
        "credit(&mut project.construction, account, *amount)?;",
        "credit(&mut Ledger::new(), account, *amount)?;",
        "owned_stock", "work::reserve_work_and_output_count_every_milligram_once",
    ),
    "carrier-ignored": (
        "src/projects/mod.rs",
        "project.carrier.map_or(Some(project.site), |carrier|\n"
        "            self.body_at_start(carrier).and_then(Entity::resident_site))",
        "project.carrier.map_or(Some(project.site), |_| Some(project.site))",
        "owned_stock", "placement::carried_work_follows_its_carrier_and_rests_where_dropped",
    ),
    "drop-keeps-site": (
        "src/projects/admission.rs",
        "            project.site = site;\n            project.carrier = None;",
        "            project.carrier = None;",
        "owned_stock", "placement::carried_work_follows_its_carrier_and_rests_where_dropped",
    ),
    "unwitnessed-carrier": (
        "src/projects/mod.rs",
        '    #[serde(default, skip_serializing_if = "Option::is_none")]\n    pub carrier: Option<Id>,',
        "    #[serde(skip)]\n    pub carrier: Option<Id>,",
        "owned_stock", "placement::carried_work_follows_its_carrier_and_rests_where_dropped",
    ),
    "gate-open": (
        "src/lib.rs",
        "    if SOCIETY { Ok(()) } else { Err(",
        "    if true { Ok(()) } else { Err(",
        "society_gate", "society_operations_are_refused_in_a_build_without_the_feature",
    ),
    "command-ungated": (
        "src/history.rs",
        "Command::Party(change) => crate::society().and_then(|()| sim.party_change(change)),",
        "Command::Party(change) => sim.party_change(change),",
        "society_gate", "society_operations_are_refused_in_a_build_without_the_feature",
    ),
    "query-ungated": (
        "src/queries.rs",
        "Query::Project { .. } | Query::ProjectDecision | Query::InstitutionDecision | Query::Institution { .. }) {\n            crate::society()?;\n        }",
        "Query::Project { .. } | Query::ProjectDecision | Query::InstitutionDecision | Query::Institution { .. }) {\n        }",
        "society_gate", "society_effects_and_queries_are_refused_without_the_feature",
    ),
    "state-gated": (
        "src/projects/mod.rs",
        '    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]\n    pub ownership: BTreeMap<Key, OwnedLot>,',
        "    #[serde(skip)]\n    pub ownership: BTreeMap<Key, OwnedLot>,",
        "society_gate", "society_state_is_kept_and_reads_in_every_build",
    ),
    "item-stays-behind": (
        "src/custody/follow.rs",
        "let changed = held.journey != source.journey || held.place != source.place || held.patch.is_some();",
        "let changed = false;",
        "society_custody", "carry::a_carried_item_rides_the_carriers_journey_and_rests_where_dropped",
    ),
    "burden-ignores-projects": (
        "src/custody/burden.rs",
        "if project.carrier.is_some_and(|carrier| carrier == actor || follows(carrier)) {",
        "if false && project.carrier.is_some_and(|carrier| carrier == actor || follows(carrier)) {",
        "society_custody", "refusal::burden_counts_every_carried_item_and_project_against_capacity",
    ),
    "capacity-unchecked": (
        "src/custody/admission.rs",
        "if change.action == Action::Carry && burden > u128::from(capacity) {",
        "if change.action == Action::Carry && burden > u128::MAX {",
        "society_custody", "refusal::burden_counts_every_carried_item_and_project_against_capacity",
    ),
    "double-carry": (
        "src/custody/admission.rs",
        'Action::Carry if carrier.is_some() => return Err("item already has native custody".into()),',
        'Action::Carry if false => return Err("item already has native custody".into()),',
        "society_custody", "refusal::custody_refuses_without_control_contact_provider_or_a_free_item",
    ),
    "drop-by-anyone": (
        "src/custody/admission.rs",
        "Action::Drop | Action::Equip | Action::Unequip if carrier != Some(change.actor) =>",
        "Action::Drop | Action::Equip | Action::Unequip if false =>",
        "society_custody", "refusal::custody_refuses_without_control_contact_provider_or_a_free_item",
    ),
    "no-contact": (
        "src/custody/admission.rs",
        "|| change.actor == change.item || !actor.colocated(item) {",
        "|| change.actor == change.item {",
        "society_custody", "refusal::custody_refuses_without_control_contact_provider_or_a_free_item",
    ),
    "stranger-opts-in": (
        "src/party.rs",
        'if self.plays(participant) != Some(member) { return Err("only the player\'s currently played body opts in".into()); }',
        'if false { return Err("only the player\'s currently played body opts in".into()); }',
        "society_party", "membership::only_a_players_currently_played_body_opts_in",
    ),
    "withdrawal-needs-current-body": (
        "src/party.rs",
        "// 857: the participant owns its earlier opt-in even after Take.\n",
        "if self.plays(participant) != Some(member) { return Err(\"withdrawal needs the played body\".into()); }\n",
        "society_party", "membership::a_player_withdraws_an_earlier_membership_after_changing_bodies",
    ),
    "disclose-unknown-facts": (
        "src/party.rs",
        "if !sites.is_subset(&known.sites) || !events.is_subset(&known.events)",
        "if false",
        "society_party", "disclosure::a_disclosure_is_bounded_by_the_bodys_native_knowledge",
    ),
    "stranger-discloses": (
        "src/party.rs",
        "if self.plays(participant) != Some(member) || !self.state.relations.contains(&held) {",
        "if false {",
        "society_party", "disclosure::a_disclosure_is_bounded_by_the_bodys_native_knowledge",
    ),
    "leaving-erases-records": (
        "src/party.rs",
        "if declared != party { continue; }",
        "if declared != party || !map.members.iter().any(|m| m.member == *member) { continue; }",
        "society_party", "disclosure::disclosed_places_and_routes_persist_after_the_member_leaves",
    ),
    "delivery-closes-title": (
        "src/spread/owned.rs",
        "held.at = LotAt::Held(StockSource::Site(transfer.to));",
        "held.at = LotAt::Closed;",
        "checkpoint11_spread",
        "normative_direction_requires_exact_owned_quantities_and_retains_claims_through_delivery",
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
        relative, original, changed, target, test = FAULTS[name]
        path = (CRATE / relative).resolve(strict=True)
        if not path.is_relative_to(CRATE):
            raise ValueError("control source is outside its native crate")
        before = path.read_bytes()
        row = {"fault": name, "path": str(path), "target": target, "test": test,
               "flags": FLAGS.get(name, ""),
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
