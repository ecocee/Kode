-- Runs once per container start (PGDATA is a tmpfs, so initdb runs every time).
--
-- postgres  : superuser, used only by the harness for DDL, seeding and stats.
-- bench     : the role every language client logs in as. Deliberately NOT a
--             superuser, so pg_stat_statements can attribute server time to the
--             clients alone and a client cannot read other sessions' statistics.
CREATE ROLE bench LOGIN PASSWORD 'bench' NOSUPERUSER NOCREATEDB NOCREATEROLE;
GRANT ALL ON DATABASE kode_bench TO bench;
GRANT USAGE, CREATE ON SCHEMA public TO bench;
