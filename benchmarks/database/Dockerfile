# syntax=docker/dockerfile:1.7
#
# One Dockerfile, one target per language. Build context = repository root.
#   docker build --target python -f benchmarks/database/Dockerfile -t kode-db-bench-python .
#
# Every client image contains only the driver and the compiled client: no source
# tree, no build cache. Each client reads /bench/workloads/cases.json, so all
# languages execute the same SQL, the same iteration counts and the same
# verification queries.
#
# Driver versions are pinned. They are the established driver for each language,
# used the way its documentation recommends (prepared statements in loops, the
# native pool implementation for pooled cases).

# ---------------------------------------------------------------- python
# psycopg 3 is the current psycopg generation; psycopg_pool is its pool.
FROM python:3.12-slim AS python
RUN pip install --no-cache-dir "psycopg[binary]==3.2.3" "psycopg_pool==3.2.3"
COPY benchmarks/database/python/client.py /bench/client.py
COPY benchmarks/database/workloads /bench/workloads
WORKDIR /bench
ENTRYPOINT ["python", "-u", "/bench/client.py"]

# ---------------------------------------------------------------- node
FROM node:22-alpine AS node
WORKDIR /bench
RUN npm install --no-audit --no-fund --omit=dev pg@8.13.1
COPY benchmarks/database/node/client.js /bench/client.js
COPY benchmarks/database/workloads /bench/workloads
ENTRYPOINT ["node", "/bench/client.js"]

# ---------------------------------------------------------------- go
# lib/pq is the long-standing pure-Go PostgreSQL driver, so the runtime image
# needs no libc and the reported RSS is the client alone.
FROM golang:1.24-bookworm AS go-build
WORKDIR /src
COPY benchmarks/database/go/go.mod benchmarks/database/go/go.sum* ./
RUN go mod download
COPY benchmarks/database/go/client.go ./
RUN CGO_ENABLED=0 go build -trimpath -ldflags="-s -w" -o /out/client ./...

FROM alpine:3.20 AS go
RUN adduser -D bench
COPY --from=go-build /out/client /bench/client
COPY benchmarks/database/workloads /bench/workloads
USER bench
ENTRYPOINT ["/bench/client"]

# ---------------------------------------------------------------- rust
# `postgres` (rust-postgres) is the maintained synchronous driver; r2d2 is the
# standard pool used around it.
FROM rust:1-bookworm AS rust-build
WORKDIR /src
COPY benchmarks/database/rust/Cargo.toml ./
COPY benchmarks/database/rust/src ./src
RUN cargo build --release --offline 2>/dev/null || cargo build --release
RUN cp target/release/kode-db-bench /out-client

FROM debian:bookworm-slim AS rust
COPY --from=rust-build /out-client /bench/client
COPY benchmarks/database/workloads /bench/workloads
ENTRYPOINT ["/bench/client"]

# ---------------------------------------------------------------- java
# PostgreSQL JDBC 42.x + HikariCP, shaded into one jar.
FROM maven:3.9-eclipse-temurin-21 AS java-build
WORKDIR /src
COPY benchmarks/database/java/pom.xml ./
RUN mvn -q -B dependency:go-offline
COPY benchmarks/database/java/src ./src
RUN mvn -q -B -DskipTests package

FROM eclipse-temurin:21-jre AS java
COPY --from=java-build /src/target/db-bench.jar /bench/client.jar
COPY benchmarks/database/workloads /bench/workloads
# Heap from the container limit; SerialGC keeps JVM overhead predictable.
ENTRYPOINT ["java", "-XX:MaxRAMPercentage=70", "-XX:+UseSerialGC", "-Xshare:auto", \
            "-cp", "/bench/client.jar", "bench.Client"]

# ---------------------------------------------------------------- csharp
# Npgsql 8 with its built-in data source pooling.
FROM mcr.microsoft.com/dotnet/sdk:8.0 AS csharp-build
WORKDIR /src
COPY benchmarks/database/csharp/db-bench.csproj ./
RUN dotnet restore
COPY benchmarks/database/csharp/*.cs ./
RUN dotnet publish -c Release -o /out --no-restore

FROM mcr.microsoft.com/dotnet/runtime:8.0 AS csharp
COPY --from=csharp-build /out /bench/app
COPY benchmarks/database/workloads /bench/workloads
WORKDIR /bench
ENTRYPOINT ["dotnet", "/bench/app/db-bench.dll"]

# ---------------------------------------------------------------- kode
# The Kode compiler is built from this repository, then compiles the client to a
# standalone native binary. The PostgreSQL client is part of the Kode runtime
# library linked into that binary (ADR 0033): no driver package, no pool.
FROM rust:1-bookworm AS kode-build
WORKDIR /src
COPY . /src
RUN cargo build --release --bin kode \
 && mkdir -p /out \
 && /src/target/release/kode build /src/benchmarks/database/kode/client.kode --release -o /out/client

FROM debian:bookworm-slim AS kode
COPY --from=kode-build /out/client /bench/client
COPY benchmarks/database/workloads /bench/workloads
WORKDIR /bench
ENTRYPOINT ["/bench/client"]