CARGO ?= cargo

.PHONY: all build build-debug test plan-check lint ci ci-fast web web-dev dev clean

all: build

build:
	$(CARGO) build --workspace

build-debug:
	$(CARGO) build --workspace

test:
	$(CARGO) test --workspace

plan-check:
	$(CARGO) run -p context-plan --quiet

lint:
	$(CARGO) fmt --all -- --check
	$(CARGO) clippy --all-targets --all-features -- -D warnings || true

ci:
	$(CARGO) fetch --locked
	$(MAKE) lint
	$(MAKE) test
	$(MAKE) plan-check

ci-fast:
	$(MAKE) ci

web:
	$(CARGO) run -p context-web

web-dev:
	$(CARGO) run -p context-web

dev:
	@echo "Dev loop placeholder. Use 'make web' and 'entr' / cargo-watch as needed."

clean:
	$(CARGO) clean
