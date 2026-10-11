#!/usr/bin/env bash
set -euo pipefail

# Each invocation has its own feature graph, outside the repository workspace.
export CARGO_TARGET_DIR="$PWD/target/feature-consumer"
for feature in facade-minimal facade-default facade-fluent core-minimal core-fluent collection-minimal collection-default collection-selected collection-fluent; do
    cargo run --manifest-path ci/feature-consumer/Cargo.toml --locked \
        --no-default-features --features "$feature"
done
