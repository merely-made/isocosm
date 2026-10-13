// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::rules::{Binding, Rules};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Selector {
    Process(Key,u64,String,Key),
    Spread(SpreadDebit,crate::spread::Identity,crate::spread::Substrate),
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
                    OperationRule::Spread { operation, .. } => *operation,
                    _ => return Err("native spread output claims need their exact spread operation".into()),
                };
                if outputs.is_empty() { return Err("native spread disposition has no explicit output claims".into()); }
                for output in outputs {
                    crate::validation::key(&output.input)?;
                    if output.amount == 0 || !matches!((operation,output.output),
                        (SpreadDebit::Overflow,SpreadOutput::Route(_)) | (SpreadDebit::ForkStrain,SpreadOutput::GeneratedStrain)) {
                        return Err("spread output claim names a different operation or zero quantity".into());
                    }
                }
                if operation == SpreadDebit::Overflow && !closed.is_empty() {
                    return Err("unchanged transported claims cannot close instead of travelling".into());
                }
                quantities(closed,0,false)?;
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
