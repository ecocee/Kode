-- Kode database benchmark schema.
--
-- One schema is used by every language implementation: the same tables, the same
-- indexes, the same constraints. Anything that would give one client an advantage
-- (an extra index, a different column type, a partition) belongs in a new ADR, not
-- in a per-language setup script.
--
-- Drop and recreate so a run always starts from a known state.

DROP TABLE IF EXISTS orders;
DROP TABLE IF EXISTS customers;
DROP TABLE IF EXISTS products;
DROP TABLE IF EXISTS bench_events;
DROP TABLE IF EXISTS bench_writes;

-- ---------------------------------------------------------------- application
-- The realistic inventory workload (workload "application"): products,
-- customers and orders, with the foreign keys a real shop needs.

CREATE TABLE products (
    id          BIGINT PRIMARY KEY,
    sku         TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL,
    category    TEXT NOT NULL,
    price       NUMERIC(12, 2) NOT NULL,
    stock       INTEGER NOT NULL CHECK (stock >= 0),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX products_category_idx ON products (category);

CREATE TABLE customers (
    id          BIGINT PRIMARY KEY,
    email       TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL,
    city        TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE orders (
    id           BIGINT PRIMARY KEY,
    customer_id  BIGINT NOT NULL REFERENCES customers (id),
    product_id   BIGINT NOT NULL REFERENCES products (id),
    qty          INTEGER NOT NULL CHECK (qty > 0),
    unit_price   NUMERIC(12, 2) NOT NULL,
    status       TEXT NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX orders_customer_idx ON orders (customer_id);
CREATE INDEX orders_status_idx ON orders (status);

-- ---------------------------------------------------------------- micro tables
-- bench_events is the read dataset. It is seeded once per workload invocation
-- with a deterministic formula (see schema/seed.sql) and never written.
--
--   id     1..rows          primary key
--   k      (i * 37) % rows  indexed
--   tag    "t" || (i % 10)  deliberately not indexed (full scan case)

CREATE TABLE bench_events (
    id       BIGINT PRIMARY KEY,
    k        BIGINT NOT NULL,
    payload  TEXT NOT NULL,
    tag      TEXT NOT NULL,
    note     TEXT
);

CREATE INDEX bench_events_k_idx ON bench_events (k);

-- bench_writes is the write dataset: insert / update / delete / transaction and
-- concurrency workloads. Same shape as bench_events, same indexes, so read and
-- write timings refer to the same physical layout.
CREATE TABLE bench_writes (
    id       BIGINT PRIMARY KEY,
    k        BIGINT NOT NULL,
    payload  TEXT NOT NULL,
    tag      TEXT NOT NULL,
    note     TEXT
);

CREATE INDEX bench_writes_k_idx ON bench_writes (k);

-- pg_stat_statements gives per-role server execution time, which is how this
-- suite separates database time from client-side driver time.
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

-- ---------------------------------------------------------------- reset helpers
-- Untimed per-case resets, defined once so every client calls the same SQL and
-- no implementation can pick a different starting state. Clients run these
-- before a case, outside the measured section.

CREATE OR REPLACE FUNCTION bench_clear_writes() RETURNS void LANGUAGE sql AS $$
    TRUNCATE bench_writes;
$$;

CREATE OR REPLACE FUNCTION bench_reset_writes(n bigint) RETURNS void LANGUAGE sql AS $$
    TRUNCATE bench_writes;
    INSERT INTO bench_writes (id, k, payload, tag)
    SELECT g + 1, (g * 37) % n, 'payload-' || g, 't' || (g % 10)
    FROM generate_series(0, n - 1) g;
$$;

-- Canonical state of the three application tables (1,000 products, 10,000
-- customers, 100,000 orders), identical to schema/seed.sql.
CREATE OR REPLACE FUNCTION bench_reset_app() RETURNS void LANGUAGE sql AS $$
    TRUNCATE products, customers, orders RESTART IDENTITY;
    INSERT INTO products (id, sku, name, category, price, stock)
    SELECT g + 1,
           'SKU-' || lpad((g + 1)::text, 6, '0'),
           'product-' || (g + 1),
           (ARRAY['books', 'games', 'tools', 'music'])[(g % 4) + 1],
           (5 + (g % 50))::numeric(12, 2),
           100 + (g % 400)
    FROM generate_series(0, 999) g;
    INSERT INTO customers (id, email, name, city)
    SELECT g + 1,
           'customer' || (g + 1) || '@example.com',
           'customer-' || (g + 1),
           (ARRAY['Bengaluru', 'Berlin', 'Lisbon', 'Osaka'])[(g % 4) + 1]
    FROM generate_series(0, 9999) g;
    INSERT INTO orders (id, customer_id, product_id, qty, unit_price, status)
    SELECT g + 1,
           ((g::bigint * 7919) % 10000) + 1,
           ((g::bigint * 104729) % 1000) + 1,
           (g % 5) + 1,
           (5 + (g % 50))::numeric(12, 2),
           (ARRAY['new', 'paid', 'shipped', 'cancelled'])[(g % 4) + 1]
    FROM generate_series(0, 99999) g;
$$;

-- Clients log in as the non-superuser `bench` role (schema/init-roles.sql).
GRANT USAGE ON SCHEMA public TO bench;
GRANT SELECT, INSERT, UPDATE, DELETE, TRUNCATE ON ALL TABLES IN SCHEMA public TO bench;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO bench;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA public TO bench;
