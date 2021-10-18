#!/usr/bin/env bash

# Initializing build environment


rustup install nightly-2021-08-01
rustup target add wasm32-unknown-unknown --toolchain nightly-2020-08-01-x86_64-unknown-linux-gnu
