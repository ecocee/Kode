
FROM golang:1.24-bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    curl \
    nodejs \
    python3 \
    procps \
    && rm -rf /var/lib/apt/lists/*

# Install Rust.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --default-toolchain stable

ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /src

COPY . /src

# Build the Kode compiler.
RUN cargo build --release --bin kode

# Prepare benchmark binaries.
RUN mkdir -p /bench/bin

# Build the Kode HTTP server.
RUN /src/target/release/kode build \
    /src/benchmarks/http/server.kode \
    --release \
    -o /bench/bin/kode_server

# Build the Go HTTP server.
RUN go build \
    -o /bench/bin/go_server \
    /src/benchmarks/http/server.go

# Build the custom Go load generator.
RUN go build \
    -o /bench/bin/load \
    /src/benchmarks/http/load.go

WORKDIR /bench

CMD ["bash"]
