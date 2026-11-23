//! Ordinary player transports are independent of the family record verifier.
use super::*;

impl Harness {
    /// Run projection requests from a separate admitted peer. Only the original
    /// guest belongs to the signed record lane used for replay verification.
    pub async fn use_ordinary_player(&mut self, identity: u8) {
        self.player = Some(identity);
        if self.players.contains_key(&identity) {
            return;
        }
        let player = Arc::new(
            P2pandaTransport::builder_from_seed([identity; 32])
                .alpns(vec![host_carrier::projection_alpn()])
                .bind()
                .await
                .expect("ordinary player's projection transport"),
        );
        assert_ne!(player.local_peer_id(), self.guest.local_peer_id());
        player
            .add_peer(self.owner.endpoint_addr().await.unwrap())
            .await
            .unwrap();
        self.players.insert(identity, player);
    }

    pub(super) fn projection_player(&self) -> (&Arc<P2pandaTransport>, u8) {
        match self.player {
            Some(key) => (&self.players[&key], key),
            None => (&self.guest, self.seed.wrapping_add(3)),
        }
    }
}
