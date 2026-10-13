// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use eponym_play::{NativeHandoff,timed_action::TimedActionSession};
use eponym_play::contract::mode::{SpatialBindingInput,actions::ActionConfig};

/// Consumes an already restored real owner. A CLI may restore once with the
/// canonical solver before this transfer; this helper never loads/replays it.
pub fn take_timed(owner:TimedActionSession,spatial:&SpatialBindingInput,actions:&ActionConfig)
    -> Result<NativeHandoff,(TimedActionSession,String)> {
    owner.into_native_mode(spatial,actions)
}
