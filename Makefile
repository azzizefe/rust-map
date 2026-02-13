# GeoPic Makefile

BINARY_NAME=rust-map
TARGET_DIR=target/release

all: build

build:
	@echo "🛠️ Building GeoPic..."
	cargo build --release

install: build
	@echo "🚚 Installing binary to /usr/local/bin/..."
	sudo cp $(TARGET_DIR)/$(BINARY_NAME) /usr/local/bin/geopic
	@echo "✅ Installed! run 'geopic <file>' to search."

clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

help:
	@echo "GeoPic Build System"
	@echo "-------------------"
	@echo "make build   - Build the release binary"
	@echo "make install - Build and install to /usr/local/bin"
	@echo "make clean   - Remove build artifacts"
