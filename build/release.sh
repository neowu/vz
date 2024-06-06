#!/bin/bash -e
cargo build --release
codesign --force --entitlement resources/vz.entitlements --sign - target/release/vz
