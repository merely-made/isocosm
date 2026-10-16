// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

fn main() {
    println!("cargo:rerun-if-env-changed=MESOCOSM_STACK_BYTES");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        let bytes = std::env::var("MESOCOSM_STACK_BYTES")
            .map(|value| {
                value
                    .parse::<u64>()
                    .expect("MESOCOSM_STACK_BYTES must be an integer")
            })
            .unwrap_or(8 * 1024 * 1024);
        assert!(
            bytes >= 1024 * 1024,
            "MESOCOSM_STACK_BYTES must be at least one MiB"
        );
        println!("cargo:rustc-link-arg-bin=mesocosm-genet=/STACK:{bytes}");
    }
}
