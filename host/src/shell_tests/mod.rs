// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The desktop shell's own tests, over the library's fixture. The shell
//! retires behind a Graphshell endpoint (ruling 896).

#[cfg(not(feature = "fixtures"))]
compile_error!("the shell's product tests need the library's `fixtures` feature");

mod file_save;
mod products;
mod reading;
mod viewport;

pub(crate) use isocosm_host::fixtures::*;
pub(crate) use std::collections::BTreeSet;
pub(crate) use isocosm::{Command, Execution, Session};
pub(crate) use isocosm_overlay::Seat;
pub(crate) use isocosm_host::world::World;
