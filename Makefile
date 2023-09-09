CARGO_BIN := $(shell which cargo)

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

all: build lint fmt test install
