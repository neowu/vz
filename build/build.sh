#!/bin/bash -e
cargo build
codesign --force --entitlement resources/vz.entitlements --sign - target/debug/vz
