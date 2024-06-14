# Default platform is linux/amd64
# ARG PLATFORM=linux/amd64
ARG BUILDPLATFORM=linux/amd64
ARG TARGETPLATFORM=linux/amd64

# Use the official Rust image as the build environment
FROM --platform=$BUILDPLATFORM rust:latest AS builder

# Install necessary dependencies for building Rust projects
RUN apt-get update && apt-get install -y \
    build-essential \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Create a new directory for our application
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock config.yaml ./

# Copy the source code
COPY src ./src

# Build the application in release mode
RUN cargo build --release

# Use a minimal base image for the final stage
FROM --platform=$TARGETPLATFORM rust:latest

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the build stage
COPY --from=builder /app/target/release/rust-beam /usr/local/bin/rust-beam
COPY --from=builder /app/config.yaml /config.yaml

# Set the startup command to run your binary
CMD ["rust-beam", "relay"]
