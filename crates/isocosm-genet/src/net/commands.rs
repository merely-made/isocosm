// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

pub(super) fn drain_commands(commands: &Receiver<BridgeCommand>) -> Result<Vec<BridgeCommand>, ()> {
    let mut drained = Vec::new();
    loop {
        match commands.try_recv() {
            Ok(command) => drained.push(command),
            Err(TryRecvError::Empty) => return Ok(drained),
            Err(TryRecvError::Disconnected) => return Err(()),
        }
    }
}
