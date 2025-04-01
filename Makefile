
build: contract.wasm

.PHONY: build

contract.wasm: $(shell find src -type f -name '*.rs')
	@rm -f contract.wasm
	@./build.sh
