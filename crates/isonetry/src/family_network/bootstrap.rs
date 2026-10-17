//! Bounded, host-authenticated checkpoint delivery over Graphshell resources.
//!
//! A join invitation carries the bounded resource descriptor and pinned host
//! identity. An admitted Graphshell resource request transfers the bytes; the
//! client verifies the host signature before installing any family state.

use chirograph::{ContentHash, ProjectionSession, ResourceRequest, ResourceResponse};
use p2panda_core::{
    Body, Header, Operation, SigningKey, VerifyingKey, operation::validate_operation,
};
use serde::{Deserialize, Serialize};

use crate::family::{Checkpoint, FAMILY_FORMAT_VERSION, SessionId, WireLimits};

const SIGNED_MAGIC: &[u8] = b"ISOCOSM-FAMILY-SIGNED-CHECKPOINT-V1\0";
const CHECKPOINT_MAGIC: &[u8] = b"ISOCOSM-FAMILY-CHECKPOINT-V1\0";

/// Resource identity carried with the host pin and family session in a join
/// invitation. It discloses no checkpoint bytes; those travel after admission.
/// Its projection-session field is invitation metadata, not a field in the
/// signed p2panda header.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FamilyCheckpointDescriptor {
    pub family_session: SessionId,
    pub projection_session: ProjectionSession,
    pub resource: ContentHash,
    pub byte_size: u64,
}

impl FamilyCheckpointDescriptor {
    pub fn request(&self) -> ResourceRequest {
        ResourceRequest {
            session: self.projection_session.clone(),
            resource: self.resource,
        }
    }
}

/// A signed checkpoint resource bound to one Graphshell projection session.
/// It contains the full World and all opaque sections. Expose it only to
/// invited peers authorized for full World replication; permission to read
/// one projection alone does not grant that access. The descriptor binds the
/// delivery session; the signature binds the family checkpoint and host.
#[derive(Clone, Debug)]
pub struct FamilyCheckpointResource {
    descriptor: FamilyCheckpointDescriptor,
    response: ResourceResponse,
}

impl FamilyCheckpointResource {
    /// Sign and encode a checkpoint under the sole host's key.
    pub fn new(
        checkpoint: &Checkpoint,
        key: &SigningKey,
        projection_session: ProjectionSession,
        limits: WireLimits,
    ) -> Result<Self, String> {
        checkpoint.validate()?;
        let body = checkpoint.encode_with_limits(limits)?;
        let mut signed_body = Vec::with_capacity(CHECKPOINT_MAGIC.len() + body.len());
        signed_body.extend_from_slice(CHECKPOINT_MAGIC);
        signed_body.extend_from_slice(&body);
        let header = Header::builder()
            .body(&signed_body)
            .seq_num(0)
            .backlink(None)
            .build(
                key,
                super::FamilyRecordExt {
                    session: checkpoint.session,
                    format: FAMILY_FORMAT_VERSION,
                },
            );
        let header = header.encode();
        let header_len =
            u32::try_from(header.len()).map_err(|_| "checkpoint header is too large")?;
        let mut bytes =
            Vec::with_capacity(SIGNED_MAGIC.len() + 4 + header.len() + signed_body.len());
        bytes.extend_from_slice(SIGNED_MAGIC);
        bytes.extend_from_slice(&header_len.to_le_bytes());
        bytes.extend_from_slice(&header);
        bytes.extend_from_slice(&signed_body);
        if bytes.len() > limits.max_bytes {
            return Err("signed family checkpoint exceeds configured wire limit".into());
        }
        let response = ResourceResponse::new(projection_session.clone(), bytes);
        let descriptor = FamilyCheckpointDescriptor {
            family_session: checkpoint.session,
            projection_session,
            resource: response.resource,
            byte_size: response.bytes.len() as u64,
        };
        Ok(Self {
            descriptor,
            response,
        })
    }

    pub fn descriptor(&self) -> &FamilyCheckpointDescriptor {
        &self.descriptor
    }

    pub fn response(&self) -> &ResourceResponse {
        &self.response
    }

    /// Answer only the exact request named in the join descriptor. The caller
    /// must enforce the full-replication authorization before serving it.
    pub fn respond(&self, request: &ResourceRequest) -> Option<ResourceResponse> {
        (request.session == self.descriptor.projection_session
            && request.resource == self.descriptor.resource)
            .then(|| self.response.clone())
    }
}

/// Authenticate and validate a checkpoint resource before replica install.
pub fn verify_checkpoint_resource(
    response: &ResourceResponse,
    descriptor: &FamilyCheckpointDescriptor,
    host: &VerifyingKey,
    limits: WireLimits,
) -> Result<Checkpoint, String> {
    if response.session != descriptor.projection_session
        || response.resource != descriptor.resource
        || response.bytes.len() as u64 != descriptor.byte_size
        || !response.has_valid_address()
    {
        return Err("checkpoint resource differs from its join descriptor".into());
    }
    if response.bytes.len() > limits.max_bytes {
        return Err("signed family checkpoint exceeds configured wire limit".into());
    }
    let rest = response
        .bytes
        .strip_prefix(SIGNED_MAGIC)
        .ok_or("resource is not a signed family checkpoint")?;
    if rest.len() < 4 {
        return Err("signed checkpoint header length is missing".into());
    }
    let header_len = u32::from_le_bytes(rest[..4].try_into().expect("four bytes")) as usize;
    let rest = &rest[4..];
    if rest.len() < header_len {
        return Err("signed checkpoint header is truncated".into());
    }
    let (header_bytes, body) = rest.split_at(header_len);
    let header = Header::<super::FamilyRecordExt>::decode(header_bytes)
        .map_err(|error| format!("checkpoint header is invalid: {error}"))?;
    if header.seq_num != 0 || header.backlink.is_some() {
        return Err("checkpoint signature is not at the bootstrap log position".into());
    }
    if &header.verifying_key != host
        || header.extensions.session != descriptor.family_session
        || header.extensions.format != FAMILY_FORMAT_VERSION
    {
        return Err("checkpoint header differs from the pinned family host or session".into());
    }
    let operation = Operation {
        hash: header.hash(),
        header,
        body: Some(Body::from_bytes(body)),
    };
    validate_operation(&operation).map_err(|error| error.to_string())?;
    let Some(body) = operation.body.as_ref().map(Body::to_bytes) else {
        return Err("signed checkpoint has no body".into());
    };
    let checkpoint_bytes = body
        .strip_prefix(CHECKPOINT_MAGIC)
        .ok_or("signed resource is not a family checkpoint")?;
    let checkpoint = Checkpoint::decode_with_limits(checkpoint_bytes, limits)?;
    if checkpoint.session != descriptor.family_session {
        return Err("checkpoint body names another family session".into());
    }
    checkpoint.validate()?;
    Ok(checkpoint)
}

#[cfg(test)]
#[path = "bootstrap_tests.rs"]
mod tests;
