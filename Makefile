.DEFAULT_GOAL := help

APP := take
SCRIPT ?= demo.take
ARGS ?=

.PHONY: help build run play check test fmt lint ci release install clean

help:
	@printf "\n"
	@printf "Usage: make <target>\n\n"
	@printf "Targets:\n"
	@printf "  %-12s %s\n" "build" "Build the project (debug)."
	@printf "  %-12s %s\n" "run" "Run the CLI (use ARGS='...')."
	@printf "  %-12s %s\n" "play" "Run 'take play' (use SCRIPT='file.take')."
	@printf "  %-12s %s\n" "check" "Type-check and compile without linking."
	@printf "  %-12s %s\n" "test" "Run tests."
	@printf "  %-12s %s\n" "fmt" "Format Rust code."
	@printf "  %-12s %s\n" "lint" "Run clippy and fail on warnings."
	@printf "  %-12s %s\n" "ci" "Run fmt + lint + test checks."
	@printf "  %-12s %s\n" "release" "Build optimized release binary."
	@printf "  %-12s %s\n" "install" "Install binary from this path."
	@printf "  %-12s %s\n" "clean" "Remove Cargo build artifacts."
	@printf "\n"
	@printf "Examples:\n"
	@printf "  make run ARGS=\"--help\"\n"
	@printf "  make play SCRIPT=\"examples/demo.take\"\n"
	@printf "\n"

build:
	cargo build

run:
	cargo run -- $(ARGS)

play:
	cargo run -- play $(SCRIPT)

check:
	cargo check --all-targets

test:
	cargo test

fmt:
	cargo fmt --all

lint:
	cargo clippy --all-targets -- -D warnings

ci: fmt lint test

release:
	cargo build --release

install:
	cargo install --path .

clean:
	cargo clean
