// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Writing and reading a legacy move as a native [`Flow`].

use super::{Account, ENCLOSURE, Process, Subject, untyped};
use crate::flows::{Composition, Conversion, Flow, Holder};
use crate::legacy::mesocosm::body::SpeciesId;
use crate::legacy::mesocosm::organism::{Kingdom, OrganismId};
use crate::matter::{Material, Stock};

/// A side: the account, on the subject's body where it is a body's.
fn side(account: Account, subject: Option<Subject>) -> (Holder, crate::schema::Key) {
    let holder = match account {
        Account::Soil => Holder::Site(ENCLOSURE),
        Account::Dev => Holder::Dev,
        Account::Substance | Account::Reserve => {
            Holder::Entity(subject.expect("a body's account names its body").id())
        },
    };
    (holder, account.key().into())
}

/// The bare move every constructor starts from; the ledger stamps its tick.
fn moved(
    process: Process,
    from: (Account, Option<Subject>),
    to: (Account, Option<Subject>),
    amount: u64,
) -> Flow {
    Flow {
        tick: 0,
        made_by: process.made_by(),
        from: side(from.0, from.1),
        to: side(to.0, to.1),
        amount,
        count: 1,
        from_kind: from.1.map(Subject::kind),
        to_kind: to.1.map(Subject::kind),
        composition: None,
    }
}

impl Flow {
    /// A producer drawing out of the ground into one of its accounts.
    pub fn uptake(to: Subject, into: Account, amount_mg: u64) -> Self {
        let flow = moved(
            Process::Uptake,
            (Account::Soil, None),
            (into, Some(to)),
            amount_mg,
        );
        Flow {
            composition: untyped(amount_mg),
            ..flow
        }
    }

    /// A body's matter going back into the ground.
    pub fn returned(process: Process, from: Subject, out_of: Account, amount_mg: u64) -> Self {
        let flow = moved(
            process,
            (out_of, Some(from)),
            (Account::Soil, None),
            amount_mg,
        );
        Flow {
            composition: (out_of == Account::Reserve).then(|| Composition::untyped(amount_mg)),
            ..flow
        }
    }

    /// The dev source putting matter into the ground.
    pub fn placed(amount_mg: u64) -> Self {
        let flow = moved(
            Process::Place,
            (Account::Dev, None),
            (Account::Soil, None),
            amount_mg,
        );
        Flow {
            composition: untyped(amount_mg),
            ..flow
        }
    }

    pub fn between(
        process: Process,
        from: Subject,
        out_of: Account,
        to: Subject,
        into: Account,
        amount_mg: u64,
    ) -> Self {
        let flow = moved(process, (out_of, Some(from)), (into, Some(to)), amount_mg);
        Flow {
            composition: (out_of == Account::Reserve).then(|| Composition::untyped(amount_mg)),
            ..flow
        }
    }

    /// Typed matter mineralizing within the ground.
    pub fn soil_mineralization(stock: Stock) -> Self {
        let amount_mg =
            u64::try_from(stock.total()).expect("a scalar-bounded soil conversion fits one flow");
        moved(
            Process::Decay,
            (Account::Soil, None),
            (Account::Soil, None),
            amount_mg,
        )
        .mineralized(stock)
    }

    /// The account the move left, in the legacy world's terms.
    pub fn source(&self) -> Option<Account> {
        Account::of(&self.from)
    }

    /// The account it reached.
    pub fn destination(&self) -> Option<Account> {
        Account::of(&self.to)
    }

    pub fn process(&self) -> Option<Process> {
        Process::of(&self.made_by)
    }

    /// The organism whose account it left.
    pub fn from_organism(&self) -> Option<OrganismId> {
        organism(self.from.0)
    }

    /// The organism whose account it reached.
    pub fn to_organism(&self) -> Option<OrganismId> {
        organism(self.to.0)
    }

    pub fn from_lineage(&self) -> Option<SpeciesId> {
        super::lineage_of(&self.from_kind.as_ref()?.lineage)
    }

    pub fn to_lineage(&self) -> Option<SpeciesId> {
        super::lineage_of(&self.to_kind.as_ref()?.lineage)
    }

    pub fn from_kingdom(&self) -> Option<Kingdom> {
        super::kingdom_of(&self.from_kind.as_ref()?.kingdom)
    }

    pub fn to_kingdom(&self) -> Option<Kingdom> {
        super::kingdom_of(&self.to_kind.as_ref()?.kingdom)
    }

    /// What the move did to an account, signed.
    pub fn net_on(&self, account: Account) -> i128 {
        let into = i128::from(self.destination() == Some(account));
        let out = i128::from(self.source() == Some(account));
        (into - out) * i128::from(self.amount)
    }

    /// Ground drawn up and made producer tissue.
    pub fn synthesized(mut self, output: Stock) -> Self {
        assert_eq!(self.source(), Some(Account::Soil));
        assert_eq!(self.destination(), Some(Account::Substance));
        assert_eq!(output.total(), u128::from(self.amount));
        assert_eq!(output.amount(Material::Untyped), 0);
        assert_eq!(output.amount(Material::Consumer), 0);
        assert_eq!(output.amount(Material::Decomposer), 0);
        self.composition = Some(Composition {
            input: Stock::single(Material::Untyped, self.amount),
            output,
            conversion: Some(Conversion::Synthesis),
        });
        self
    }

    pub fn with_stock(mut self, stock: Stock) -> Self {
        assert_eq!(stock.total(), u128::from(self.amount));
        self.composition = Some(Composition::carried(stock));
        self
    }

    /// Typed tissue returned to the ground as untyped matter.
    pub fn mineralized(self, stock: Stock) -> Self {
        assert_eq!(stock.total(), u128::from(self.amount));
        if stock.amounts()[1..].iter().all(|amount| *amount == 0) {
            return self.with_stock(stock);
        }
        assert_eq!(self.destination(), Some(Account::Soil));
        assert!(matches!(
            self.source(),
            Some(Account::Substance | Account::Soil)
        ));
        self.converted(stock, Conversion::Mineralization)
    }

    /// Typed tissue banked as untyped reserve.
    pub fn digested(self, stock: Stock) -> Self {
        assert_eq!(stock.total(), u128::from(self.amount));
        if stock.amounts()[1..].iter().all(|amount| *amount == 0) {
            return self.with_stock(stock);
        }
        assert_eq!(self.source(), Some(Account::Substance));
        assert_eq!(self.destination(), Some(Account::Reserve));
        self.converted(stock, Conversion::Digestion)
    }

    fn converted(mut self, stock: Stock, conversion: Conversion) -> Self {
        self.composition = Some(Composition {
            input: stock,
            output: Stock::single(Material::Untyped, self.amount),
            conversion: Some(conversion),
        });
        self
    }
}

fn organism(holder: Holder) -> Option<OrganismId> {
    holder
        .body()
        .map(|id| OrganismId(u32::try_from(id).expect("a legacy organism id fits u32")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn subject() -> Subject {
        Subject {
            organism: OrganismId(1),
            lineage: SpeciesId(3),
            kingdom: Kingdom::Producer,
        }
    }

    #[test]
    fn a_transfer_nets_out_of_one_account_and_into_the_other() {
        let flow = Flow::uptake(subject(), Account::Substance, 40);
        assert_eq!(flow.net_on(Account::Soil), -40);
        assert_eq!(flow.net_on(Account::Substance), 40);
        assert_eq!(flow.net_on(Account::Reserve), 0);
        assert_eq!(flow.to_organism(), Some(OrganismId(1)));
        assert_eq!(flow.to_kingdom(), Some(Kingdom::Producer));
        assert_eq!(flow.process(), Some(Process::Uptake));
    }

    #[test]
    fn the_dev_source_is_outside_the_enclosure() {
        let placed = Flow::placed(400);
        assert_eq!(placed.net_on(Account::Soil), 400);
        assert_eq!(placed.net_on(Account::Dev), -400);
        assert_eq!(
            placed.made_by,
            crate::flows::MadeBy::Command("PlaceMatter".into())
        );
        assert!(placed.from_kind.is_none() && placed.to_kind.is_none());
        let stream = [
            placed,
            Flow::placed(75),
            Flow::uptake(subject(), Account::Reserve, 900),
        ];
        assert_eq!(Account::issued_mg(&stream), 475);
    }

    #[test]
    fn digestion_within_a_body_is_internal() {
        let me = subject();
        let flow = Flow::between(
            Process::Uptake,
            me,
            Account::Substance,
            me,
            Account::Reserve,
            9,
        );
        assert!(flow.internal());
        assert!(!Flow::uptake(me, Account::Reserve, 9).internal());
    }
}
