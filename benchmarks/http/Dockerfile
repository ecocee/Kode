# One Linux image with Kode, Go, Node and Python, so every server runs on the
# same kernel, the same 1 vCPU and the same RAM.
# Go's own image, because Debian's golang-go (1.19) is too old for the load
# tool; Rust is installed on top of it.
FROM golang:1.24-bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        curl \
        nodejs \
        python3 \
        procps \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --default-toolchain stable
ENV PATH=/root/.cargo/bin:$PATH

# The load tool: `hey` speaks keep-alive, takes a fixed request count and prints
# the 99th percentile.
ENV GOPATH=/root/go
ENV PATH=/root/go/bin:$PATH
RUN go install github.com/rakyll/hey@latest

WORKDIR /src
COPY . /src
RUN cargo build --release --bin kode

WORKDIR /bench
CMD ["bash", "/bench/measure.sh"]
