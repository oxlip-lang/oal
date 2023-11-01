CARGO_BIN := $(shell which cargo)
WASM_PACK := $(shell which wasm-pack)

ifeq ($(CARGO_BIN),)
$(error cargo not found)
endif

.DEFAULT_GOAL := build

.PHONY: build
build:
	$(CARGO_BIN) build

.PHONY: lint
lint:
	$(CARGO_BIN) clippy

.PHONY: fmt
fmt:
	$(CARGO_BIN) fmt --all

.PHONY: test
test:
	$(CARGO_BIN) test

.PHONY: install
install:
	$(CARGO_BIN) install --path oal-client

.PHONY: wasm
ifeq ($(WASM_PACK),)
wasm:
	@echo "wasm-pack not found" && exit 1
else
wasm:
	$(WASM_PACK) build oal-wasm
endif

all: fmt lint build test install
