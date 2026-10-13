// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Live channel/handshake custody on successive connections of one endpoint.

use super::*;

pub(super) fn spawn_accept(host: &HostNet) {
    let endpoint = host.endpoint.clone();
    let session = host.session.clone();
    let peers = host.peers.clone();
    tokio::spawn(async move {
        while let Some(connecting) = endpoint.accept().await {
            let Ok(conn) = connecting.await else { continue };
            let peer = peer_of(&conn);
            // A failed stream open has not installed a new live connection.
            let Ok((send,recv)) = conn.open_bi().await else { continue };
            let mut authority = session.lock().await;
            let (connection,out) = match authority.begin_authenticated_connection(peer,*conn.remote_id().as_bytes()) {
                Ok(result) => result,Err(_) => continue,
            };
            let (tx,rx) = mpsc::unbounded_channel();
            peers.lock().await.insert(peer,tx);
            tokio::spawn(writer_task(send,rx));
            // Snapshot and live tail use this same installed channel. Holding
            // the authority through enqueue prevents a successor or local act
            // overtaking publication of this current snapshot.
            dispatch(&peers,out).await;
            tokio::spawn(reader(conn,recv,peer,connection,session.clone(),peers.clone()));
        }
    });
}

async fn reader(_conn: Connection, mut recv: RecvStream, peer: PeerId, connection: u64,
    session: Arc<Mutex<HostSession>>, peers: PeerMap) {
    loop {
        let Ok(message) = read_frame(&mut recv).await else { break };
        let mut authority = session.lock().await;
        if !authority.current_connection(peer,connection) { break; }
        let out = authority.on_connection_message(peer,connection,message);
        // Lock order is authority -> peer channels at both admission and
        // lifecycle boundaries, so a reply cannot target a replacement channel.
        dispatch(&peers,out).await;
    }
    let mut authority = session.lock().await;
    if authority.disconnect_connection(peer,connection) {
        peers.lock().await.remove(&peer);
    }
}
