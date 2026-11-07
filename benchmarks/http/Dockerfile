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

# ---------------------------------------------------------------- kode
FROM rust:1-bookworm AS kode-build
WORKDIR /src
COPY . /src
RUN cargo build --release --bin kode \
 && mkdir -p /out \
 && /src/target/release/kode build /src/benchmarks/http/server.kode --release -o /out/kode_server

FROM debian:bookworm-slim AS kode
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