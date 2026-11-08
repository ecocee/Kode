"""PostgreSQL lifecycle, statistics and resource sampling for the harness."""

import json
import os
import re
import subprocess
import threading
import time

import config


class DatabaseError(RuntimeError):
    pass


class Database:
    """The single shared PostgreSQL instance, driven through docker compose."""

    def __init__(self, compose_file=None, verbose=False):
        self.compose = ["docker", "compose", "-f", compose_file or config.COMPOSE_FILE]
        self.verbose = verbose

    # ---------------------------------------------------------------- plumbing
    def _exec(self, args, stdin=None, timeout=180, check=True):
        proc = subprocess.run(
            self.compose + args,
            input=stdin,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=timeout,
        )
        if check and proc.returncode != 0:
            raise DatabaseError(
                "docker compose " + " ".join(args) + " failed:\n" + proc.stderr.decode("utf-8", "replace")
            )
        return proc

    def psql(self, sql, timeout=180, check=True):
        """Run SQL as the postgres superuser and return unaligned output."""
        proc = self._exec(
            ["exec", "-T", "postgres", "psql", "-X", "-q", "-v", "ON_ERROR_STOP=1",
             "-U", "postgres", "-d", "kode_bench", "-t", "-A", "-F", "|", "-f", "-"],
            stdin=sql.encode(),
            timeout=timeout,
            check=check,
        )
        return proc.stdout.decode("utf-8", "replace").strip()

    def psql_file(self, path, variables=None, timeout=300):
        with open(path, "rb") as f:
            content = f.read()
        vars_args = []
        for key, value in (variables or {}).items():
            vars_args += ["-v", f"{key}={value}"]
        proc = self._exec(
            ["exec", "-T", "postgres", "psql", "-X", "-q", "-v", "ON_ERROR_STOP=1",
             "-U", "postgres", "-d", "kode_bench"] + vars_args + ["-f", "-"],
            stdin=content,
            timeout=timeout,
        )
        return proc.stdout.decode("utf-8", "replace").strip()

    # ---------------------------------------------------------------- lifecycle
    def up(self, timeout=180):
        self._exec(["up", "-d", "postgres"], timeout=timeout)
        deadline = time.time() + timeout
        last = ""
        while time.time() < deadline:
            proc = self._exec(["exec", "-T", "postgres", "psql", "-X", "-q", "-U", "postgres",
                               "-d", "kode_bench", "-t", "-A", "-c", "SELECT 1"],
                              timeout=20, check=False)
            if proc.returncode == 0:
                return True
            last = proc.stderr.decode("utf-8", "replace")[-300:]
            time.sleep(0.5)
        raise DatabaseError("PostgreSQL did not become ready: " + last)

    def down(self, keep=False):
        if keep:
            return
        self._exec(["down", "--remove-orphans"], timeout=180, check=False)

    def apply_schema(self):
        return self.psql_file(config.SCHEMA_FILE)

    def reset(self, rows):
        """Truncate and re-seed; every invocation starts from identical data."""
        started = time.time()
        self.psql_file(config.SEED_FILE, {"rows": rows})
        return (time.time() - started) * 1000.0

    def server_info(self):
        row = self.psql(
            "SELECT current_setting('server_version'), current_setting('server_version_num'),"
            " pg_size_pretty(pg_database_size(current_database())),"
            " current_setting('max_connections'), current_setting('shared_buffers'),"
            " current_setting('fsync'), current_setting('default_transaction_isolation');"
        )
        parts = row.split("|")
        keys = ["version", "version_num", "database_size", "max_connections",
                "shared_buffers", "fsync", "isolation"]
        return dict(zip(keys, parts))

    # ---------------------------------------------------------------- statistics
    def stats(self):
        """Server-side totals attributable to the client role, plus deadlocks."""
        out = self.psql(
            "SELECT COALESCE((SELECT sum(s.total_exec_time) FROM pg_stat_statements s"
            " JOIN pg_roles r ON r.oid = s.userid WHERE r.rolname = 'bench'), 0)::text"
            " || '|' || (SELECT deadlocks FROM pg_stat_database"
            "            WHERE datname = current_database())::text;"
        )
        db_ms, deadlocks = (out.split("|") + ["0", "0"])[:2]
        return {"db_total_ms": float(db_ms or 0), "deadlocks": int(float(deadlocks or 0))}

    def cgroup(self):
        """Sampled cgroup v2 counters for the postgres container."""
        proc = self._exec(
            ["exec", "-T", "postgres", "sh", "-c",
             "cat /sys/fs/cgroup/memory.current 2>/dev/null; "
             "awk '/^usage_usec/{print $2}' /sys/fs/cgroup/cpu.stat 2>/dev/null"],
            timeout=20, check=False,
        )
        if proc.returncode != 0:
            return None
        parts = proc.stdout.decode("utf-8", "replace").split()
        if len(parts) < 2:
            return None
        return {"mem_bytes": int(parts[0]), "cpu_usec": int(parts[1])}

    def active_client_connections(self):
        out = self.psql(
            "SELECT count(*) FROM pg_stat_activity WHERE usename = 'bench';",
            timeout=20, check=False,
        )
        try:
            return int(out)
        except ValueError:
            return 0

    # ---------------------------------------------------------------- db timing
    def explain_ms(self, sql, params):
        """Server-side execution time of one representative query.

        Wrapped in a transaction that is always rolled back, so an INSERT/DELETE
        used for measurement never changes benchmark state.
        """
        statement = sql
        for index in range(len(params), 0, -1):
            statement = statement.replace(f"${index}", literal(params[index - 1]))
        script = ("BEGIN;\nEXPLAIN (ANALYZE, TIMING OFF, SUMMARY ON, FORMAT JSON) "
                  + statement + ";\nROLLBACK;\n")
        # A statement that cannot run in the rolled-back transaction (for
        # example an INSERT whose key the case just wrote) costs its db_exec
        # attribution, never the measurement itself.
        out = self.psql(script, check=False)
        match = re.search(r'"Execution Time":\s*([0-9.]+)', out)
        if not match:
            return None
        return float(match.group(1))


def literal(value):
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, (int, float)):
        return str(value)
    return "'" + str(value).replace("'", "''") + "'"


class Sampler(threading.Thread):
    """Samples database connections and container resources while a client runs."""

    def __init__(self, db, interval=0.5):
        super().__init__(daemon=True)
        self.db = db
        self.interval = interval
        self.stop_event = threading.Event()
        self.samples = []

    def run(self):
        while not self.stop_event.is_set():
            entry = {"t": time.time(), "connections": None, "mem_bytes": None, "cpu_usec": None}
            try:
                entry["connections"] = self.db.active_client_connections()
            except Exception:
                pass
            try:
                cg = self.db.cgroup()
                if cg:
                    entry["mem_bytes"] = cg["mem_bytes"]
                    entry["cpu_usec"] = cg["cpu_usec"]
            except Exception:
                pass
            if any(entry[k] is not None for k in ("connections", "mem_bytes", "cpu_usec")):
                self.samples.append(entry)
            self.stop_event.wait(self.interval)

    def stop(self):
        self.stop_event.set()
        self.join(timeout=5)

    def summary(self):
        conns = [s["connections"] for s in self.samples if s["connections"] is not None]
        mems = [s["mem_bytes"] for s in self.samples if s["mem_bytes"] is not None]
        cpus = [s["cpu_usec"] for s in self.samples if s["cpu_usec"] is not None]
        return {
            "db_conns_max": max(conns) if conns else 0,
            "db_conns_avg": round(sum(conns) / len(conns), 2) if conns else 0.0,
            "db_mem_mb": round(max(mems) / 1048576.0, 1) if mems else 0.0,
            "db_cpu_s": round((max(cpus) - min(cpus)) / 1e6, 3) if len(cpus) > 1 else 0.0,
            "samples": len(self.samples),
        }
