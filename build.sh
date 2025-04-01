#!/bin/sh

cargo build \
	--release \
	--target wasm32-unknown-unknown

mv \
	target/wasm32-unknown-unknown/release/libexamplebucharesthashing.wasm \
	contract.wasm
