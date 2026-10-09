// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What a process does: its effects, and the conversions a transform may
//! declare.

use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    Transfer {
        from: Binding,
        to: Binding,
        account: Key,
        amount: Amount,
    },
    /// An authored transform accounts for both sides, including byproducts.
    Transform {
        who: Binding,
        take: Ledger,
        give: Ledger,
        /// The kind of conversion it declares, checked against that kind
        /// (rulings 342 and 357); undeclared transforms pass as before, and
        /// serialize as they did.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        conversion: Option<Conversion>,
    },
    Condition {
        key: Key,
        delta: i64,
    },
    Relate {
        kind: Key,
        present: bool,
    },
    Trait {
        who: Binding,
        key: Key,
        present: bool,
    },
    Practice {
        key: Key,
        amount: Amount,
    },
    Move {
        destination: Id,
    },
    Note {
        kind: Key,
        text: String,
        lifetime: Option<Tick>,
    },
    Birth {
        provision: Ledger,
    },
    /// A birth spending the actor's provision (rulings 447, 448, 518 and
    /// 530): a brood develops its lineage's whole recipe from a soma its own
    /// seed draws; a clutch lays the lineage's clutch of eggs, each its
    /// recipe's root alone, sharing the provision. Each child carries
    /// `young` from birth until something takes it away, as a caring
    /// lineage's young are unweaned until first fed (554).
    Bear {
        clutch: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        young: Option<Key>,
    },
    /// A bud (ruling 524): the provision poured into a part marked `mark`,
    /// grown at the reproducing part if there is none, which severs into a
    /// body of its own once full; with `once`, the semelparous parent dies
    /// as it severs (521).
    Bud {
        mark: Key,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        once: bool,
    },
    Death,
    Tell {
        event: Key,
    },
    FoundPolity {
        governance: Key,
        focus: BTreeSet<Key>,
        support: Key,
    },
    /// A measured account on an authored axis, judged against standing history.
    Record {
        axis: Key,
        account: Key,
    },
    /// A body's kept level eases by up to `amount`, never below nothing:
    /// strain bleeding off (ruling 159).
    Ease {
        who: Binding,
        key: Key,
        amount: Amount,
    },
    /// Eating (ruling 287): up to `amount` of a body's matter, drawn from
    /// all its matter accounts in proportion, largest remainders first in
    /// key order, and credited to the actor's own `into` account. A meal
    /// naming accounts takes from those alone (ruling 456); meals naming
    /// none serialize as before.
    Eat {
        from: Binding,
        amount: Amount,
        into: Key,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        of: Vec<Key>,
        /// Whether a bite that would take all of a part's tissue takes the
        /// part whole, as its affinity allows (ruling 516).
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        whole: bool,
    },
    /// Effects applied only where a guard comes to something (X5), such as
    /// an eater paying for a bite only while the gland is charged, and
    /// others where it does not, the guard read once: TD5's routing of a
    /// meal by one reading of hunger.
    When {
        guard: Expr,
        then: Vec<Effect>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        otherwise: Vec<Effect>,
    },
    /// Up to `amount` taken from the actor's accounts in the order listed,
    /// each share moved to the same account at `to` (ruling 446): upkeep
    /// paid from the reserve before the tissue. With `into`, what is paid
    /// arrives as that account instead, living matter returned as the
    /// world's: rent mineralized at once, as Mesocosm returns it.
    Spend {
        from: Vec<Key>,
        to: Binding,
        amount: Amount,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        into: Option<Key>,
    },
    /// The actor's next missing parts grown from what it holds of `from`
    /// (rulings 479 and 510), each paying PD2's price from its reserve and
    /// then its tissue into the place's `into`, and filled by `conversion`.
    Grow {
        from: Key,
        into: Key,
        conversion: Conversion,
    },
    /// Up to `amount` of one ledger's `from` accounts, a share of each in
    /// proportion, given to the same ledger as `to` by a declared
    /// conversion: a meal digested, soil synthesized, matter mineralized.
    Convert {
        who: Binding,
        from: Vec<Key>,
        to: Key,
        amount: Amount,
        conversion: Conversion,
    },
    /// A value computed against the act as it stands, kept under `name`
    /// for the act's later effects to read.
    Keep {
        name: Key,
        value: Expr,
    },
    /// Cells of the bound part moved to a function it expresses, from
    /// another or, with no `from`, from its free cells (X6).
    Allocate {
        from: Option<Key>,
        to: Key,
        cells: Amount,
    },
    /// `ask` carried to `who`'s effect parts through the systems naming
    /// `function` in `role` (rulings 562 and 581), each asking by its room
    /// for `lands`: where a part is no effect or was carried less than it
    /// asked, what the act then gives each part is bounded by what reached
    /// it.
    Carry {
        who: Binding,
        function: Key,
        role: Role,
        ask: Amount,
        lands: Vec<Key>,
    },
}

impl Effect {
    /// The amounts an act resolves before it stages this effect (X3); a
    /// guarded effect's are its branches'.
    pub fn amounts(&self) -> Vec<&Amount> {
        match self {
            Self::Transfer { amount, .. }
            | Self::Practice { amount, .. }
            | Self::Ease { amount, .. }
            | Self::Eat { amount, .. }
            | Self::Spend { amount, .. }
            | Self::Convert { amount, .. } => vec![amount],
            Self::Allocate { cells, .. } => vec![cells],
            Self::Carry { ask, .. } => vec![ask],
            Self::When { .. } => self.branches().flat_map(Effect::amounts).collect(),
            _ => vec![],
        }
    }
    /// A guarded effect's branches, both of them; nothing for another.
    pub fn branches(&self) -> impl Iterator<Item = &Effect> {
        let (then, otherwise): (&[Effect], &[Effect]) = match self {
            Self::When {
                then, otherwise, ..
            } => (then, otherwise),
            _ => (&[], &[]),
        };
        then.iter().chain(otherwise)
    }
    pub fn draws(&self) -> bool {
        matches!(self, Self::When { guard: x, .. } | Self::Keep { value: x, .. } if x.draws())
            || self.amounts().iter().any(|a| a.draws())
    }
    /// The expressions it reads besides its amounts: a guard, or a kept
    /// value.
    pub fn computed(&self) -> Option<&Expr> {
        match self {
            Self::When { guard: x, .. } | Self::Keep { value: x, .. } => Some(x),
            _ => None,
        }
    }
    /// The branch a guard chooses, read once; nothing for another effect.
    pub fn branch(
        &self,
        read: &mut Read,
        draw: &mut Draw,
        parts: &mut PartsOf,
    ) -> crate::Result<Option<&[Effect]>> {
        let Self::When {
            guard,
            then,
            otherwise,
        } = self
        else {
            return Ok(None);
        };
        Ok(Some(match guard.eval_in(read, draw, parts)? {
            0 => otherwise,
            _ => then,
        }))
    }
    /// Whether resolving it computes anything, a guard included.
    pub fn computes(&self) -> bool {
        matches!(self, Self::When { .. } | Self::Keep { .. })
            || self
                .amounts()
                .iter()
                .any(|a| matches!(a, Amount::Computed(_)))
    }
    /// This effect with every amount resolved to the number it comes to; a
    /// guarded effect is applied by its branch instead.
    pub fn resolve(
        &self,
        read: &mut Read,
        draw: &mut Draw,
        parts: &mut PartsOf,
    ) -> crate::Result<Effect> {
        let mut e = self.clone();
        match &mut e {
            Self::Transfer { amount, .. }
            | Self::Practice { amount, .. }
            | Self::Ease { amount, .. }
            | Self::Eat { amount, .. }
            | Self::Spend { amount, .. }
            | Self::Convert { amount, .. } => *amount = amount.resolve(read, draw, parts)?,
            Self::Allocate { cells, .. } => *cells = cells.resolve(read, draw, parts)?,
            Self::Carry { ask, .. } => *ask = ask.resolve(read, draw, parts)?,
            Self::When { .. } => return Err("a guarded effect is applied by its branch".into()),
            _ => {},
        }
        Ok(e)
    }
}

/// The conversions a transform may declare (rulings 342 and 357). World
/// matter is matter of a lineage of the world's kingdom (rulings 98 and 100);
/// all other matter is living.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Conversion {
    /// World matter into the body's own lineage's: whatever synthesizes is a
    /// producer.
    Synthesis,
    /// Living matter into the eater's own lineage's.
    Digestion,
    /// Living matter back into the world's.
    Mineralization,
}
