#!/bin/sh

err() {
	>&2 echo "$@"
	exit 1
}

usage() {
	err "./register.sh <deployed contract address> <your testnet address> <your git repository>"
}

[ -z "$SPN_SUPERPOSITION_KEY" ] && \
	err "SPN_SUPERPOSITION_KEY needs to be set with a private key for submission (export SPN_SUPERPOSITION_KEY=<your private key>) before running this script"

if [ -z "$1" ] || [ -z "$2" ] || [ -z "$3" ]; then usage; fi

cast send \
	--rpc-url https://testnet-rpc.superposition.so \
	--private-key "$SPN_SUPERPOSITION_KEY" \
	0x301fa1a4e2c1d543efc4237209507f168df00eb3 \
	'register(address,address,string)' \
	$1 \
	$2 \
	$3
