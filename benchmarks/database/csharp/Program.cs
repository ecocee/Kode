// PostgreSQL benchmark client for C# (.NET 8 + Npgsql 8).
//
// One workload per process, one JSON record per case on stdout. Commands are
// prepared once per case and reused with new parameter values, which is the
// Npgsql pattern for repeated statements. Concurrency uses the data source's
// built-in pool with one held connection per worker.

using System.Diagnostics;
using System.Text.Json;
using System.Text.RegularExpressions;
using Npgsql;

namespace Bench;

internal static class Program
{
    private const string Language = "csharp";
    private static readonly string[] Statuses = { "new", "paid", "shipped", "cancelled" };
    private static readonly string[] Categories = { "books", "games", "tools", "music" };
    private static readonly string[] Cities = { "Bengaluru", "Berlin", "Lisbon", "Osaka" };
    private const string InsertRowSql = "INSERT INTO bench_writes (id, k, payload, tag) VALUES ($1,$2,$3,$4)";

    private static string _workload = "";
    private static string _casesPath = "/bench/workloads/cases.json";
    private static long _rows = 10_000;
    private static long _ops = 200;
    private static string _connectionString = "";
    private static string _directConnectionString = "";
    private static readonly List<string> Only = new();

    private static int Main(string[] args)
    {
        ParseArgs(args);
        using JsonDocument doc = JsonDocument.Parse(File.ReadAllText(_casesPath));
        int chunk = doc.RootElement.TryGetProperty("chunk", out JsonElement chunkElement)
            ? chunkElement.GetInt32() : 500;

        var cases = new List<JsonElement>();
        foreach (JsonElement node in doc.RootElement.GetProperty("cases").EnumerateArray())
        {
            if (node.GetProperty("workload").GetString() != _workload)
            {
                continue;
            }
            if (Only.Count > 0 && !Only.Contains(node.GetProperty("id").GetString() ?? ""))
            {
                continue;
            }
            cases.Add(node);
        }
        if (cases.Count == 0)
        {
            Console.WriteLine(JsonSerializer.Serialize(new Dictionary<string, object?>
            {
                ["language"] = Language, ["workload"] = _workload, ["error"] = "no cases",
            }));
            return 2;
        }

        using NpgsqlDataSource dataSource = NpgsqlDataSource.Create(_connectionString);
        using NpgsqlConnection conn = dataSource.OpenConnection();

        foreach (JsonElement c in cases)
        {
            long iterations = c.TryGetProperty("iterations", out JsonElement it) && it.ValueKind == JsonValueKind.Number
                ? it.GetInt64() : _rows;
            if (c.TryGetProperty("setup", out JsonElement setup))
            {
                using var setupCommand = new NpgsqlCommand(
                    (setup.GetString() ?? "").Replace("{rows}", _rows.ToString()), conn);
                setupCommand.ExecuteNonQuery();
            }

            double startedCpu = CpuMs();
            long started = Stopwatch.GetTimestamp();
            string shape = c.GetProperty("shape").GetString() ?? "";
            Dictionary<string, object?> result;
            switch (shape)
            {
                case "stmt_loop":
                    result = StmtLoop(conn, c, _rows, iterations);
                    break;
                case "query_loop":
                    result = QueryLoop(conn, c, _rows, iterations);
                    break;
                case "chunk_insert":
                    result = ChunkInsert(conn, c, _rows, iterations, chunk, inTxn: false);
                    break;
                case "chunk_insert_txn":
                    result = ChunkInsert(conn, c, _rows, iterations, chunk, inTxn: true);
                    break;
                case "batch_update":
                    result = BatchUpdate(conn, c, _rows, iterations, chunk);
                    break;
                case "batch_delete":
                    result = BatchDelete(conn, c, iterations, chunk);
                    break;
                case "txn_loop":
                    result = TxnLoop(conn, c, _rows, iterations);
                    break;
                case "txn_case":
                    result = TxnCase(conn, c, _rows, iterations);
                    break;
                case "app_order_txn":
                    result = AppOrderTxn(conn, iterations);
                    break;
                case "conc":
                    result = Conc(c, _rows, _ops);
                    iterations = _ops * c.GetProperty("workers").GetInt32();
                    break;
                case "conn":
                    result = ConnCase(c, _rows, iterations);
                    break;
                default:
                    throw new InvalidOperationException("unknown shape: " + shape);
            }
            double caseCpu = CpuMs() - startedCpu;
            List<List<string>> checks = RunChecks(conn, c, _rows);

            var record = new Dictionary<string, object?>
            {
                ["language"] = Language,
                ["workload"] = _workload,
                ["case"] = c.GetProperty("id").GetString(),
                ["shape"] = shape,
                ["rows"] = _rows,
                ["iterations"] = iterations,
                ["expect_errors"] = c.TryGetProperty("expect_errors", out JsonElement ee)
                                    && ee.ValueKind == JsonValueKind.True,
                ["checks"] = checks,
                ["cpu_ms"] = Math.Round(caseCpu, 3),
                ["peak_rss_kb"] = PeakRssKb(),
                ["elapsed_ms"] = Math.Round((Stopwatch.GetTimestamp() - started) * 1000.0
                                            / Stopwatch.Frequency, 3),
                ["setup_ms"] = 0.0,
                ["error_sample"] = "",
                ["status"] = "ok",
            };
            foreach (KeyValuePair<string, object?> pair in result)
            {
                record[pair.Key] = pair.Value;
            }
            Console.WriteLine(JsonSerializer.Serialize(record));
            Console.Out.Flush();
        }
        return 0;
    }

    // ---------------------------------------------------------------- arguments
    private static void ParseArgs(string[] argv)
    {
        string host = Env("PGHOST", "postgres");
        string port = Env("PGPORT", "5432");
        string user = Env("PGUSER", "bench");
        string password = Env("PGPASSWORD", "bench");
        string db = Env("PGDATABASE", "kode_bench");

        for (int i = 0; i + 1 < argv.Length; i += 2)
        {
            string key = argv[i].TrimStart('-');
            string value = argv[i + 1];
            switch (key)
            {
                case "workload": _workload = value; break;
                case "cases": _casesPath = value; break;
                case "rows": _rows = long.Parse(value); break;
                case "ops": _ops = long.Parse(value); break;
                case "host": host = value; break;
                case "port": port = value; break;
                case "user": user = value; break;
                case "password": password = value; break;
                case "db": db = value; break;
                case "only": Only.AddRange(value.Split(',')); break;
            }
        }
        string baseString = $"Host={host};Port={port};Username={user};Password={password};"
                            + $"Database={db};Application Name=kode-bench-{Language};";
        // Pooled for steady-state cases, pooling off for cases that measure
        // connection establishment or terminate a backend underneath a client.
        _connectionString = baseString + "Pooling=true;No Reset On Close=true;";
        _directConnectionString = baseString + "Pooling=false;";
    }

    private static string Env(string key, string fallback)
    {
        string? value = Environment.GetEnvironmentVariable(key);
        return string.IsNullOrEmpty(value) ? fallback : value;
    }

    // ---------------------------------------------------------------- utilities
    private static double Percentile(double[] sorted, double p)
    {
        if (sorted.Length == 0)
        {
            return 0;
        }
        double pos = (sorted.Length - 1) * p;
        int lo = (int)Math.Floor(pos);
        int hi = (int)Math.Ceiling(pos);
        if (lo == hi)
        {
            return sorted[lo];
        }
        return sorted[lo] + (sorted[hi] - sorted[lo]) * (pos - lo);
    }

    private static Dictionary<string, object?> Summarise(double[] latencies, double wallMs)
    {
        double[] sorted = (double[])latencies.Clone();
        Array.Sort(sorted);
        int ops = sorted.Length;
        var outMap = new Dictionary<string, object?>
        {
            ["ops"] = ops,
            ["wall_ms"] = Math.Round(wallMs, 3),
            ["throughput"] = wallMs > 0 ? Math.Round(ops / (wallMs / 1000.0), 2) : 0.0,
            ["p50_ms"] = Math.Round(Percentile(sorted, 0.50), 3),
            ["p95_ms"] = Math.Round(Percentile(sorted, 0.95), 3),
            ["p99_ms"] = Math.Round(Percentile(sorted, 0.99), 3),
        };
        if (ops > 0)
        {
            outMap["min_ms"] = Math.Round(sorted[0], 3);
            outMap["max_ms"] = Math.Round(sorted[ops - 1], 3);
        }
        return outMap;
    }

    private static double CpuMs() => Process.GetCurrentProcess().TotalProcessorTime.TotalMilliseconds;

    private static long PeakRssKb()
    {
        try
        {
            foreach (string line in File.ReadLines("/proc/self/status"))
            {
                if (line.StartsWith("VmHWM:", StringComparison.Ordinal))
                {
                    Match match = Regex.Match(line, @"\d+");
                    return match.Success ? long.Parse(match.Value) : 0;
                }
            }
        }
        catch (IOException)
        {
            // Not Linux: report 0 rather than guessing.
        }
        return 0;
    }

    // ------------------------------------------------------------ parameters
    private static object Gen(string name, long i, long rows) => name switch
    {
        "id" => i + 1,
        "k" => (i * 37) % rows,
        "k_hi" => ((i * 37) % rows) + 25,
        "pick_id" => ((i * 7) % rows) + 1,
        "pick_product" => ((i * 7) % 1000) + 1,
        "offset" => (i * 100) % 1000,
        "qty" => (i % 5) + 1,
        "order_id" => ((i * 997) % 100000) + 1,
        "app_product_id" => 100000 + i,
        "app_customer_id" => 100000 + i,
        "app_order_id" => 1000000 + i,
        "worker_id" => i + 1,
        "payload" => "payload-" + i,
        "payload2" => "updated-" + i,
        "tag" => "t" + (i % 10),
        "status" => Statuses[i % 4],
        "price" => "19.99",
        "sku" => "SKU-NEW-" + i,
        "name" => "name-" + i,
        "email" => $"user{i}@example.com",
        "city" => Cities[i % 4],
        "category" => Categories[i % 4],
        "name_like" => "name-%",
        _ => throw new InvalidOperationException("unknown parameter generator: " + name),
    };

    private static object[] ParamsFor(List<string> names, long i, long rows)
    {
        var values = new object[names.Count];
        for (int k = 0; k < names.Count; k++)
        {
            values[k] = Gen(names[k], i, rows);
        }
        return values;
    }

    private static List<object> ChunkValues(List<string> names, long start, long count, long rows)
    {
        var values = new List<object>();
        for (long i = start; i < start + count; i++)
        {
            values.AddRange(ParamsFor(names, i, rows));
        }
        return values;
    }

    private static List<string> Names(JsonElement node, string field)
    {
        var list = new List<string>();
        if (node.TryGetProperty(field, out JsonElement array) && array.ValueKind == JsonValueKind.Array)
        {
            foreach (JsonElement item in array.EnumerateArray())
            {
                list.Add(item.GetString() ?? "");
            }
        }
        return list;
    }

    private static NpgsqlCommand Command(NpgsqlConnection conn, string sql, IReadOnlyList<object> values)
    {
        var command = new NpgsqlCommand(sql, conn);
        for (int i = 0; i < values.Count; i++)
        {
            command.Parameters.Add(new NpgsqlParameter { Value = values[i] });
        }
        return command;
    }

    private static bool IsDeadlock(PostgresException error) => error.SqlState == "40P01";

    private static bool IsUnique(PostgresException error) => error.SqlState == "23505";

    private static string FirstLine(Exception error)
    {
        string text = error.Message;
        int newline = text.IndexOf('\n', StringComparison.Ordinal);
        if (newline >= 0)
        {
            text = text[..newline];
        }
        return text.Length > 200 ? text[..200] : text;
    }

    // ------------------------------------------------------------------ shapes
    private static Dictionary<string, object?> StmtLoop(NpgsqlConnection conn, JsonElement c,
                                                        long rows, long iterations)
    {
        string sql = c.GetProperty("sql").GetString() ?? "";
        List<string> names = Names(c, "params");
        var lat = new double[iterations];
        long affected = 0;
        int errors = 0;
        string sample = "";
        long started = Stopwatch.GetTimestamp();
        using NpgsqlCommand command = Command(conn, sql, ParamsFor(names, 0, rows));
        command.Prepare();
        for (long i = 0; i < iterations; i++)
        {
            object[] values = ParamsFor(names, i, rows);
            for (int k = 0; k < values.Length; k++)
            {
                command.Parameters[k].Value = values[k];
            }
            long t0 = Stopwatch.GetTimestamp();
            try
            {
                affected += Math.Max(command.ExecuteNonQuery(), 0);
            }
            catch (PostgresException error)
            {
                errors++;
                if (sample.Length == 0)
                {
                    sample = FirstLine(error);
                }
            }
            lat[i] = (Stopwatch.GetTimestamp() - t0) * 1000.0 / Stopwatch.Frequency;
        }
        Dictionary<string, object?> result = Summarise(lat,
            (Stopwatch.GetTimestamp() - started) * 1000.0 / Stopwatch.Frequency);
        result["rows_affected"] = affected;
        result["errors"] = errors;
        result["error_sample"] = sample;
        return result;
    }

    private static Dictionary<string, object?> QueryLoop(NpgsqlConnection conn, JsonElement c,
                                                         long rows, long iterations)
    {
        string sql = c.GetProperty("sql").GetString() ?? "";
        List<string> names = Names(c, "params");
        var lat = new double[iterations];
        long fetched = 0;
        int errors = 0;
        string sample = "";
        long started = Stopwatch.GetTimestamp();
        using NpgsqlCommand command = Command(conn, sql, ParamsFor(names, 0, rows));
        command.Prepare();
        for (long i = 0; i < iterations; i++)
        {
            object[] values = ParamsFor(names, i, rows);
            for (int k = 0; k < values.Length; k++)
            {
                command.Parameters[k].Value = values[k];
            }
            long t0 = Stopwatch.GetTimestamp();
            try
            {
                using NpgsqlDataReader reader = command.ExecuteReader();
                int cols = reader.FieldCount;
                while (reader.Read())
                {
                    for (int col = 0; col < cols; col++)
                    {
                        reader.GetValue(col);
                    }
                    fetched++;
                }
            }
            catch (PostgresException error)
            {
                errors++;
                if (sample.Length == 0)
                {
                    sample = FirstLine(error);
                }
            }
            lat[i] = (Stopwatch.GetTimestamp() - t0) * 1000.0 / Stopwatch.Frequency;
        }
        Dictionary<string, object?> result = Summarise(lat,
            (Stopwatch.GetTimestamp() - started) * 1000.0 / Stopwatch.Frequency);
        result["rows_fetched"] = fetched;
        result["errors"] = errors;
        result["error_sample"] = sample;
        return result;
    }

    private static Dictionary<string, object?> ChunkInsert(NpgsqlConnection conn, JsonElement c,
                                                           long rows, long iterations, int chunk, bool inTxn)
    {
        string prefix = c.GetProperty("sql").GetString() ?? "";
        List<string> names = Names(c, "params");
        int width = names.Count;
        int batches = (int)((iterations + chunk - 1) / chunk);
        var lat = new double[batches];
        long affected = 0;
        int errors = 0;
        string sample = "";
        long started = Stopwatch.GetTimestamp();
        int index = 0;
        NpgsqlTransaction? transaction = inTxn ? conn.BeginTransaction() : null;
        try
        {
            for (long s = 0; s < iterations; s += chunk)
            {
                int n = (int)Math.Min(chunk, iterations - s);
                var sql = new System.Text.StringBuilder(prefix);
                for (int r = 0; r < n; r++)
                {
                    if (r > 0)
                    {
                        sql.Append(',');
                    }
                    sql.Append('(');
                    for (int j = 0; j < width; j++)
                    {
                        if (j > 0)
                        {
                            sql.Append(',');
                        }
                        sql.Append('$').Append(r * width + j + 1);
                    }
                    sql.Append(')');
                }
                long t0 = Stopwatch.GetTimestamp();
                try
                {
                    using NpgsqlCommand command = Command(conn, sql.ToString(), ChunkValues(names, s, n, rows));
                    command.Transaction = transaction;
                    affected += Math.Max(command.ExecuteNonQuery(), 0);
                }
                catch (PostgresException error)
                {
                    errors++;
                    if (sample.Length == 0)
                    {
                        sample = FirstLine(error);
                    }
                }
                lat[index++] = (Stopwatch.GetTimestamp() - t0) * 1000.0 / Stopwatch.Frequency;
            }
            if (transaction is not null)
            {
                if (errors == 0)
                {
                    transaction.Commit();
                }
                else
                {
                    transaction.Rollback();
                }
            }
        }
        finally
        {
            transaction?.Dispose();
        }
        Dictionary<string, object?> result = Summarise(lat,
            (Stopwatch.GetTimestamp() - started) * 1000.0 / Stopwatch.Frequency);
        result["rows_affected"] = affected;
        result["errors"] = errors;
        result["error_sample"] = sample;
        result["batches"] = batches;
        return result;
    }

    private static Dictionary<string, object?> BatchUpdate(NpgsqlConnection conn, JsonElement c,
                                                           long rows, long iterations, int chunk)
    {
        string prefix = c.GetProperty("sql").GetString() ?? "";
        List<string> names = Names(c, "params");
        int batches = (int)((iterations + chunk - 1) / chunk);
        var lat = new double[batches];
        long affected = 0;
        int errors = 0;
        string sample = "";
        long started = Stopwatch.GetTimestamp();
        int index = 0;
        for (long s = 0; s < iterations; s += chunk)
        {
            int n = (int)Math.Min(chunk, iterations - s);
            var sql = new System.Text.StringBuilder(prefix);
            for (int r = 0; r < n; r++)
            {
                if (r > 0)
                {
                    sql.Append(',');
                }
                sql.Append("($").Append(r * 2 + 1).Append("::bigint,$").Append(r * 2 + 2).Append("::text)");
            }
            sql.Append(") AS v(id, payload) WHERE bench_writes.id = v.id");
            long t0 = Stopwatch.GetTimestamp();
            try
            {
                using NpgsqlCommand command = Command(conn, sql.ToString(), ChunkValues(names, s, n, rows));
                affected += Math.Max(command.ExecuteNonQuery(), 0);
            }
            catch (PostgresException error)
            {
                errors++;
                if (sample.Length == 0)
                {
                    sample = FirstLine(error);
                }
            }
            lat[index++] = (Stopwatch.GetTimestamp() - t0) * 1000.0 / Stopwatch.Frequency;
        }
        Dictionary<string, object?> result = Summarise(lat,
            (Stopwatch.GetTimestamp() - started) * 1000.0 / Stopwatch.Frequency);
        result["rows_affected"] = affected;
        result["errors"] = errors;
        result["error_sample"] = sample;
        result["batches"] = batches;
        return result;
    }

    private static Dictionary<string, object?> BatchDelete(NpgsqlConnection conn, JsonElement c,
                                                           long iterations, int chunk)
    {
        string sql = c.GetProperty("sql").GetString() ?? "";
        int batches = (int)((iterations + chunk - 1) / chunk);
        var lat = new double[batches];
        long affected = 0;
        int errors = 0;
        string sample = "";
        long started = Stopwatch.GetTimestamp();
        int index = 0;
        using NpgsqlCommand command = new(sql, conn);
        // Prepare resolves parameter types from values, so bind the first range first.
        command.Parameters.Add(new NpgsqlParameter { Value = 1L });
        command.Parameters.Add(new NpgsqlParameter { Value = (long)chunk });
        command.Prepare();
        for (long s = 0; s < iterations; s += chunk)
        {
            int n = (int)Math.Min(chunk, iterations - s);
            command.Parameters[0].Value = s + 1;
            command.Parameters[1].Value = s + n;
            long t0 = Stopwatch.GetTimestamp();
            try
            {
                affected += Math.Max(command.ExecuteNonQuery(), 0);
            }
            catch (PostgresException error)
            {
                errors++;
                if (sample.Length == 0)
                {
                    sample = FirstLine(error);
                }
            }
            lat[index++] = (Stopwatch.GetTimestamp() - t0) * 1000.0 / Stopwatch.Frequency;
        }
        Dictionary<string, object?> result = Summarise(lat,
            (Stopwatch.GetTimestamp() - started) * 1000.0 / Stopwatch.Frequency);
        result["rows_affected"] = affected;
        result["errors"] = errors;
        result["error_sample"] = sample;
        result["batches"] = batches;
        return result;
    }

    private static Dictionary<string, object?> TxnLoop(NpgsqlConnection conn, JsonElement c,
                                                       long rows, long iterations)
    {
        string sql = c.GetProperty("sql").GetString() ?? "";
        List<string> names = Names(c, "params");
        var lat = new double[iterations];
        long affected = 0;
        int errors = 0;
        string sample = "";
        long started = Stopwatch.GetTimestamp();
        using NpgsqlTransaction transaction = conn.BeginTransaction();
        using NpgsqlCommand command = Command(conn, sql, ParamsFor(names, 0, rows));
        command.Transaction = transaction;
        command.Prepare();
        for (long i = 0; i < iterations; i++)
        {
            object[] values = ParamsFor(names, i, rows);
            for (int k = 0; k < values.Length; k++)
            {
                command.Parameters[k].Value = values[k];
            }
            long t0 = Stopwatch.GetTimestamp();
            try
            {
                affected += Math.Max(command.ExecuteNonQuery(), 0);
            }
            catch (PostgresException error)
            {
                errors++;
                if (sample.Length == 0)
                {
                    sample = FirstLine(error);
                }
            }
            lat[i] = (Stopwatch.GetTimestamp() - t0) * 1000.0 / Stopwatch.Frequency;
        }
        if (errors == 0)
        {
            transaction.Commit();
        }
        else
        {
            transaction.Rollback();
        }
        Dictionary<string, object?> result = Summarise(lat,
            (Stopwatch.GetTimestamp() - started) * 1000.0 / Stopwatch.Frequency);
        result["rows_affected"] = affected;
        result["errors"] = errors;
        result["error_sample"] = sample;
        result["transactions"] = 1;
        return result;
    }

    private static object[] RowValues(long i, long rows) => new object[]
    {
        i + 1, (i * 37) % rows, "payload-" + i, "t" + (i % 10),
    };

    private static void InsertRow(NpgsqlCommand command, long i, long rows)
    {
        object[] values = RowValues(i, rows);
        for (int k = 0; k < values.Length; k++)
        {
            command.Parameters[k].Value = values[k];
        }
        command.ExecuteNonQuery();
    }

    private static Dictionary<string, object?> TxnCase(NpgsqlConnection conn, JsonElement c,
                                                       long rows, long iterations)
    {
        string scenario = c.GetProperty("scenario").GetString() ?? "";
        long perTxn = c.TryGetProperty("per_txn", out JsonElement pt) ? pt.GetInt64() : 1;
        var lat = new double[iterations];
        long txns = 0;
        long affected = 0;
        int errors = 0;
        string sample = "";
        long started = Stopwatch.GetTimestamp();
        using NpgsqlCommand command = Command(conn, InsertRowSql, RowValues(0, rows));
        command.Prepare();
        for (long i = 0; i < iterations; i++)
        {
            long t0 = Stopwatch.GetTimestamp();
            using NpgsqlTransaction transaction = conn.BeginTransaction();
            command.Transaction = transaction;
            try
            {
                switch (scenario)
                {
                    case "commit":
                    case "rollback":
                        InsertRow(command, i, rows);
                        if (scenario == "rollback")
                        {
                            transaction.Rollback();
                        }
                        else
                        {
                            transaction.Commit();
                        }
                        txns++;
                        affected++;
                        break;
                    case "multi":
                        for (long j = 0; j < perTxn; j++)
                        {
                            InsertRow(command, i * perTxn + j, rows);
                        }
                        transaction.Commit();
                        txns++;
                        affected++;
                        break;
                    case "constraint":
                    case "recovery":
                        InsertRow(command, i, rows);
                        try
                        {
                            InsertRow(command, i, rows);
                        }
                        catch (PostgresException error)
                        {
                            if (!IsUnique(error))
                            {
                                throw;
                            }
                            errors++;
                            transaction.Rollback();
                            if (scenario == "recovery")
                            {
                                using NpgsqlTransaction second = conn.BeginTransaction();
                                command.Transaction = second;
                                InsertRow(command, 1_000_000 + i, rows);
                                second.Commit();
                                txns++;
                                affected++;
                            }
                            lat[i] = (Stopwatch.GetTimestamp() - t0) * 1000.0 / Stopwatch.Frequency;
                            continue;
                        }
                        transaction.Commit();
                        txns++;
                        affected++;
                        break;
                    default:
                        throw new InvalidOperationException("unknown scenario: " + scenario);
                }
            }
            catch (PostgresException error)
            {
                errors++;
                if (sample.Length == 0)
                {
                    sample = FirstLine(error);
                }
                try
                {
                    transaction.Rollback();
                }
                catch (InvalidOperationException)
                {
                    // The transaction was already rolled back.
                }
            }
            lat[i] = (Stopwatch.GetTimestamp() - t0) * 1000.0 / Stopwatch.Frequency;
        }
        Dictionary<string, object?> result = Summarise(lat,
            (Stopwatch.GetTimestamp() - started) * 1000.0 / Stopwatch.Frequency);
        result["transactions"] = txns;
        result["rows_affected"] = affected;
        result["errors"] = errors;
        result["error_sample"] = sample;
        return result;
    }

    private static Dictionary<string, object?> AppOrderTxn(NpgsqlConnection conn, long iterations)
    {
        var lat = new double[iterations];
        long committed = 0;
        long rolled = 0;
        int errors = 0;
        string sample = "";
        long started = Stopwatch.GetTimestamp();
        const string update = "UPDATE products SET stock = stock - $1 WHERE id = $2 AND stock >= $3";
        const string insert = "INSERT INTO orders (id, customer_id, product_id, qty, unit_price, status) "
                              + "VALUES ($1,$2,$3,$4,$5::text::numeric,$6)";
        using NpgsqlCommand updateCommand = Command(conn, update, new object[] { 1L, 1L, 1L });
        using NpgsqlCommand insertCommand = Command(conn, insert,
            new object[] { 1L, 1L, 1L, 1L, "19.99", "new" });
        for (long i = 0; i < iterations; i++)
        {
            long product = ((i * 7) % 1000) + 1;
            long t0 = Stopwatch.GetTimestamp();
            using NpgsqlTransaction transaction = conn.BeginTransaction();
            updateCommand.Transaction = transaction;
            insertCommand.Transaction = transaction;
            try
            {
                updateCommand.Parameters[0].Value = 1L;
                updateCommand.Parameters[1].Value = product;
                updateCommand.Parameters[2].Value = 1L;
                if (updateCommand.ExecuteNonQuery() == 1)
                {
                    insertCommand.Parameters[0].Value = 2_000_000 + i;
                    insertCommand.Parameters[1].Value = product;
                    insertCommand.Parameters[2].Value = product;
                    insertCommand.Parameters[3].Value = 1L;
                    insertCommand.Parameters[4].Value = "19.99";
                    insertCommand.Parameters[5].Value = "new";
                    insertCommand.ExecuteNonQuery();
                    transaction.Commit();
                    committed++;
                }
                else
                {
                    transaction.Rollback();
                    rolled++;
                }
            }
            catch (PostgresException error)
            {
                errors++;
                if (sample.Length == 0)
                {
                    sample = FirstLine(error);
                }
                transaction.Rollback();
            }
            lat[i] = (Stopwatch.GetTimestamp() - t0) * 1000.0 / Stopwatch.Frequency;
        }
        Dictionary<string, object?> result = Summarise(lat,
            (Stopwatch.GetTimestamp() - started) * 1000.0 / Stopwatch.Frequency);
        result["transactions"] = committed + rolled;
        result["rows_affected"] = committed;
        result["rolled_back"] = rolled;
        result["errors"] = errors;
        result["error_sample"] = sample;
        return result;
    }

    private static NpgsqlDataSource Pooled(int size)
    {
        var builder = new NpgsqlConnectionStringBuilder(_connectionString)
        {
            MaxPoolSize = size,
            MinPoolSize = size,
        };
        return NpgsqlDataSource.Create(builder.ConnectionString);
    }

    private static Dictionary<string, object?> Conc(JsonElement c, long rows, long ops)
    {
        string mode = c.GetProperty("mode").GetString() ?? "read";
        int workers = c.TryGetProperty("workers", out JsonElement workersNode) ? workersNode.GetInt32() : 10;
        var lat = new double[workers * ops];
        int errors = 0;
        int deadlocks = 0;
        string sample = "";

        long setupStart = Stopwatch.GetTimestamp();
        using NpgsqlDataSource dataSource = Pooled(workers);
        var connections = new List<NpgsqlConnection>();
        for (int i = 0; i < workers; i++)
        {
            connections.Add(dataSource.OpenConnection());
        }
        double setupMs = (Stopwatch.GetTimestamp() - setupStart) * 1000.0 / Stopwatch.Frequency;

        long started = Stopwatch.GetTimestamp();
        var tasks = new List<Task>();
        for (int worker = 0; worker < workers; worker++)
        {
            int w = worker;
            NpgsqlConnection conn = connections[w];
            tasks.Add(Task.Run(() =>
            {
                using NpgsqlCommand readCommand = Command(conn,
                    "SELECT id, payload FROM bench_events WHERE id = $1", new object[] { 1L });
                using NpgsqlCommand writeCommand = Command(conn, InsertRowSql,
                    new object[] { 1L, 0L, "", "" });
                for (long j = 0; j < ops; j++)
                {
                    bool write = mode == "write" || (mode == "mixed" && j % 10 >= 7);
                    long t0 = Stopwatch.GetTimestamp();
                    try
                    {
                        if (write)
                        {
                            writeCommand.Parameters[0].Value = w * 1_000_000 + j + 1;
                            writeCommand.Parameters[1].Value = (j * 37) % Math.Max(rows, 1);
                            writeCommand.Parameters[2].Value = "payload-w" + w;
                            writeCommand.Parameters[3].Value = "t" + (j % 10);
                            writeCommand.ExecuteNonQuery();
                        }
                        else
                        {
                            readCommand.Parameters[0].Value = ((((long)w * ops + j) * 7) % rows) + 1;
                            using NpgsqlDataReader reader = readCommand.ExecuteReader();
                            int cols = reader.FieldCount;
                            while (reader.Read())
                            {
                                for (int col = 0; col < cols; col++)
                                {
                                    reader.GetValue(col);
                                }
                            }
                        }
                    }
                    catch (PostgresException error)
                    {
                        Interlocked.Increment(ref errors);
                        if (IsDeadlock(error))
                        {
                            Interlocked.Increment(ref deadlocks);
                        }
                        if (sample.Length == 0)
                        {
                            sample = FirstLine(error);
                        }
                    }
                    lat[w * ops + j] = (Stopwatch.GetTimestamp() - t0) * 1000.0 / Stopwatch.Frequency;
                }
            }));
        }
        Task.WaitAll(tasks.ToArray());
        double wallMs = (Stopwatch.GetTimestamp() - started) * 1000.0 / Stopwatch.Frequency;
        foreach (NpgsqlConnection connection in connections)
        {
            connection.Dispose();
        }

        Dictionary<string, object?> result = Summarise(lat, wallMs);
        result["errors"] = errors;
        result["deadlocks"] = deadlocks;
        result["workers"] = workers;
        result["connections"] = workers;
        result["setup_ms"] = Math.Round(setupMs, 3);
        result["error_sample"] = sample;
        return result;
    }

    private static Dictionary<string, object?> ConnCase(JsonElement c, long rows, long iterations)
    {
        string scenario = c.GetProperty("scenario").GetString() ?? "";
        var lat = new double[iterations];
        int errors = 0;
        string sample = "";
        double setupMs = 0;
        int connections = 1;

        switch (scenario)
        {
            case "connect":
                for (long i = 0; i < iterations; i++)
                {
                    long t0 = Stopwatch.GetTimestamp();
                    try
                    {
                        using var conn = new NpgsqlConnection(_directConnectionString);
                        conn.Open();
                        using var command = new NpgsqlCommand("SELECT 1", conn);
                        command.ExecuteScalar();
                    }
                    catch (NpgsqlException error)
                    {
                        errors++;
                        if (sample.Length == 0)
                        {
                            sample = FirstLine(error);
                        }
                    }
                    lat[i] = (Stopwatch.GetTimestamp() - t0) * 1000.0 / Stopwatch.Frequency;
                }
                break;
            case "reuse":
            {
                long t0 = Stopwatch.GetTimestamp();
                using var dataSource = NpgsqlDataSource.Create(_connectionString);
                using NpgsqlConnection conn = dataSource.OpenConnection();
                setupMs = (Stopwatch.GetTimestamp() - t0) * 1000.0 / Stopwatch.Frequency;
                using NpgsqlCommand command = Command(conn,
                    "SELECT id, payload FROM bench_events WHERE id = $1", new object[] { 1L });
                command.Prepare();
                for (long i = 0; i < iterations; i++)
                {
                    command.Parameters[0].Value = (((i * 7) % rows) + 1);
                    long t = Stopwatch.GetTimestamp();
                    try
                    {
                        using NpgsqlDataReader reader = command.ExecuteReader();
                        reader.Read();
                    }
                    catch (PostgresException error)
                    {
                        errors++;
                        if (sample.Length == 0)
                        {
                            sample = FirstLine(error);
                        }
                    }
                    lat[i] = (Stopwatch.GetTimestamp() - t) * 1000.0 / Stopwatch.Frequency;
                }
                break;
            }
            case "pool":
            {
                long t0 = Stopwatch.GetTimestamp();
                using NpgsqlDataSource dataSource = Pooled(10);
                setupMs = (Stopwatch.GetTimestamp() - t0) * 1000.0 / Stopwatch.Frequency;
                for (long i = 0; i < iterations; i++)
                {
                    long t = Stopwatch.GetTimestamp();
                    try
                    {
                        using NpgsqlConnection conn = dataSource.OpenConnection();
                        using var command = new NpgsqlCommand(
                            "SELECT id, payload FROM bench_events WHERE id = $1", conn);
                        command.Parameters.Add(new NpgsqlParameter
                        {
                            Value = ((i * 7) % rows) + 1,
                        });
                        using NpgsqlDataReader reader = command.ExecuteReader();
                        reader.Read();
                    }
                    catch (PostgresException error)
                    {
                        errors++;
                        if (sample.Length == 0)
                        {
                            sample = FirstLine(error);
                        }
                    }
                    lat[i] = (Stopwatch.GetTimestamp() - t) * 1000.0 / Stopwatch.Frequency;
                }
                connections = 10;
                break;
            }
            case "exhaust":
            {
                long t0 = Stopwatch.GetTimestamp();
                using NpgsqlDataSource dataSource = Pooled(4);
                var warm = new List<NpgsqlConnection>();
                for (int i = 0; i < 4; i++)
                {
                    warm.Add(dataSource.OpenConnection());
                }
                setupMs = (Stopwatch.GetTimestamp() - t0) * 1000.0 / Stopwatch.Frequency;
                foreach (NpgsqlConnection warmConn in warm)
                {
                    warmConn.Dispose();
                }
                int localErrors = 0;
                string localSample = "";
                var tasks = new List<Task>();
                for (int worker = 0; worker < 20; worker++)
                {
                    int w = worker;
                    tasks.Add(Task.Run(() =>
                    {
                        for (long j = 0; j < iterations; j++)
                        {
                            long t = Stopwatch.GetTimestamp();
                            try
                            {
                                using NpgsqlConnection conn = dataSource.OpenConnection();
                                using var command = new NpgsqlCommand(
                                    "SELECT id FROM bench_events WHERE id = $1", conn);
                                command.Parameters.Add(new NpgsqlParameter
                                {
                                    Value = ((w * iterations + j) % rows) + 1,
                                });
                                using NpgsqlDataReader reader = command.ExecuteReader();
                                reader.Read();
                            }
                            catch (PostgresException error)
                            {
                                Interlocked.Increment(ref localErrors);
                                if (localSample.Length == 0)
                                {
                                    localSample = FirstLine(error);
                                }
                            }
                            lat[w * iterations + j] =
                                (Stopwatch.GetTimestamp() - t) * 1000.0 / Stopwatch.Frequency;
                        }
                    }));
                }
                Task.WaitAll(tasks.ToArray());
                errors = localErrors;
                sample = localSample;
                connections = 4;
                break;
            }
            case "recover":
            {
                using var control = new NpgsqlConnection(_directConnectionString);
                control.Open();
                for (long i = 0; i < iterations; i++)
                {
                    long t0 = Stopwatch.GetTimestamp();
                    try
                    {
                        using (var victim = new NpgsqlConnection(_directConnectionString))
                        {
                            victim.Open();
                            long pid;
                            using (var pidCommand = new NpgsqlCommand("SELECT pg_backend_pid()", victim))
                            {
                                pid = (long)(pidCommand.ExecuteScalar() ?? 0L);
                            }
                            using (var kill = new NpgsqlCommand("SELECT pg_terminate_backend($1)", control))
                            {
                                kill.Parameters.Add(new NpgsqlParameter { Value = (int)pid });
                                kill.ExecuteScalar();
                            }
                            try
                            {
                                using var probe = new NpgsqlCommand("SELECT 1", victim);
                                probe.ExecuteScalar();
                            }
                            catch (PostgresException)
                            {
                                errors++; // expected: the backend was terminated
                            }
                        }
                        using var check = new NpgsqlCommand(
                            "SELECT count(*) FROM bench_events WHERE id = 1", control);
                        check.ExecuteScalar();
                    }
                    catch (NpgsqlException error)
                    {
                        errors++;
                        if (sample.Length == 0)
                        {
                            sample = FirstLine(error);
                        }
                    }
                    lat[i] = (Stopwatch.GetTimestamp() - t0) * 1000.0 / Stopwatch.Frequency;
                }
                connections = 2;
                break;
            }
            default:
                throw new InvalidOperationException("unknown connection scenario: " + scenario);
        }

        double wall = lat.Sum();
        Dictionary<string, object?> result = Summarise(lat, wall);
        result["errors"] = errors;
        result["error_sample"] = sample;
        result["setup_ms"] = Math.Round(setupMs, 3);
        result["connections"] = connections;
        result["reconnect_ok"] = scenario == "recover" && errors == iterations;
        return result;
    }

    private static List<List<string>> RunChecks(NpgsqlConnection conn, JsonElement c, long rows)
    {
        var output = new List<List<string>>();
        if (!c.TryGetProperty("checks", out JsonElement checks) || checks.ValueKind != JsonValueKind.Array)
        {
            return output;
        }
        foreach (JsonElement check in checks.EnumerateArray())
        {
            string sql = check.GetProperty("sql").GetString() ?? "";
            List<string> names = Names(check, "params");
            using NpgsqlCommand command = Command(conn, sql, ParamsFor(names, 0, rows));
            using NpgsqlDataReader reader = command.ExecuteReader();
            if (reader.Read())
            {
                var row = new List<string>();
                for (int col = 0; col < reader.FieldCount; col++)
                {
                    object? value = reader.GetValue(col);
                    row.Add(value switch
                    {
                        null or DBNull => "NULL",
                        bool b => b ? "true" : "false",
                        _ => Convert.ToString(value, System.Globalization.CultureInfo.InvariantCulture) ?? "",
                    });
                }
                output.Add(row);
            }
            else
            {
                output.Add(new List<string>());
            }
        }
        return output;
    }
}
