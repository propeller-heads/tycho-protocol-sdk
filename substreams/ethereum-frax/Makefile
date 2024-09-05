CARGO_VERSION := $(shell cargo version 2>/dev/null)
START_BLOCK := 12369300
STOP_BLOCK ?= +1000
ENDPOINT ?= mainnet.eth.streamingfast.io:443

.PHONY: build
build:
ifdef CARGO_VERSION
	cargo build --target wasm32-unknown-unknown --release
else
	@echo "Building substreams target using Docker. To speed up this step, install a Rust development environment."
	docker run --rm -ti --init -v ${PWD}:/usr/src --workdir /usr/src/ rust:bullseye cargo build --target wasm32-unknown-unknown --release
endif

.PHONY: run
run: build
	substreams run substreams.yaml $(if $(MODULE),$(MODULE),map_protocol_changes) $(if $(START_BLOCK),-s $(START_BLOCK)) $(if $(STOP_BLOCK),-t $(STOP_BLOCK))

.PHONY: gui
gui: build
	substreams gui substreams.yaml $(if $(MODULE),$(MODULE),map_protocol_changes) $(if $(START_BLOCK),-s $(START_BLOCK)) $(if $(STOP_BLOCK),-t $(STOP_BLOCK))

.PHONY: protogen
protogen:
	substreams protogen ./substreams.yaml --exclude-paths="sf/substreams,google"

.PHONY: pack
pack: build
	substreams pack substreams.yaml

.PHONY: stream
stream: build
	substreams run -e $(ENDPOINT) substreams.yaml map_protocol_changes -s 12369621 -t $(STOP_BLOCK)
