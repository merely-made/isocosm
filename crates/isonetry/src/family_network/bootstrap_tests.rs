use super::*;
use crate::family::FamilyReplica;
use crate::family_network::tests::host;
use chirograph::ResourceResponse;
use cosmograph::Execution;
use p2panda_core::SigningKey;

fn checkpoint_resource(key: &SigningKey) -> FamilyCheckpointResource {
    let mut host = host();
    let checkpoint = host.checkpoint().unwrap();
    FamilyCheckpointResource::new(
        &checkpoint,
        key,
        ProjectionSession("vtt:table".into()),
        WireLimits::default(),
    )
    .unwrap()
}

#[test]
fn signed_checkpoint_resource_verifies_before_fresh_replica_install() {
    let key = SigningKey::from_bytes(&[0x61; 32]);
    let resource = checkpoint_resource(&key);
    let response = resource.response();
    let descriptor = resource.descriptor();
    assert_eq!(
        resource.respond(&descriptor.request()).as_ref(),
        Some(response)
    );
    let checkpoint = verify_checkpoint_resource(
        response,
        descriptor,
        &key.verifying_key(),
        WireLimits::default(),
    )
    .unwrap();
    let replica = FamilyReplica::install(checkpoint, &[], Execution::Individuals).unwrap();
    assert_eq!(replica.sequence(), 0);
}

#[test]
fn signed_checkpoint_resource_rejects_wrong_pin_session_and_resource_bytes() {
    let key = SigningKey::from_bytes(&[0x62; 32]);
    let resource = checkpoint_resource(&key);
    let response = resource.response();
    let descriptor = resource.descriptor();
    let foreign = SigningKey::from_bytes(&[0x63; 32]);
    assert!(
        verify_checkpoint_resource(
            response,
            descriptor,
            &foreign.verifying_key(),
            WireLimits::default(),
        )
        .is_err()
    );
    let mut wrong_projection = descriptor.clone();
    wrong_projection.projection_session = chirograph::ProjectionSession("vtt:other".into());
    assert!(
        verify_checkpoint_resource(
            response,
            &wrong_projection,
            &key.verifying_key(),
            WireLimits::default(),
        )
        .is_err()
    );
    let mut wrong_resource = descriptor.clone();
    wrong_resource.resource = chirograph::ContentHash([0; 32]);
    assert!(
        verify_checkpoint_resource(
            response,
            &wrong_resource,
            &key.verifying_key(),
            WireLimits::default(),
        )
        .is_err()
    );
    assert!(
        verify_checkpoint_resource(
            response,
            &FamilyCheckpointDescriptor {
                family_session: SessionId([0x99; 16]),
                ..descriptor.clone()
            },
            &key.verifying_key(),
            WireLimits::default(),
        )
        .is_err()
    );

    let mut changed = response.clone();
    changed.bytes.push(0);
    assert!(
        verify_checkpoint_resource(
            &changed,
            descriptor,
            &key.verifying_key(),
            WireLimits::default(),
        )
        .is_err()
    );

    let mut forged_bytes = response.bytes.clone();
    let header_start = SIGNED_MAGIC.len() + 4;
    let header_len = u32::from_le_bytes(
        forged_bytes[SIGNED_MAGIC.len()..header_start]
            .try_into()
            .unwrap(),
    ) as usize;
    forged_bytes[header_start + header_len - 1] ^= 1;
    let forged = ResourceResponse::new(response.session.clone(), forged_bytes);
    assert!(forged.has_valid_address());
    let mut forged_descriptor = descriptor.clone();
    forged_descriptor.resource = forged.resource;
    forged_descriptor.byte_size = forged.bytes.len() as u64;
    assert!(
        verify_checkpoint_resource(
            &forged,
            &forged_descriptor,
            &key.verifying_key(),
            WireLimits::default(),
        )
        .is_err()
    );
    assert!(
        verify_checkpoint_resource(
            response,
            descriptor,
            &key.verifying_key(),
            WireLimits { max_bytes: 1 },
        )
        .is_err()
    );
}
