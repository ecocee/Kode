"""Shared constants for the database benchmark harness."""

import os

HERE = os.path.dirname(os.path.abspath(__file__))          # benchmarks/database/common
DB_DIR = os.path.dirname(HERE)                             # benchmarks/database
ROOT = os.path.dirname(os.path.dirname(DB_DIR))            # repository root

COMPOSE_FILE = os.path.join(DB_DIR, "docker-compose.yml")
CASES_FILE = os.path.join(DB_DIR, "workloads", "cases.json")
SCHEMA_FILE = os.path.join(DB_DIR, "schema", "schema.sql")
SEED_FILE = os.path.join(DB_DIR, "schema", "seed.sql")
RESULTS_DIR = os.path.join(DB_DIR, "results")
RAW_DIR = os.path.join(RESULTS_DIR, "raw")
REPORTS_DIR = os.path.join(RESULTS_DIR, "reports")

# Languages with a real PostgreSQL driver in this suite.
DRIVER_LANGUAGES = ["rust", "go", "node", "java", "python", "csharp"]
# Kode is listed so its absence is reported, never silently omitted.
ALL_LANGUAGES = DRIVER_LANGUAGES + ["kode"]

WORKLOADS = [
    "insert",
    "select",
    "update",
    "delete",
    "join",
    "transactions",
    "concurrency",
    "connections",
    "application",
]

STATUS_PASS = "PASS"
STATUS_FAIL = "FAIL"
STATUS_TIMEOUT = "TIMEOUT"
STATUS_UNSUPPORTED = "UNSUPPORTED"
STATUS_NOT_RUN = "NOT RUN"

# Pinned in benchmarks/database/Dockerfile. Recorded in every report so a number
# can always be traced to the exact driver and runtime that produced it.
LANGUAGE_VERSIONS = {
    "rust": {
        "runtime": "rust:1-bookworm (rustc 1.9x, glibc)",
        "driver": "postgres 0.19.x (rust-postgres)",
        "pool": "r2d2 0.8 + r2d2_postgres 0.18",
        "build": "cargo build --release",
    },
    "go": {
        "runtime": "golang:1.24-bookworm, static binary on alpine:3.20",
        "driver": "github.com/lib/pq 1.10.9",
        "pool": "database/sql",
        "build": "go build -trimpath -ldflags='-s -w'",
    },
    "node": {
        "runtime": "node:22-alpine",
        "driver": "pg 8.13.1",
        "pool": "pg.Pool",
        "build": "no build step (plain JS)",
    },
    "java": {
        "runtime": "eclipse-temurin 21 JRE",
        "driver": "org.postgresql:postgresql 42.7.4",
        "pool": "HikariCP 5.1.0",
        "build": "mvn package (shaded jar)",
    },
    "python": {
        "runtime": "python:3.12-slim",
        "driver": "psycopg 3.2.3 (binary)",
        "pool": "psycopg_pool 3.2.3",
        "build": "no build step",
    },
    "csharp": {
        "runtime": ".NET 8 (mcr.microsoft.com/dotnet/runtime:8.0)",
        "driver": "Npgsql 8.0.5",
        "pool": "NpgsqlDataSource pool",
        "build": "dotnet publish -c Release",
    },
    "kode": {
        "runtime": "Kode 0.3.x native binary",
        "driver": "none",
        "pool": "none",
        "build": "kode build --release",
    },
}

# Cases that are generator-shaped rather than single statements.
SPECIAL_SHAPES = {
    "batch_update",
    "batch_delete",
    "chunk_insert",
    "chunk_insert_txn",
    "txn_case",
    "conc",
    "conn",
    "app_order_txn",
}

DEFAULT_ROWS = 10000
DEFAULT_REPETITIONS = 5
DEFAULT_OPS = 200
DEFAULT_TIMEOUT_S = 600
DEFAULT_WORKERS = [10, 50, 100]
