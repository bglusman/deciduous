.PHONY: build release debug test test-verbose clean install uninstall serve analyze gen-test-files fmt lint check help

# Default target
all: release

# Build targets
release:
	cargo build --release

debug:
	cargo build

build: release

# Testing
test:
	cargo test

test-verbose:
	cargo test -- --nocapture

test-filter:
	@test -n "$(FILTER)" || (echo "Usage: make test-filter FILTER=pattern" && exit 1)
	cargo test $(FILTER)

# Code quality
fmt:
	cargo fmt

fmt-check:
	cargo fmt -- --check

lint:
	cargo clippy -- -D warnings

check:
	cargo check

# Run the analyzer
analyze:
	@test -n "$(FILE)" || (echo "Usage: make analyze FILE=path/to/file" && exit 1)
	cargo run --release -- $(FILE)

analyze-dir:
	@test -n "$(DIR)" || (echo "Usage: make analyze-dir DIR=path/to/directory" && exit 1)
	cargo run --release -- $(DIR)

analyze-no-spectral:
	@test -n "$(FILE)" || (echo "Usage: make analyze-no-spectral FILE=path/to/file" && exit 1)
	cargo run --release -- --no-spectral $(FILE)

# Generate reports
report-html:
	@test -n "$(FILE)" || (echo "Usage: make report-html FILE=path/to/file" && exit 1)
	cargo run --release -- -o report.html $(FILE)

report-json:
	@test -n "$(FILE)" || (echo "Usage: make report-json FILE=path/to/file" && exit 1)
	cargo run --release -- -o report.json $(FILE)

# Interactive web UI
serve:
	@echo "Starting web UI at http://localhost:3000"
	cargo run --release -- serve $(or $(DIR),.) --port $(or $(PORT),3000)

# Generate test files (requires ffmpeg, lame, sox)
gen-test-files:
	@command -v ffmpeg >/dev/null || (echo "Error: ffmpeg not found" && exit 1)
	@command -v lame >/dev/null || (echo "Error: lame not found" && exit 1)
	@command -v sox >/dev/null || (echo "Error: sox not found" && exit 1)
	./examples/generate_test_files.sh

# Analyze demo files
demo:
	@test -d examples/demo_files || (echo "Run 'make gen-test-files' first" && exit 1)
	cargo run --release -- examples/demo_files/

# Installation
install: release
	cp target/release/losselot /usr/local/bin/

uninstall:
	rm -f /usr/local/bin/losselot

# Clean build artifacts
clean:
	cargo clean

clean-reports:
	rm -f report.html report.json report.csv

clean-all: clean clean-reports

# Documentation
doc:
	cargo doc --open

# Development helpers
watch:
	@command -v cargo-watch >/dev/null || (echo "Install cargo-watch: cargo install cargo-watch" && exit 1)
	cargo watch -x test

bench:
	@test -n "$(FILE)" || (echo "Usage: make bench FILE=path/to/file" && exit 1)
	@echo "Timing analysis..."
	time cargo run --release -- $(FILE)

# Help
help:
	@echo "Losselot - Audio Forensics Tool"
	@echo ""
	@echo "Build:"
	@echo "  make              Build release binary"
	@echo "  make release      Build release binary"
	@echo "  make debug        Build debug binary"
	@echo ""
	@echo "Test:"
	@echo "  make test         Run all tests"
	@echo "  make test-verbose Run tests with output"
	@echo "  make test-filter FILTER=pattern  Run specific tests"
	@echo ""
	@echo "Code Quality:"
	@echo "  make fmt          Format code"
	@echo "  make fmt-check    Check formatting"
	@echo "  make lint         Run clippy linter"
	@echo "  make check        Quick compile check"
	@echo ""
	@echo "Analyze:"
	@echo "  make analyze FILE=path           Analyze a file"
	@echo "  make analyze-dir DIR=path        Analyze a directory"
	@echo "  make analyze-no-spectral FILE=path  Fast binary-only analysis"
	@echo "  make demo                        Analyze demo files"
	@echo ""
	@echo "Reports:"
	@echo "  make report-html FILE=path       Generate HTML report"
	@echo "  make report-json FILE=path       Generate JSON report"
	@echo ""
	@echo "Server:"
	@echo "  make serve                       Start web UI on port 3000"
	@echo "  make serve DIR=path PORT=8080    Custom directory and port"
	@echo ""
	@echo "Test Files:"
	@echo "  make gen-test-files              Generate test audio files"
	@echo "                                   (requires ffmpeg, lame, sox)"
	@echo ""
	@echo "Install:"
	@echo "  make install      Install to /usr/local/bin"
	@echo "  make uninstall    Remove from /usr/local/bin"
	@echo ""
	@echo "Clean:"
	@echo "  make clean        Remove build artifacts"
	@echo "  make clean-reports Remove generated reports"
	@echo "  make clean-all    Remove everything"
	@echo ""
	@echo "Dev:"
	@echo "  make watch        Auto-run tests on change (needs cargo-watch)"
	@echo "  make bench FILE=path  Time analysis of a file"
	@echo "  make doc          Build and open documentation"
