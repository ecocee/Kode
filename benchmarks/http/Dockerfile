# syntax=docker/dockerfile:1.7
#
# One Dockerfile, one target per language. Build context = repo root.
#   docker build --target java -f benchmarks/http/Dockerfile -t kode-http-bench-java .
#
# Every server image contains ONLY the runtime + the compiled server, so
# memory / image-size numbers are not polluted by compilers or toolchains.
# Every server accepts the port as its first argument and reads THREADS
# from the environment (run.sh sets it).

# ---------------------------------------------------------------- load
FROM golang:1.24-bookworm AS load-build
WORKDIR /src
COPY benchmarks/http/load.go .
RUN CGO_ENABLED=0 go build -o /out/load load.go

FROM alpine:3.20 AS load
RUN apk add --no-cache curl
COPY --from=load-build /out/load /bench/bin/load
CMD ["tail", "-f", "/dev/null"]

# ------------------------------------------------ idle (keep-alive holders)
# Opens N connections, one request each, then holds them idle and reports how
# many the server kept. Built from the same Go toolchain stage as `load`.
FROM golang:1.24-bookworm AS idle-build
WORKDIR /src
COPY benchmarks/http/idle.go .
RUN CGO_ENABLED=0 go build -o /out/idle idle.go

FROM alpine:3.20 AS idle
RUN apk add --no-cache curl
COPY --from=idle-build /out/idle /bench/bin/idle
CMD ["tail", "-f", "/dev/null"]

# ------------------------------------------------------- oha (independent)
# The second, independent load generator Stage 2 asks for (the first is `wrk`).
# Pinned: a benchmark number is only comparable with the tool that produced it.
FROM alpine:3.20 AS oha
ARG OHA_VERSION=1.16.0
ARG ARCH=arm64
RUN apk add --no-cache curl \
 && mkdir -p /bench/bin \
 && curl -fsSL -o /bench/bin/oha \
      "https://github.com/hatoo/oha/releases/download/v${OHA_VERSION}/oha-linux-${ARCH}" \
 && chmod +x /bench/bin/oha
CMD ["tail", "-f", "/dev/null"]

# --------------------------------------------------------------- wrk (second)
# Built from source because there is no arm64 image for a prebuilt wrk.
FROM debian:bookworm-slim AS wrk
ARG WRK_VERSION=4.2.0
RUN apt-get update \
 && apt-get install -y --no-install-recommends build-essential libssl-dev ca-certificates git unzip \
 && git clone --depth 1 --branch "${WRK_VERSION}" https://github.com/wg/wrk /tmp/wrk \
 && make -C /tmp/wrk -j"$(nproc)" \
 && install -D /tmp/wrk/wrk /bench/bin/wrk \
 && rm -rf /tmp/wrk /var/lib/apt/lists/*
CMD ["tail", "-f", "/dev/null"]

# ------------------------------------------------------- tools (load side)
# One container with every load generator, so a Stage 2 run uses the same
# network and the same image for oha, wrk and idle.
# Debian, not Alpine: `wrk` is linked against glibc and would fail to exec on musl.
FROM debian:bookworm-slim AS tools
RUN apt-get update \
 && apt-get install -y --no-install-recommends curl ca-certificates \
 && rm -rf /var/lib/apt/lists/*
COPY --from=oha /bench/bin/oha /bench/bin/oha
COPY --from=wrk /bench/bin/wrk /bench/bin/wrk
COPY --from=idle /bench/bin/idle /bench/bin/idle
ENV PATH="/bench/bin:${PATH}"
CMD ["tail", "-f", "/dev/null"]

# ---------------------------------------------------------------- kode
FROM rust:1-trixie AS kode-build
# `kode build` shells out to clang for the LLVM object; the Rust image has none.
RUN apt-get update \
 && apt-get install -y --no-install-recommends clang \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY . /src
RUN cargo build --release --bin kode \
 && mkdir -p /out \
 && /src/target/release/kode build /src/benchmarks/http/server.kode --release -o /out/kode_server

FROM debian:trixie-slim AS kode
COPY --from=kode-build /out/kode_server /bench/bin/kode_server
ENTRYPOINT ["/bench/bin/kode_server"]
CMD ["8080"]

# ---------------------------------------------------------------- go
FROM golang:1.24-bookworm AS go-build
WORKDIR /src
COPY benchmarks/http/server.go .
RUN CGO_ENABLED=0 go build -o /out/go_server server.go

FROM alpine:3.20 AS go
COPY --from=go-build /out/go_server /bench/bin/go_server
ENTRYPOINT ["/bench/bin/go_server"]
CMD ["8080"]

# ---------------------------------------------------------------- node
FROM node:22-alpine AS node
COPY benchmarks/http/server.js /app/server.js
ENTRYPOINT ["node", "/app/server.js"]
CMD ["8080"]

# ---------------------------------------------------------------- java
FROM eclipse-temurin:21-jdk AS java-build
WORKDIR /src
COPY benchmarks/http/java/Server.java .
RUN javac -d /out Server.java

FROM eclipse-temurin:21-jre AS java
COPY --from=java-build /out /app
# Heap is sized from the container limit, so low --memory values still work.
ENTRYPOINT ["java", "-XX:MaxRAMPercentage=70", "-XX:+UseSerialGC", "-Xshare:auto", \
            "-Dsun.net.httpserver.nodelay=true", "-Dsun.net.httpserver.maxIdleConnections=100000", \
            "-cp", "/app", "Server"]
CMD ["8080"]

# ---------------------------------------------------------------- php
# PHP's built-in server is not keep-alive friendly, so PHP is benchmarked with
# Swoole (the usual production-grade async PHP HTTP server).
FROM phpswoole/swoole:php8.3 AS php
COPY benchmarks/http/php/server.php /app/server.php
ENTRYPOINT ["php", "/app/server.php"]
CMD ["8080"]

# ---------------------------------------------------------------- python
FROM python:3.12-slim AS python
COPY benchmarks/http/server.py /app/server.py
ENTRYPOINT ["python", "-u", "/app/server.py"]
CMD ["8080"]

# ---------------------------------------------------------------- csharp
FROM mcr.microsoft.com/dotnet/sdk:8.0 AS csharp-build
WORKDIR /src
COPY benchmarks/http/csharp/ .
RUN dotnet publish -c Release -o /out

FROM mcr.microsoft.com/dotnet/aspnet:8.0 AS csharp
COPY --from=csharp-build /out /app
ENTRYPOINT ["dotnet", "/app/server.dll"]
CMD ["8080"]