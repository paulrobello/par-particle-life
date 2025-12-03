# Makefile for par-particle-life

.PHONY: build clean format lint test checkall release run help

# Default target
all: build

# Build debug version
build:
	cargo build

# Build release version
release:
	cargo build --release

# Run the application (release mode)
run:
	cargo run --release

# Clean build artifacts
clean:
	cargo clean

# Format code with rustfmt
format:
	cargo fmt

# Lint code with clippy
lint:
	cargo clippy -- -D warnings

# Run tests
test:
	cargo test

# Check all: format, lint, and test
checkall: format lint test
	@echo "All checks passed!"

# Type check only (fast)
check:
	cargo check

# Help
help:
	@echo "Available targets:"
	@echo "  build      - Build debug version"
	@echo "  release    - Build release version"
	@echo "  run        - Run the application (release mode)"
	@echo "  bundle     - Create macOS App Bundle"
	@echo "  run-bundle - Create and run macOS App Bundle"
	@echo "  clean      - Clean build artifacts"
	@echo "  format     - Format code with rustfmt"
	@echo "  lint       - Lint code with clippy"
	@echo "  test       - Run tests"
	@echo "  checkall   - Run format, lint, and test"
	@echo "  check      - Type check only (fast)"
	@echo "  help       - Show this help message"

APP_NAME = ParParticleLife
BUNDLE_DIR = target/release/bundle/$(APP_NAME).app
MACOS_DIR = $(BUNDLE_DIR)/Contents/MacOS
RES_DIR = $(BUNDLE_DIR)/Contents/Resources

bundle: release
	@echo "Creating App Bundle..."
	@mkdir -p $(MACOS_DIR)
	@mkdir -p $(RES_DIR)
	@cp target/release/par-particle-life $(MACOS_DIR)/$(APP_NAME)
	@chmod +x $(MACOS_DIR)/$(APP_NAME)
	@echo '<?xml version="1.0" encoding="UTF-8"?>' > $(BUNDLE_DIR)/Contents/Info.plist
	@echo '<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">' >> $(BUNDLE_DIR)/Contents/Info.plist
	@echo '<plist version="1.0">' >> $(BUNDLE_DIR)/Contents/Info.plist
	@echo '<dict>' >> $(BUNDLE_DIR)/Contents/Info.plist
	@echo '    <key>CFBundleExecutable</key>' >> $(BUNDLE_DIR)/Contents/Info.plist
	@echo '    <string>$(APP_NAME)</string>' >> $(BUNDLE_DIR)/Contents/Info.plist
	@echo '    <key>CFBundleIdentifier</key>' >> $(BUNDLE_DIR)/Contents/Info.plist
	@echo '    <string>com.paulrobello.par-particle-life</string>' >> $(BUNDLE_DIR)/Contents/Info.plist
	@echo '    <key>CFBundleName</key>' >> $(BUNDLE_DIR)/Contents/Info.plist
	@echo '    <string>$(APP_NAME)</string>' >> $(BUNDLE_DIR)/Contents/Info.plist
	@echo '    <key>CFBundleIconFile</key>' >> $(BUNDLE_DIR)/Contents/Info.plist
	@echo '    <string>AppIcon</string>' >> $(BUNDLE_DIR)/Contents/Info.plist
	@echo '    <key>CFBundlePackageType</key>' >> $(BUNDLE_DIR)/Contents/Info.plist
	@echo '    <string>APPL</string>' >> $(BUNDLE_DIR)/Contents/Info.plist
	@echo '    <key>NSHighResolutionCapable</key>' >> $(BUNDLE_DIR)/Contents/Info.plist
	@echo '    <true/>' >> $(BUNDLE_DIR)/Contents/Info.plist
	@echo '</dict>' >> $(BUNDLE_DIR)/Contents/Info.plist
	@echo '</plist>' >> $(BUNDLE_DIR)/Contents/Info.plist
	@mkdir -p target/release/icon.iconset
	@cp assets/icon_32.png target/release/icon.iconset/icon_16x16@2x.png 2>/dev/null || true
	@cp assets/icon_32.png target/release/icon.iconset/icon_32x32.png 2>/dev/null || true
	@cp assets/icon_64.png target/release/icon.iconset/icon_32x32@2x.png 2>/dev/null || true
	@cp assets/icon_256.png target/release/icon.iconset/icon_128x128@2x.png 2>/dev/null || true
	@cp assets/icon_256.png target/release/icon.iconset/icon_256x256.png 2>/dev/null || true
	@iconutil -c icns target/release/icon.iconset -o $(RES_DIR)/AppIcon.icns 2>/dev/null || cp assets/icon.png $(RES_DIR)/AppIcon.png
	@rm -rf target/release/icon.iconset
	@echo "Bundle created at $(BUNDLE_DIR)"

run-bundle: bundle
	@echo "Running App Bundle..."
	@open $(BUNDLE_DIR)
