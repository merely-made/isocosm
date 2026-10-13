// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A stable endpoint routing ID may have several successive connections.
//! Only the currently installed connection may negotiate, submit or withdraw.

use super::*;

impl HostSession {
    fn clear_live_peer(&mut self, peer: PeerId) {
        self.peer_names.remove(&peer);
        self.peer_endpoints.remove(&peer);
        self.negotiated.remove(&peer);
        self.refused.remove(&peer);
    }

    /// The in-process driver opens a real new logical connection. It does
    /// not authenticate an endpoint or inherit an earlier native binding.
    pub fn begin_connection(&mut self, peer: PeerId) -> Result<(u64,Vec<Outbound>),String> {
        if matches!(peer,PeerId::HOST | PeerId::UNATTRIBUTED) {
            return Err("connection uses a reserved routing identity".into());
        }
        let connection = self.next_connection.checked_add(1).ok_or("connection sequence exhausted")?;
        self.next_connection = connection;
        self.clear_live_peer(peer);
        self.connections.insert(peer,connection);
        Ok((connection,self.on_connect(peer)))
    }

    /// Endpoint proof belongs to the trusted transport, never to Hello/name.
    /// A new connection must negotiate even when its endpoint is unchanged.
    pub fn begin_authenticated_connection(&mut self, peer: PeerId, endpoint: [u8;32])
        -> Result<(u64,Vec<Outbound>),String> {
        if self.peer_endpoints.get(&peer).is_some_and(|held| *held != endpoint) {
            return Err("authenticated endpoint conflicts with an occupied routing identity".into());
        }
        let result = self.begin_connection(peer)?;
        self.peer_endpoints.insert(peer,endpoint);
        Ok(result)
    }

    pub fn current_connection(&self, peer: PeerId, connection: u64) -> bool {
        self.connections.get(&peer) == Some(&connection)
    }

    /// A delayed old reader has no authority over a replacement connection.
    pub fn on_connection_message(&mut self, peer: PeerId, connection: u64, message: NetMessage) -> Vec<Outbound> {
        if !self.current_connection(peer,connection) { return Vec::new(); }
        self.on_message(peer,message)
    }

    pub fn disconnect_connection(&mut self, peer: PeerId, connection: u64) -> bool {
        if !self.current_connection(peer,connection) { return false; }
        self.connections.remove(&peer);
        self.clear_live_peer(peer);
        // Saved native peers, player rosters/holds, admitted state/history and
        // already received action requests do not belong to the live channel.
        true
    }
}
