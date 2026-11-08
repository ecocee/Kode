-- Deterministic seed data, identical for every language implementation.
--
-- psql variables (set by the harness with -v):
--   rows  size of the read dataset (bench_events) and the write dataset
--         (bench_writes) for the current invocation.
--
-- The application tables keep fixed sizes so join and report numbers are
-- comparable across dataset sweeps: 1,000 products, 10,000 customers and
-- 100,000 orders.

\set ON_ERROR_STOP on

TRUNCATE products, customers, orders, bench_events, bench_writes RESTART IDENTITY;

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

-- Read dataset: this is the only place the benchmark's row formula is written
-- for SQL. Every client reproduces it for parameters only.
INSERT INTO bench_events (id, k, payload, tag)
SELECT g + 1,
       (g * 37) % :rows,
       'payload-' || g,
       't' || (g % 10)
FROM generate_series(0, :rows - 1) g;

-- The write dataset is seeded by the same rule, so an update/delete workload
-- sees exactly the rows an insert workload would have produced.
INSERT INTO bench_writes (id, k, payload, tag)
SELECT g + 1,
       (g * 37) % :rows,
       'payload-' || g,
       't' || (g % 10)
FROM generate_series(0, :rows - 1) g;

ANALYZE products;
ANALYZE customers;
ANALYZE orders;
ANALYZE bench_events;
ANALYZE bench_writes;

SELECT 'products' AS table, count(*) FROM products
UNION ALL SELECT 'customers', count(*) FROM customers
UNION ALL SELECT 'orders', count(*) FROM orders
UNION ALL SELECT 'bench_events', count(*) FROM bench_events
UNION ALL SELECT 'bench_writes', count(*) FROM bench_writes;
