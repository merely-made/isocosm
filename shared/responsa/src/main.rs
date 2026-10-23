// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! eidetic's responsa on this repository's eidetic pin; the pre-commit hook
//! builds and runs it (DOC_POLICY core §11).

fn main() -> std::process::ExitCode {
    responsa::cli::main()
}
