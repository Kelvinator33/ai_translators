FROM rust:1.75 as builder

# Install system dependencies needed for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy and cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

# Copy actual source and build
COPY src ./src
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the built binary and static files
COPY --from=builder /app/target/release/task043 ./app
COPY static ./static

# Make binary executable
RUN chmod +x ./app

# Create data directory for database
RUN mkdir -p /app/data

# Set environment
ENV RUST_LOG=info
ENV DATABASE_PATH=/app/data/translation_service.db
ENV PORT=3000

EXPOSE 3000

CMD ["./app"]