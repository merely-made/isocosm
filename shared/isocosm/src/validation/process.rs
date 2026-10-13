// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

pub(super) fn contracts(process: &Process, rules: &Rules) -> Result<()> {
    super::body::process(process)?;
    process.validate_feeding(rules)?;
    super::harm::draws(process)?;
    process.validate_goals(rules)
}
