SHELL := /bin/sh

CARGO := cargo
WORKSPACE := --workspace
CLIPPY_FLAGS := -- -D warnings

DEBUG :=
RELEASE := --release

PACKAGE := chess-gui

.DEFAULT_GOAL := help

.PHONY: run
run: ## Run the chess GUI in debug mode.
	@$(CARGO) run -p $(PACKAGE) $(DEBUG)

.PHONY: run-release
run-release: ## Run the chess GUI in release mode.
	@$(CARGO) run -p $(PACKAGE) $(RELEASE)

.PHONY: build
build: ## Build the workspace.
	@$(CARGO) build $(WORKSPACE) $(DEBUG)

.PHONY: release
release: ## Build the workspace in release mode.
	@$(CARGO) build $(WORKSPACE) $(RELEASE)

.PHONY: check
check: ## Check the workspace without producing binaries.
	@$(CARGO) check $(WORKSPACE) --all-targets --all-features

.PHONY: fmt
fmt: ## Format Rust code.
	@$(CARGO) fmt --all

.PHONY: clippy
clippy: ## Run Clippy with warnings treated as errors.
	@$(CARGO) clippy $(WORKSPACE) --all-targets --all-features $(CLIPPY_FLAGS)

.PHONY: lint
lint: fmt-check clippy ## Run formatting and Clippy checks.

.PHONY: fmt-check
fmt-check: ## Check Rust formatting without modifying files.
	@$(CARGO) fmt --all -- --check

.PHONY: test
test: ## Run all workspace tests.
	@$(CARGO) test $(WORKSPACE) --all-features

.PHONY: bench
bench: ## Run all workspace benchmarks.
	@$(CARGO) bench $(WORKSPACE) --all-features

.PHONY: doc
doc: ## Build workspace documentation.
	@$(CARGO) doc $(WORKSPACE) --no-deps --all-features

.PHONY: miri
miri: ## Run tests under Miri.
	@$(CARGO) miri test $(WORKSPACE) --all-features

.PHONY: ci 
ci: fmt clippy test ## Run CI checks.

.PHONY: help
help: ## Show available commands.
	@printf '\nChess development commands\n\n'
	@awk 'BEGIN {FS = ":.*##"; printf "Usage: make <target>\\n\\nTargets:\\n"} \
		/^[a-zA-Z0-9_-]+:.*##/ {printf "  %-10s %s\\n", $$1, $$2}' \
		$(MAKEFILE_LIST)
	@printf '\n'
