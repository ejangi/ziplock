FROM ubuntu:22.04

# Prevent interactive prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=UTC

# Install system dependencies in a single layer
RUN apt-get update && apt-get install -y \
    # Build essentials
    build-essential \
    pkg-config \
    curl \
    ca-certificates \
    file \
    binutils \
    # GUI libraries
    libfontconfig1-dev \
    libfreetype6-dev \
    libx11-dev \
    libxft-dev \
    liblzma-dev \
    libgtk-4-dev \
    libadwaita-1-dev \
    libatk1.0-dev \
    libatk-bridge2.0-dev \
    libgtk-3-dev \
    libgdk-pixbuf-2.0-dev \
    libglib2.0-dev \
    libcairo2-dev \
    libpango1.0-dev \
    # Packaging tools
    dpkg-dev \
    fakeroot \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Install Rust toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    --default-toolchain stable \
    --target x86_64-unknown-linux-gnu \
    --component rustfmt,clippy

# Set up environment
ENV PATH="/root/.cargo/bin:${PATH}"
ENV CARGO_TARGET_DIR="/workspace/target"

# Verify installation
RUN rustc --version && cargo --version

# Create workspace directory
WORKDIR /workspace

# Pre-compile common dependencies to speed up builds
# This creates a dummy project to cache common dependencies
RUN cargo init --name dummy \
    && echo 'serde = { version = "1.0", features = ["derive"] }' >> Cargo.toml \
    && echo 'tokio = { version = "1.0", features = ["full"] }' >> Cargo.toml \
    && cargo build --release \
    && rm -rf src Cargo.toml Cargo.lock target/release/deps/dummy* target/release/dummy

# Set default command
CMD ["/bin/bash"]
