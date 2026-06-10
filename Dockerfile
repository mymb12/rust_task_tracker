# ── Stage 1: Build ──────────────────────────────────────
FROM rust:1.92 AS builder

# Install dependencies needed to compile sqlx (it links against OpenSSL)
RUN apt-get update && apt-get install -y \
  pkg-config \
  libssl-dev \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy dependency files first (Cargo.toml and Cargo.lock)
# We do this BEFORE copying source code — explanation below
COPY Cargo.toml Cargo.lock ./

# Trick: create a fake main.rs so cargo can download dependencies
# without needing your real source code yet
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 
RUN rm -rf src

# Now copy your REAL source code
COPY src ./src
RUN touch src/main.rs && cargo build --release

# ── Stage 2: Run ────────────────────────────────────────
# Switch to a tiny base image — we don't need the Rust compiler anymore
# debian:bookworm-slim is ~80MB vs the rust image's ~1.4GB
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
  ca-certificates \
  libssl3 \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app 

# Copy only the compiled binary from stage 1
COPY --from=builder /app/target/release/task-tracker .

# Copy your static files (your repo has a /static folder)
COPY static ./static

# Tell Docker this app listens on port 6570
EXPOSE 6570

# The command that runs when the container starts
CMD ["./task-tracker", "serve"]
