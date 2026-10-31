// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

pub(crate) fn declared(rules: Option<&StockRules>) -> bool {
    rules.is_some_and(|rules| rules.selections.iter().any(|rule|
        matches!(rule, SourceRule::ProducedProcess { .. } | SourceRule::ProducedSpread { .. } | SourceRule::Placement { .. }))
        || rules.dispositions.iter().any(|rule| matches!(rule.operation, OperationRule::ProducedSpread { .. })
            || matches!(rule.disposition, Disposition::RetainReferenced { .. } | Disposition::OutputsReferenced { .. })))
}

impl Context {
    pub(super) fn authored_selection(&self, request: &DebitRequest) -> Result<Option<DebitSelection>> {
        let mut matching = Vec::new();
        for rule in self.rules.iter().flat_map(|rules| &rules.selections) {
            let selected = match (rule,&self.origin) {
                (SourceRule::Process { process:named,operation,from,account,lots,unowned }, DebitOrigin::Process { process, .. })
                    if named == process && *operation == request.operation && account == &request.account
                        && self.bindings.iter().any(|(binding,at)| binding == from && *at == request.source) =>
                    Some((lots.clone(),*unowned)),
                (SourceRule::Spread { operation:named,identity:selected,at:selected_at,lots,unowned },
                    DebitOrigin::NativeSpread { operation,identity,at, .. })
                    if named == operation && selected == identity && selected_at == at
                        && request.source == substrate_source(*at) && request.account == identity.key() =>
                    Some((lots.clone(),*unowned)),
                (SourceRule::ProducedProcess { process:named,operation,from,output }, DebitOrigin::Process { process, .. })
                    if named == process && *operation == request.operation
                        && self.bindings.iter().any(|(binding,at)| binding == from && *at == request.source) =>
                    self.reference_selection(output,request,None)?,
                (SourceRule::ProducedSpread { origin,operation:named,at:selected_at,output },
                    DebitOrigin::NativeSpread { operation,identity,at, .. })
                    if named == operation && selected_at == at && *origin == ProducedOrigin::of(&self.origin)? =>
                    self.reference_selection(output,request,Some(identity))?,
                (SourceRule::Placement { identity:named,at,source,account,lots,unowned },
                    DebitOrigin::NativeSpread { parent,operation:SpreadDebit::Place,identity, .. })
                    if named == identity && *source == request.source && account == &request.account
                        && matches!(parent.as_ref(),DebitOrigin::Command(command)
                            if matches!(command.as_ref(),crate::history::Command::Spread(crate::spread::Command::Place(placement))
                                if placement.at == *at)) => Some((lots.clone(),*unowned)),
                _ => None,
            };
            if let Some(selected) = selected { matching.push(selected); }
        }
        if matching.len() > 1 { return Err("authored stock selector is ambiguous at the actual native binding".into()); }
        Ok(matching.pop().map(|(lots,unowned)| DebitSelection { request:request.clone(),lots,unowned }))
    }

    fn reference_selection(&self, selected: &ProducedQuantity, request: &DebitRequest,
        identity: Option<&crate::spread::Identity>) -> Result<Option<(Vec<LotQuantity>,u64)>> {
        let (key,record) = selected.reference.resolve(&self.produced)?;
        let ordinary_placement = record.identity.is_none() && matches!(&self.origin,
            DebitOrigin::NativeSpread { operation:SpreadDebit::Place,parent, .. }
                if matches!(parent.as_ref(),DebitOrigin::Command(command)
                    if matches!(command.as_ref(),crate::history::Command::Spread(crate::spread::Command::Place(placement))
                        if matches!(&placement.source,crate::spread::Supply::Matter { from,account }
                            if substrate_source(*from) == request.source && account == &record.account))));
        if record.account != request.account || identity.is_some_and(|identity|
            record.identity.as_ref() != Some(identity) && !ordinary_placement) {
            return Ok(None);
        }
        let held = self.shadow.get(key).ok_or("named produced output has no native claim")?;
        if selected.amount == 0 || selected.amount != request.amount || held.current_source() != Some(request.source)
            || held.lot.key != *key || held.lot.source != record.source || held.lot.account != record.account || held.lot.owners.is_empty()
            || selected.amount > held.lot.amount || held.lot.amount > record.amount
        { return Err("produced selection differs from its actual holder, debit or remaining quantity".into()); }
        if let Some(identity) = &record.identity {
            if identity.key() != record.account || self.identities.get(&record.account) != Some(identity) {
                return Err("produced selection identity is not registered".into());
            }
        } else if !self.matter.contains(&record.account) {
            return Err("produced selection account is not declared native matter".into());
        }
        Ok(Some((vec![LotQuantity { key:key.clone(),amount:selected.amount }],0)))
    }

    pub(super) fn disposition_matches(&self, authored: &OperationRule, actual: &OperationRule,
        selected: &BTreeMap<Key,(StockLot,u64)>) -> Result<bool> {
        if matches!(authored,OperationRule::Command(DirectOperation::SpreadPlace))
            && matches!(actual,OperationRule::Spread { operation:SpreadDebit::Place, .. })
        {
            return Ok(matches!(&self.origin,DebitOrigin::NativeSpread { parent, .. }
                if matches!(parent.as_ref(),DebitOrigin::Command(command)
                    if matches!(command.as_ref(),crate::history::Command::Spread(crate::spread::Command::Place(_))))));
        }
        if let (OperationRule::ProducedSpread { origin,operation:named,at:source,input },
            OperationRule::Spread { operation,identity,at }) = (authored,actual)
        {
            if named != operation || source != at || *origin != ProducedOrigin::of(&self.origin)? { return Ok(false); }
            let (key,record) = input.resolve(&self.produced)?;
            let ordinary_placement = *operation == SpreadDebit::Place && record.identity.is_none()
                && selected.get(key).is_some_and(|(lot,_)| lot.account == record.account);
            return Ok((record.identity.as_ref() == Some(identity) || ordinary_placement) && selected.contains_key(key));
        }
        Ok(authored == actual)
    }

    fn input_key(&self, input: &ClaimInput, selected: &BTreeMap<Key,(StockLot,u64)>) -> Result<Key> {
        let key = match input {
            ClaimInput::Lot(key) => key.clone(),
            ClaimInput::Produced(reference) => reference.resolve(&self.produced)?.0.clone(),
        };
        if !selected.contains_key(&key) { return Err("referenced disposition input was not actually selected".into()); }
        Ok(key)
    }

    pub(super) fn referenced_disposition(&self, disposition: &Disposition,
        selected: &BTreeMap<Key,(StockLot,u64)>) -> Result<Disposition> {
        let quantities = |closed: &[ClaimQuantity]| -> Result<Vec<LotQuantity>> {
            closed.iter().map(|portion| Ok(LotQuantity {
                key:self.input_key(&portion.input,selected)?,amount:portion.amount,
            })).collect()
        };
        match disposition {
            Disposition::RetainReferenced { outputs,closed } => Ok(Disposition::Retain {
                outputs:outputs.iter().map(|output| Ok(OutputClaim {
                    input:self.input_key(&output.input,selected)?,amount:output.amount,
                    destination:output.destination,account:output.account.clone(),
                })).collect::<Result<_>>()?,closed:quantities(closed)?,
            }),
            Disposition::OutputsReferenced { outputs,closed } => Ok(Disposition::Outputs {
                outputs:outputs.iter().map(|output| Ok(RetainedOutput {
                    input:self.input_key(&output.input,selected)?,amount:output.amount,output:output.output,
                })).collect::<Result<_>>()?,closed:quantities(closed)?,
            }),
            other => Ok(other.clone()),
        }
    }
}

pub(super) fn validate_input(input: &ClaimInput, rules: &crate::rules::Rules) -> Result<()> {
    match input { ClaimInput::Lot(key) => crate::validation::key(key), ClaimInput::Produced(reference) => reference.validate(rules) }
}

pub(super) fn validate_quantities(quantities: &[ClaimQuantity], rules: &crate::rules::Rules) -> Result<()> {
    let mut seen = BTreeSet::new();
    for quantity in quantities {
        validate_input(&quantity.input,rules)?;
        if quantity.amount == 0 || !seen.insert(serde_json::to_string(&quantity.input).map_err(|e| e.to_string())?) {
            return Err("referenced disposition repeats an input or names zero quantity".into());
        }
    }
    Ok(())
}
