# Stage 1: Builder
FROM rust:1.78 AS builder

# Install protobuf-compiler (if your project or its dependencies need it)
# RUN apt-get update && apt-get install -y protobuf-compiler

# Install ADB tools
RUN apt-get update && \
    apt-get install -y --no-install-recommends android-sdk-platform-tools-common adb && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/tuya-monitor
COPY . .

# Build the application
# Ensure Cargo.lock is up-to-date before building
RUN cargo fetch
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install ADB tools and ca-certificates (for HTTPS requests)
RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates adb && \
    rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/tuya-monitor/target/release/tuya /usr/local/bin/tuya-monitor

# Copy the .env file for environment variables (will be sourced by the run command or entrypoint script)
# Alternatively, pass environment variables via `docker run --env-file`
COPY .env .env

WORKDIR /app

# Set the entrypoint
# The CMD will pass arguments to adb_wrapper.sh
CMD ["/usr/local/bin/tuya-monitor"]
