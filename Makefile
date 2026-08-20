# Chandra - development and release tasks.
#
# Every target here is also what CI runs, so a green `make check` locally means
# a green pipeline.

SHELL := /bin/bash
APP := target/release/bundle/macos/Chandra.app
INSTALLED := /Applications/Chandra.app
VENDOR := $(shell ls -d $$HOME/.cargo/registry/src/*/swiss-eph-0.2.1/vendor/swisseph 2>/dev/null | head -1)

.PHONY: help dev build install uninstall run test lint fmt check golden swetest clean

help:
	@grep -E '^[a-z-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-12s\033[0m %s\n", $$1, $$2}'

dev: ## Run the app against the Vite dev server
	npx tauri dev

build: ## Build the release .app bundle
	npx tauri build --bundles app

dmg: ## Build a distributable .dmg
	npx tauri build --bundles app,dmg

install: build ## Install to /Applications, ad-hoc signed, and launch
	@echo "==> replacing $(INSTALLED)"
	@pkill -f "Chandra.app/Contents/MacOS/chandra" 2>/dev/null || true
	@rm -rf "$(INSTALLED)"
	@cp -R "$(APP)" "$(INSTALLED)"
	@# Ad-hoc signature: Gatekeeper then asks once on first launch instead of
	@# refusing outright. No Apple Developer account is involved (D-012).
	@codesign --force --deep --sign - "$(INSTALLED)"
	@xattr -dr com.apple.quarantine "$(INSTALLED)" 2>/dev/null || true
	@open "$(INSTALLED)"
	@echo "==> Chandra is in the menu bar"

uninstall: ## Remove the app and its settings
	@pkill -f "Chandra.app/Contents/MacOS/chandra" 2>/dev/null || true
	@rm -rf "$(INSTALLED)"
	@rm -rf "$$HOME/Library/Application Support/com.parasdsingh.chandra"

run: ## Run the release binary in the foreground, showing its output
	cargo run --release -p chandra

test: ## Run the whole test suite
	cargo test --workspace
	npx tsc --noEmit

lint: ## Formatting and lint checks
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets -- -D warnings
	npx tsc --noEmit

fmt: ## Format everything
	cargo fmt --all

check: lint test ## Everything CI runs

swetest: ## Build Swiss Ephemeris' reference CLI, used to generate golden vectors
	@test -n "$(VENDOR)" || (echo "swiss-eph sources not vendored; run 'cargo fetch' first" && exit 1)
	@mkdir -p tools
	cc -O2 -o tools/swetest \
		$(VENDOR)/swetest.c $(VENDOR)/swedate.c $(VENDOR)/swehouse.c \
		$(VENDOR)/swejpl.c $(VENDOR)/swemmoon.c $(VENDOR)/swemplan.c \
		$(VENDOR)/sweph.c $(VENDOR)/swephlib.c $(VENDOR)/swecl.c \
		$(VENDOR)/swehel.c -I$(VENDOR) -lm

golden: swetest ## Regenerate the ephemeris golden vectors
	@echo "Only do this for a deliberate change, never to make a failing test pass."
	python3 tools/gen_golden.py > crates/ephemeris/tests/golden.json

preview: ## Regenerate the fixture behind the front-end visual harness
	cargo run -q -p chandra --example preview_data > src/dev/fixture.json
	@echo "Open http://localhost:5273/?preview with 'npm run dev' running."

clean:
	cargo clean
	rm -rf dist tools/swetest
