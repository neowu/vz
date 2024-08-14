#!/bin/bash -e
cargo build --release
codesign --force --entitlement resources/vz.entitlements --sign - target/release/vz

sudo cp ./target/release/vz /usr/local/bin
vz complete | sudo tee /usr/local/share/zsh/site-functions/_vz
