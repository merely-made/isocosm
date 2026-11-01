//! A table's projections as the VTT page shows them, from any endpoint: the
//! host's own, or a peer's over its admitted stream.

use super::*;

/// The displayed snapshot's invocation for one page action, refused unless
/// its card still advertises it.
pub(crate) fn invocation(snapshots: &BTreeMap<String, ProjectionSnapshot>, request: VttActionRequest)
-> Result<IntentInvocation, String> {
    let snapshot = snapshots
        .get(&request.projection)
        .ok_or("VTT action names a projection that is not currently displayed")?;
    let target = snapshot
        .scene
        .active_items_in_order()
        .into_iter()
        .find_map(|(instance, _)| (card_identifier(instance) == request.card).then_some(instance))
        .ok_or("VTT action target is absent from the displayed projection")?;
    let advertised = snapshot
        .presentation
        .offers_for(target)
        .into_iter()
        .flatten()
        .flat_map(|offer| offer.semantics.actions.iter())
        .any(|action| action.intent.0 == request.action);
    if !advertised {
        return Err("VTT action is no longer advertised by its displayed card".into());
    }
    Ok(IntentInvocation {
        session: snapshot.session.clone(),
        target,
        observed_epoch: SceneEpoch(request.epoch),
        observed_revision: Revision(request.revision),
        intent: request.action,
        payload: request.payload,
    })
}

pub(crate) fn projection_view(
    fetch: &mut dyn FnMut(ResourceRequest) -> Result<chirograph::ResourceResponse, String>,
    label: &str,
    id: &str,
    snapshot: &ProjectionSnapshot,
) -> Result<VttProjectionView, String> {
    let mut cards = Vec::new();
    for (instance, item) in snapshot.scene.active_items_in_order() {
        let source = snapshot.scene.tables.sources[item.source.0 as usize]
            .as_ref()
            .ok_or("VTT projection has an absent item source")?;
        let offers = snapshot
            .presentation
            .offers_for(instance)
            .ok_or("VTT card has no presentation offer")?;
        let portable = offers
            .iter()
            .find(|offer| offer.codec == chirograph::PresentationCodec::PortableCardV1);
        let card = match portable {
            Some(offer) => {
                let response = fetch(ResourceRequest {
                    session: snapshot.session.clone(),
                    resource: offer.resource.clone(),
                })?;
                serde_json::from_slice::<PortableCardV1>(&response.bytes)
                    .map_err(|error| format!("VTT portable card is invalid: {error}"))?
            },
            None => PortableCardV1 {
                title: source.id.clone(),
                values: Vec::new(),
                badges: Vec::new(),
                media: Vec::new(),
            },
        };
        let actions = offers
            .iter()
            .flat_map(|offer| offer.semantics.actions.iter())
            .map(|action| VttActionView {
                id: action.intent.0.clone(),
                label: action.label.clone(),
                explanation: action.explanation.clone(),
                payload_schema: action.payload_schema.clone(),
            })
            .collect();
        cards.push(VttCardView {
            id: card_identifier(instance),
            title: card.title,
            values: card
                .values
                .into_iter()
                .map(|value| VttCardValue {
                    label: value.label,
                    value: value.value,
                })
                .collect(),
            badges: card.badges,
            actions,
        });
    }
    Ok(VttProjectionView {
        id: id.to_owned(),
        label: label.to_owned(),
        epoch: snapshot.scene.epoch.0,
        revision: snapshot.scene.revision.0,
        cards,
    })
}

pub(crate) fn card_identifier(instance: impl std::fmt::Debug) -> String {
    format!("{instance:?}")
}
