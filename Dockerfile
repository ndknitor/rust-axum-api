# ---------- BUILD STAGE ----------
FROM rust:latest AS builder

WORKDIR /app

# Install musl target + tools
RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install -y musl-tools ca-certificates

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN rm -rf src

# Copy real source
COPY src ./src

# Force rebuild of the binary (not just dependencies)
RUN touch src/main.rs && cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/rust-axum-api /app/rust-axum-api

# # Copy static files
# COPY wwwroot /app/wwwroot

# Create non-root user
RUN useradd -r -s /bin/false appuser
USER appuser

# Expose port
EXPOSE 3000

# Set environment variables
ENV HOST=0.0.0.0
ENV PORT=3000

CMD ["/app/rust-axum-api"]

# # ---------- RUNTIME STAGE ----------
# FROM scratch

# WORKDIR /app

# # Copy binary
# COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/rust-axum-api /app/rust-axum-api

# # Copy CA certificates for HTTPS
# COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# # Use non-root UID (no useradd in scratch)
# USER 1000:1000

# EXPOSE 3000

# ENV HOST=0.0.0.0
# ENV PORT=3000

# ENTRYPOINT ["/app/rust-axum-api"]
