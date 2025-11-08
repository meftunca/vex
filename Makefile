# Vex Language - Makefile

.PHONY: all build test clean docs docs-watch

# Default target
all: build test

# Build the project
build:
	cargo build

# Run tests
test:
	./test_all.sh

# Clean build artifacts
clean:
	cargo clean
	rm -f vex-builds/*

# Documentation
docs:
	./scripts/update_docs.sh

# Watch for changes and update docs
docs-watch:
	while true; do \
		inotifywait -qre modify .; \
		./scripts/update_docs.sh; \
		sleep 1; \
	done

# Development workflow
dev: build test docs

# Release build
release:
	cargo build --release

# Install development tools
install-dev:
	cargo install cargo-watch
	cargo install cargo-expand