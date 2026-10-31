// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::rules::{Binding, Rules};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Selector {
    Process(Key,u64,String,Key),
    Spread(SpreadDebit,crate::spread::Identity,crate::spread::Substrate),
    ProducedProcess(Key,u64,String,OutputReference),
    ProducedSpread(ProducedOrigin,SpreadDebit,crate::spread::Substrate,OutputReference),
    Placement(crate::spread::Identity,crate::spread::Substrate,StockSource,Key),
}

pub(super) fn stock_rules(stock:&StockRules, rules:&Rules) -> Result<()> {
    let mut dispositions = BTreeSet::new();
    for rule in &stock.dispositions {
        if !dispositions.insert(rule.operation.clone()) { return Err("stock operation disposition is ambiguous".into()); }
        match &rule.operation {
            OperationRule::Process { process, .. } => {
                crate::validation::key(process)?;
                if !rules.processes.contains_key(process) { return Err("stock disposition names no process".into()); }
            },
            OperationRule::Spread { identity, .. } => crate::validation::key(&identity.key())?,
            OperationRule::ProducedSpread { origin,input, .. } => { origin.validate(rules)?; input.validate(rules)?; },
            _ => {},
        }
        match &rule.disposition {
            Disposition::CloseConsumed => {},
            Disposition::Retain { outputs, closed } => {
                if outputs.is_empty() { return Err("retained stock disposition has no explicit outputs".into()); }
                for output in outputs {
                    crate::validation::key(&output.input)?;
                    matter_key(rules,&output.account)?;
                    if output.amount == 0 { return Err("retained claim needs exact positive quantity".into()); }
                }
                quantities(closed,0,false)?;
            },
            Disposition::Outputs { outputs, closed } => {
                let operation = match &rule.operation {
                    OperationRule::Spread { operation, .. } | OperationRule::ProducedSpread { operation, .. } => *operation,
                    OperationRule::Command(DirectOperation::SpreadPlace) => SpreadDebit::Place,
                    _ => return Err("native spread output claims need their exact spread operation".into()),
                };
                if outputs.is_empty() { return Err("native spread disposition has no explicit output claims".into()); }
                for output in outputs {
                    crate::validation::key(&output.input)?;
                    if output.amount == 0 || !matches!((operation,output.output),
                        (SpreadDebit::Overflow,SpreadOutput::Route(_)) | (SpreadDebit::ForkStrain,SpreadOutput::GeneratedStrain)
                            | (SpreadDebit::Place,SpreadOutput::Placement)) {
                        return Err("spread output claim names a different operation or zero quantity".into());
                    }
                }
                if operation == SpreadDebit::Overflow && !closed.is_empty() {
                    return Err("unchanged transported claims cannot close instead of travelling".into());
                }
                quantities(closed,0,false)?;
            },
            Disposition::RetainReferenced { outputs,closed } => {
                if outputs.is_empty() { return Err("referenced retained disposition has no outputs".into()); }
                for output in outputs {
                    references::validate_input(&output.input,rules)?;
                    matter_key(rules,&output.account)?;
                    if output.amount == 0 { return Err("referenced retained output needs exact positive quantity".into()); }
                }
                references::validate_quantities(closed,rules)?;
            },
            Disposition::OutputsReferenced { outputs,closed } => {
                let operation = match &rule.operation {
                    OperationRule::Spread { operation, .. } | OperationRule::ProducedSpread { operation, .. } => *operation,
                    OperationRule::Command(DirectOperation::SpreadPlace) => SpreadDebit::Place,
                    _ => return Err("referenced spread outputs require an exact native spread operation".into()),
                };
                if outputs.is_empty() { return Err("referenced spread disposition has no output claims".into()); }
                for output in outputs {
                    references::validate_input(&output.input,rules)?;
                    if output.amount == 0 || !matches!((operation,output.output),
                        (SpreadDebit::Overflow,SpreadOutput::Route(_)) | (SpreadDebit::ForkStrain,SpreadOutput::GeneratedStrain)
                            | (SpreadDebit::Place,SpreadOutput::Placement))
                    { return Err("referenced spread output names another operation or zero quantity".into()); }
                }
                if operation == SpreadDebit::Overflow && !closed.is_empty() {
                    return Err("unchanged referenced transport cannot close consumed claims".into());
                }
                references::validate_quantities(closed,rules)?;
            },
        }
    }
    let mut selections = BTreeSet::new();
    for selector in &stock.selections {
        let (key,lots,unowned) = match selector {
            SourceRule::Process { process,operation,from,account,lots,unowned } => {
                crate::validation::key(process)?;
                matter_key(rules,account)?;
                if !rules.processes.contains_key(process) || *from == Binding::Part {
                    return Err("stock selection lacks its native process or body/site binding".into());
                }
                (Selector::Process(process.clone(),*operation,format!("{from:?}"),account.clone()),lots,*unowned)
            },
            SourceRule::Spread { operation,identity,at,lots,unowned } => {
                crate::validation::key(&identity.key())?;
                (Selector::Spread(*operation,identity.clone(),*at),lots,*unowned)
            },
            SourceRule::Placement { identity,at,source,account,lots,unowned } => {
                crate::validation::key(&identity.key())?;
                crate::validation::key(account)?;
                if matches!(source,StockSource::Transit(_)) || (account != &identity.key() && !crate::meaning::matter(rules,account)) {
                    return Err("placement selector requires an actual local rule account or exact native identity".into());
                }
                (Selector::Placement(identity.clone(),*at,*source,account.clone()),lots,*unowned)
            },
            SourceRule::ProducedProcess { process,operation,from,output } => {
                crate::validation::key(process)?;
                output.reference.validate(rules)?;
                if !rules.processes.contains_key(process) || *from == Binding::Part || output.amount == 0 {
                    return Err("produced selector lacks its process, actual body/site binding or positive quantity".into());
                }
                let key = Selector::ProducedProcess(process.clone(),*operation,format!("{from:?}"),output.reference.clone());
                if !selections.insert(key) { return Err("produced source selector is ambiguous".into()); }
                continue;
            },
            SourceRule::ProducedSpread { origin,operation,at,output } => {
                origin.validate(rules)?;
                output.reference.validate(rules)?;
                if output.amount == 0 { return Err("produced spread selector needs an exact positive quantity".into()); }
                let key = Selector::ProducedSpread(origin.clone(),*operation,*at,output.reference.clone());
                if !selections.insert(key) { return Err("produced spread selector is ambiguous".into()); }
                continue;
            },
        };
        if !selections.insert(key) { return Err("stock source selector is ambiguous".into()); }
        quantities(lots,unowned,true)?;
    }
    Ok(())
}

fn quantities(lots:&[LotQuantity], unowned:u64, positive:bool) -> Result<()> {
    let mut seen = BTreeSet::new();
    let mut amount = unowned;
    for lot in lots {
        crate::validation::key(&lot.key)?;
        if lot.amount == 0 || !seen.insert(&lot.key) { return Err("stock selector repeats a lot or names zero quantity".into()); }
        amount = amount.checked_add(lot.amount).ok_or("selected stock quantity overflow")?;
    }
    if positive && amount == 0 { return Err("stock selection needs an exact positive debit".into()); }
    Ok(())
}

fn matter_key(rules:&Rules, key:&str) -> Result<()> {
    crate::validation::key(key)?;
    if !crate::meaning::matter(rules,key) { return Err("ordinary stock declaration names no rule matter account".into()); }
    Ok(())
}
