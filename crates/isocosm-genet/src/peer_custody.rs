//! Identity operations the table declares to Djinn on its application connection.

use crate::family_table::invite::{RECORD_SALT, TABLE_SALT, TERMS};
use castellan_client::custody_identity::CustodyIdentity;
use ostiary::declaration::{
    AppDeclaration, CustodyOperation as Op, CustodyUse, NamePattern, SaltPattern, SaltUse,
};

fn declaration(hosting: bool, records: bool) -> AppDeclaration {
    let mut custody = CustodyUse {
        operations: [Op::Status, Op::WatchLock, Op::Release, Op::Attest].into(),
        salts: vec![SaltUse::persona(SaltPattern::Exact(TABLE_SALT.into()))],
        ..Default::default()
    };
    if hosting {
        custody.operations.insert(Op::Sign);
        custody
            .acts
            .push(SaltUse::persona(SaltPattern::DelegationScope(
                TERMS.domain.into(),
            )));
        if records {
            custody
                .salts
                .push(SaltUse::persona(SaltPattern::Exact(RECORD_SALT.into())));
        }
    }
    AppDeclaration {
        routes: vec![NamePattern::exact(castellan_client::custody::CUSTODY_ROUTE)],
        custody,
        ..Default::default()
    }
}

pub(super) fn identity(hosting: bool, records: bool) -> Result<CustodyIdentity, String> {
    let persona = super::arg("--persona").map(personae::vault::ProfileId);
    CustodyIdentity::connect_declared_as("isocosm", declaration(hosting, records), persona)
        .map_err(|error| format!("the custody identity is unavailable: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_replication_host_requests_the_record_key() {
        for (host, records) in [(false, false), (true, false), (true, true)] {
            let wants = declaration(host, records).custody;
            assert!(
                wants
                    .salts
                    .iter()
                    .any(|key| *key == SaltUse::persona(SaltPattern::Exact(TABLE_SALT.into())))
            );
            assert_eq!(
                wants
                    .salts
                    .iter()
                    .any(|key| *key == SaltUse::persona(SaltPattern::Exact(RECORD_SALT.into()))),
                records
            );
            assert_eq!(wants.operations.contains(&Op::Sign), host);
            assert_eq!(wants.acts.len(), usize::from(host));
        }
    }
}
