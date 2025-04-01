#!/bin/sh -e

# Make sure you set the environment variable (SPN_SUPERPOSITION_KEY) to
# your private key for deployment to Superposition Testnet!

export SPN_SUPERPOSITION_URL=https://testnet-rpc.superposition.so

if [ -z "$SPN_SUPERPOSITION_KEY" ]; then
	>&2 echo "SPN_SUPERPOSITION_KEY needs to be set with a private key for deploy"
	exit 1
fi

# We deploy the wasm contract after using the Makefile build process! We
# skip the verification so that it's faster to take this live and to see
# the response from the node.

cargo stylus deploy \
	--endpoint $SPN_SUPERPOSITION_URL \
	--wasm-file contract.wasm \
	--no-verify \
	--private-key $SPN_SUPERPOSITION_KEY
