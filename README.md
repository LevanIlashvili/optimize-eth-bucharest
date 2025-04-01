
# ETH Bucharest reference contract

This is the example contract to fork and develop on top of for the ETH Bucharest Stylus
Gas Redux challenge.

The goal of the hackathon is to find cheaper solutions when the "prove" function is
called, returning output that can be verified by the reference prover
[https://github.com/af-afk/ethbucharest.bayge.xyz/blob/trunk/src/prover.rs](here).

You can read more about ETH Bucharest hashing
[https://stylus-saturdays.com/i/159344476/introducing-bucharest-pow](here).

## Getting started

### Dependencies

|        Description     |                               Link                              |
|------------------------|-----------------------------------------------------------------|
| Standard foundry suite | [https://book.getfoundry.sh/getting-started/installation](link) |

Make is not literally needed, but it can be useful as a frontend to `build.sh`, which
invokes Cargo this way:

	cargo build \
		--release \
		--target wasm32-unknown-unknown

## Building

	./build.sh

## Testing

Simple testing is possible using `proptest` to call the default-provided `solve` function,
then to call it again with the same outputs.

To compare your custom algorithm against the online contract, you can simply test your
contract's output against

## Hard requirements for solutions

Be careful! If you don't follow these goals, the remote contract won't be able to verify
your solution.

1. The online prover must reproduce the same lower and upper values when called

2. Siphash must be used for the hashing function
