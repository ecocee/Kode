package bench;

// PostgreSQL benchmark client for Java (JDBC 42.x + HikariCP).
//
// One workload per process, one JSON record per case on stdout. Prepared
// statements are created once per case and reused with new parameter values,
// the standard JDBC pattern. Concurrency uses one HikariCP connection per
// worker, held for the whole case.

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.zaxxer.hikari.HikariConfig;
import com.zaxxer.hikari.HikariDataSource;

import java.io.IOException;
import java.lang.management.ManagementFactory;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.sql.Connection;
import java.sql.DriverManager;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.ResultSetMetaData;
import java.sql.SQLException;
import java.sql.Statement;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Comparator;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicReference;

public final class Client {
    static final String LANGUAGE = "java";
    static final ObjectMapper MAPPER = new ObjectMapper();
    static final String[] STATUSES = {"new", "paid", "shipped", "cancelled"};
    static final String[] CATEGORIES = {"books", "games", "tools", "music"};
    static final String[] CITIES = {"Bengaluru", "Berlin", "Lisbon", "Osaka"};
    static final String INSERT_ROW =
            "INSERT INTO bench_writes (id, k, payload, tag) VALUES ($1,$2,$3,$4)";

    // ---------------------------------------------------------------- arguments
    static String workload = "";
    static String casesPath = "/bench/workloads/cases.json";
    static long datasetRows = 10_000;
    static long opsPerWorker = 200;
    static String url = "";
    static String user = "bench";
    static String password = "bench";
    static List<String> only = new ArrayList<>();

    static void parseArgs(String[] argv) {
        String host = env("PGHOST", "postgres");
        String port = env("PGPORT", "5432");
        String db = env("PGDATABASE", "kode_bench");
        user = env("PGUSER", "bench");
        password = env("PGPASSWORD", "bench");
        for (int i = 0; i + 1 < argv.length; i += 2) {
            String key = argv[i].replaceFirst("^--", "");
            String value = argv[i + 1];
            switch (key) {
                case "workload" -> workload = value;
                case "cases" -> casesPath = value;
                case "rows" -> datasetRows = Long.parseLong(value);
                case "ops" -> opsPerWorker = Long.parseLong(value);
                case "host" -> host = value;
                case "port" -> port = value;
                case "user" -> user = value;
                case "password" -> password = value;
                case "db" -> db = value;
                case "only" -> only = Arrays.asList(value.split(","));
                default -> { }
            }
        }
        url = "jdbc:postgresql://" + host + ":" + port + "/" + db
                + "?ApplicationName=kode-bench-" + LANGUAGE;
    }

    static String env(String key, String fallback) {
        String value = System.getenv(key);
        return value == null || value.isEmpty() ? fallback : value;
    }

    // ---------------------------------------------------------------- utilities
    static double percentile(double[] sorted, double p) {
        if (sorted.length == 0) {
            return 0.0;
        }
        double pos = (sorted.length - 1) * p;
        int lo = (int) Math.floor(pos);
        int hi = (int) Math.ceil(pos);
        if (lo == hi) {
            return sorted[lo];
        }
        return sorted[lo] + (sorted[hi] - sorted[lo]) * (pos - lo);
    }

    static double round(double value, int digits) {
        double factor = Math.pow(10, digits);
        return Math.round(value * factor) / factor;
    }

    static Map<String, Object> summarise(double[] latencies, double wallMs) {
        double[] sorted = latencies.clone();
        Arrays.sort(sorted);
        int ops = sorted.length;
        LinkedHashMap<String, Object> out = new LinkedHashMap<>();
        out.put("ops", ops);
        out.put("wall_ms", round(wallMs, 3));
        out.put("throughput", wallMs > 0 ? round(ops / (wallMs / 1000.0), 2) : 0.0);
        out.put("p50_ms", round(percentile(sorted, 0.50), 3));
        out.put("p95_ms", round(percentile(sorted, 0.95), 3));
        out.put("p99_ms", round(percentile(sorted, 0.99), 3));
        if (ops > 0) {
            out.put("min_ms", round(sorted[0], 3));
            out.put("max_ms", round(sorted[ops - 1], 3));
        }
        return out;
    }

    static double cpuMs() {
        var bean = ManagementFactory.getOperatingSystemMXBean();
        if (bean instanceof com.sun.management.OperatingSystemMXBean os) {
            return os.getProcessCpuTime() / 1_000_000.0;
        }
        return 0.0;
    }

    static long peakRssKb() {
        try {
            for (String line : Files.readAllLines(Path.of("/proc/self/status"))) {
                if (line.startsWith("VmHWM:")) {
                    return Long.parseLong(line.substring(6).trim().split("\\s+")[0]);
                }
            }
        } catch (IOException | RuntimeException ignored) {
            // Not Linux: RSS is reported as 0 rather than guessed.
        }
        return 0;
    }

    // ------------------------------------------------------------ parameters
    static Object gen(String name, long i, long rows) {
        return switch (name) {
            case "id" -> i + 1;
            case "k" -> (i * 37) % rows;
            case "k_hi" -> ((i * 37) % rows) + 25;
            case "pick_id" -> ((i * 7) % rows) + 1;
            case "pick_product" -> ((i * 7) % 1000) + 1;
            case "offset" -> (i * 100) % 1000;
            case "qty" -> (i % 5) + 1;
            case "order_id" -> ((i * 997) % 100000) + 1;
            case "app_product_id", "app_customer_id" -> 100000 + i;
            case "app_order_id" -> 1000000 + i;
            case "worker_id" -> i + 1;
            case "payload" -> "payload-" + i;
            case "payload2" -> "updated-" + i;
            case "tag" -> "t" + (i % 10);
            case "status" -> STATUSES[(int) (i % 4)];
            case "price" -> "19.99";
            case "sku" -> "SKU-NEW-" + i;
            case "name" -> "name-" + i;
            case "email" -> "user" + i + "@example.com";
            case "city" -> CITIES[(int) (i % 4)];
            case "category" -> CATEGORIES[(int) (i % 4)];
            case "name_like" -> "name-%";
            default -> throw new IllegalArgumentException("unknown parameter generator: " + name);
        };
    }

    static Object[] paramsFor(List<String> names, long i, long rows) {
        Object[] out = new Object[names.size()];
        for (int k = 0; k < names.size(); k++) {
            out[k] = gen(names.get(k), i, rows);
        }
        return out;
    }

    static List<Object> chunkValues(List<String> names, long start, long count, long rows) {
        List<Object> out = new ArrayList<>();
        for (long i = start; i < start + count; i++) {
            out.addAll(Arrays.asList(paramsFor(names, i, rows)));
        }
        return out;
    }

    record Compiled(String sql, int[] order) { }

    /** JDBC uses positional `?`; $n may appear out of order, so record the order. */
    static Compiled compile(String sql) {
        StringBuilder out = new StringBuilder();
        List<Integer> order = new ArrayList<>();
        for (int i = 0; i < sql.length(); i++) {
            char ch = sql.charAt(i);
            if (ch == '$' && i + 1 < sql.length() && Character.isDigit(sql.charAt(i + 1))) {
                int j = i + 1;
                while (j < sql.length() && Character.isDigit(sql.charAt(j))) {
                    j++;
                }
                order.add(Integer.parseInt(sql.substring(i + 1, j)) - 1);
                out.append('?');
                i = j - 1;
            } else {
                out.append(ch);
            }
        }
        return new Compiled(out.toString(), order.stream().mapToInt(Integer::intValue).toArray());
    }

    static void bind(PreparedStatement ps, Compiled compiled, Object[] values) throws SQLException {
        for (int k = 0; k < compiled.order().length; k++) {
            Object value = values[compiled.order()[k]];
            if (value instanceof Long v) {
                ps.setLong(k + 1, v);
            } else if (value instanceof Integer v) {
                ps.setInt(k + 1, v);
            } else {
                ps.setString(k + 1, String.valueOf(value));
            }
        }
    }

    static List<String> names(JsonNode node, String field) {
        List<String> out = new ArrayList<>();
        JsonNode array = node.get(field);
        if (array != null && array.isArray()) {
            array.forEach(v -> out.add(v.asText()));
        }
        return out;
    }

    static boolean isDeadlock(SQLException error) {
        return "40P01".equals(error.getSQLState());
    }

    static boolean isUnique(SQLException error) {
        return "23505".equals(error.getSQLState());
    }

    static String firstLine(Throwable error) {
        String text = error.getMessage() == null ? error.toString() : error.getMessage();
        int newline = text.indexOf('\n');
        if (newline >= 0) {
            text = text.substring(0, newline);
        }
        return text.length() > 200 ? text.substring(0, 200) : text;
    }

    // ------------------------------------------------------------------ shapes
    static Map<String, Object> stmtLoop(Connection conn, JsonNode c, long rows, long iterations)
            throws SQLException {
        Compiled compiled = compile(c.get("sql").asText());
        List<String> names = names(c, "params");
        double[] lat = new double[(int) iterations];
        long affected = 0;
        int errors = 0;
        String sample = "";
        try (PreparedStatement ps = conn.prepareStatement(compiled.sql())) {
            long start = System.nanoTime();
            for (int i = 0; i < iterations; i++) {
                bind(ps, compiled, paramsFor(names, i, rows));
                long t0 = System.nanoTime();
                try {
                    affected += Math.max(ps.executeUpdate(), 0);
                } catch (SQLException error) {
                    errors++;
                    if (sample.isEmpty()) {
                        sample = firstLine(error);
                    }
                }
                lat[i] = (System.nanoTime() - t0) / 1_000_000.0;
            }
            Map<String, Object> out = summarise(lat, (System.nanoTime() - start) / 1_000_000.0);
            out.put("rows_affected", affected);
            out.put("errors", errors);
            out.put("error_sample", sample);
            return out;
        }
    }

    static Map<String, Object> queryLoop(Connection conn, JsonNode c, long rows, long iterations)
            throws SQLException {
        Compiled compiled = compile(c.get("sql").asText());
        List<String> names = names(c, "params");
        double[] lat = new double[(int) iterations];
        long fetched = 0;
        int errors = 0;
        String sample = "";
        try (PreparedStatement ps = conn.prepareStatement(compiled.sql())) {
            long start = System.nanoTime();
            for (int i = 0; i < iterations; i++) {
                bind(ps, compiled, paramsFor(names, i, rows));
                long t0 = System.nanoTime();
                try (ResultSet rs = ps.executeQuery()) {
                    ResultSetMetaData md = rs.getMetaData();
                    int cols = md.getColumnCount();
                    while (rs.next()) {
                        for (int col = 1; col <= cols; col++) {
                            rs.getObject(col);
                        }
                        fetched++;
                    }
                } catch (SQLException error) {
                    errors++;
                    if (sample.isEmpty()) {
                        sample = firstLine(error);
                    }
                }
                lat[i] = (System.nanoTime() - t0) / 1_000_000.0;
            }
            Map<String, Object> out = summarise(lat, (System.nanoTime() - start) / 1_000_000.0);
            out.put("rows_fetched", fetched);
            out.put("errors", errors);
            out.put("error_sample", sample);
            return out;
        }
    }

    static Map<String, Object> chunkInsert(Connection conn, JsonNode c, long rows, long iterations,
                                           int chunk, boolean inTxn)
            throws SQLException {
        String prefix = c.get("sql").asText();
        List<String> names = names(c, "params");
        int width = names.size();
        int batches = (int) ((iterations + chunk - 1) / chunk);
        double[] lat = new double[batches];
        long affected = 0;
        int errors = 0;
        String sample = "";
        long start = System.nanoTime();
        int index = 0;
        boolean originalAutoCommit = conn.getAutoCommit();
        try {
            if (inTxn) {
                conn.setAutoCommit(false);
            }
            for (long s = 0; s < iterations; s += chunk) {
                int n = (int) Math.min(chunk, iterations - s);
                StringBuilder sql = new StringBuilder(prefix);
                for (int r = 0; r < n; r++) {
                    if (r > 0) {
                        sql.append(',');
                    }
                    sql.append('(');
                    for (int j = 0; j < width; j++) {
                        if (j > 0) {
                            sql.append(',');
                        }
                        sql.append('$').append(r * width + j + 1);
                    }
                    sql.append(')');
                }
                Compiled compiled = compile(sql.toString());
                long t0 = System.nanoTime();
                try (PreparedStatement ps = conn.prepareStatement(compiled.sql())) {
                    List<Object> values = chunkValues(names, s, n, rows);
                    for (int k = 0; k < compiled.order().length; k++) {
                        Object value = values.get(compiled.order()[k]);
                        if (value instanceof Long v) {
                            ps.setLong(k + 1, v);
                        } else {
                            ps.setString(k + 1, String.valueOf(value));
                        }
                    }
                    affected += Math.max(ps.executeUpdate(), 0);
                } catch (SQLException error) {
                    errors++;
                    if (sample.isEmpty()) {
                        sample = firstLine(error);
                    }
                }
                lat[index++] = (System.nanoTime() - t0) / 1_000_000.0;
            }
            if (inTxn) {
                if (errors == 0) {
                    conn.commit();
                } else {
                    conn.rollback();
                }
            }
        } finally {
            if (inTxn) {
                conn.setAutoCommit(originalAutoCommit);
            }
        }
        Map<String, Object> out = summarise(lat, (System.nanoTime() - start) / 1_000_000.0);
        out.put("rows_affected", affected);
        out.put("errors", errors);
        out.put("error_sample", sample);
        out.put("batches", batches);
        return out;
    }

    static Map<String, Object> batchUpdate(Connection conn, JsonNode c, long rows, long iterations, int chunk)
            throws SQLException {
        String prefix = c.get("sql").asText();
        List<String> names = names(c, "params");
        int batches = (int) ((iterations + chunk - 1) / chunk);
        double[] lat = new double[batches];
        long affected = 0;
        int errors = 0;
        String sample = "";
        long start = System.nanoTime();
        int index = 0;
        for (long s = 0; s < iterations; s += chunk) {
            int n = (int) Math.min(chunk, iterations - s);
            StringBuilder sql = new StringBuilder(prefix);
            for (int r = 0; r < n; r++) {
                if (r > 0) {
                    sql.append(',');
                }
                sql.append("($").append(r * 2 + 1).append("::bigint,$").append(r * 2 + 2)
                        .append("::text)");
            }
            sql.append(") AS v(id, payload) WHERE bench_writes.id = v.id");
            Compiled compiled = compile(sql.toString());
            long t0 = System.nanoTime();
            try (PreparedStatement ps = conn.prepareStatement(compiled.sql())) {
                List<Object> values = chunkValues(names, s, n, rows);
                for (int k = 0; k < compiled.order().length; k++) {
                    Object value = values.get(compiled.order()[k]);
                    if (value instanceof Long v) {
                        ps.setLong(k + 1, v);
                    } else {
                        ps.setString(k + 1, String.valueOf(value));
                    }
                }
                affected += Math.max(ps.executeUpdate(), 0);
            } catch (SQLException error) {
                errors++;
                if (sample.isEmpty()) {
                    sample = firstLine(error);
                }
            }
            lat[index++] = (System.nanoTime() - t0) / 1_000_000.0;
        }
        Map<String, Object> out = summarise(lat, (System.nanoTime() - start) / 1_000_000.0);
        out.put("rows_affected", affected);
        out.put("errors", errors);
        out.put("error_sample", sample);
        out.put("batches", batches);
        return out;
    }

    static Map<String, Object> batchDelete(Connection conn, JsonNode c, long iterations, int chunk)
            throws SQLException {
        Compiled compiled = compile(c.get("sql").asText());
        int batches = (int) ((iterations + chunk - 1) / chunk);
        double[] lat = new double[batches];
        long affected = 0;
        int errors = 0;
        String sample = "";
        long start = System.nanoTime();
        int index = 0;
        try (PreparedStatement ps = conn.prepareStatement(compiled.sql())) {
            for (long s = 0; s < iterations; s += chunk) {
                int n = (int) Math.min(chunk, iterations - s);
                ps.setLong(1, s + 1);
                ps.setLong(2, s + n);
                long t0 = System.nanoTime();
                try {
                    affected += Math.max(ps.executeUpdate(), 0);
                } catch (SQLException error) {
                    errors++;
                    if (sample.isEmpty()) {
                        sample = firstLine(error);
                    }
                }
                lat[index++] = (System.nanoTime() - t0) / 1_000_000.0;
            }
        }
        Map<String, Object> out = summarise(lat, (System.nanoTime() - start) / 1_000_000.0);
        out.put("rows_affected", affected);
        out.put("errors", errors);
        out.put("error_sample", sample);
        out.put("batches", batches);
        return out;
    }

    static Map<String, Object> txnLoop(Connection conn, JsonNode c, long rows, long iterations)
            throws SQLException {
        Compiled compiled = compile(c.get("sql").asText());
        List<String> names = names(c, "params");
        double[] lat = new double[(int) iterations];
        long affected = 0;
        int errors = 0;
        String sample = "";
        boolean originalAutoCommit = conn.getAutoCommit();
        long start = System.nanoTime();
        conn.setAutoCommit(false);
        try (PreparedStatement ps = conn.prepareStatement(compiled.sql())) {
            for (int i = 0; i < iterations; i++) {
                bind(ps, compiled, paramsFor(names, i, rows));
                long t0 = System.nanoTime();
                try {
                    affected += Math.max(ps.executeUpdate(), 0);
                } catch (SQLException error) {
                    errors++;
                    if (sample.isEmpty()) {
                        sample = firstLine(error);
                    }
                }
                lat[i] = (System.nanoTime() - t0) / 1_000_000.0;
            }
            if (errors == 0) {
                conn.commit();
            } else {
                conn.rollback();
            }
        } finally {
            conn.setAutoCommit(originalAutoCommit);
        }
        Map<String, Object> out = summarise(lat, (System.nanoTime() - start) / 1_000_000.0);
        out.put("rows_affected", affected);
        out.put("errors", errors);
        out.put("error_sample", sample);
        out.put("transactions", 1);
        return out;
    }

    static Object[] rowValues(long i, long rows) {
        return new Object[]{i + 1, (i * 37) % rows, "payload-" + i, "t" + (i % 10)};
    }

    static Map<String, Object> txnCase(Connection conn, JsonNode c, long rows, long iterations)
            throws SQLException {
        String scenario = c.get("scenario").asText();
        long perTxn = c.has("per_txn") ? c.get("per_txn").asLong() : 1;
        double[] lat = new double[(int) iterations];
        long txns = 0;
        long affected = 0;
        int errors = 0;
        String sample = "";
        boolean originalAutoCommit = conn.getAutoCommit();
        long start = System.nanoTime();
        conn.setAutoCommit(false);
        try (PreparedStatement ps = conn.prepareStatement(compile(INSERT_ROW).sql())) {
            for (int i = 0; i < iterations; i++) {
                long t0 = System.nanoTime();
                try {
                    switch (scenario) {
                        case "commit", "rollback" -> {
                            insertRow(ps, i, rows);
                            if (scenario.equals("rollback")) {
                                conn.rollback();
                            } else {
                                conn.commit();
                            }
                        }
                        case "multi" -> {
                            for (int j = 0; j < perTxn; j++) {
                                insertRow(ps, i * perTxn + j, rows);
                            }
                            conn.commit();
                        }
                        case "constraint", "recovery" -> {
                            insertRow(ps, i, rows);
                            try {
                                insertRow(ps, i, rows);
                            } catch (SQLException error) {
                                if (!isUnique(error)) {
                                    throw error;
                                }
                                errors++;
                                conn.rollback();
                                if (scenario.equals("recovery")) {
                                    insertRow(ps, 1_000_000L + i, rows);
                                    conn.commit();
                                    txns++;
                                    affected++;
                                }
                                lat[i] = (System.nanoTime() - t0) / 1_000_000.0;
                                continue;
                            }
                            conn.commit();
                        }
                        default -> throw new IllegalStateException("unknown scenario: " + scenario);
                    }
                    txns++;
                    affected++;
                } catch (SQLException error) {
                    errors++;
                    if (sample.isEmpty()) {
                        sample = firstLine(error);
                    }
                    try {
                        conn.rollback();
                    } catch (SQLException ignored) {
                        // The connection may already be aborted.
                    }
                }
                lat[i] = (System.nanoTime() - t0) / 1_000_000.0;
            }
        } finally {
            conn.setAutoCommit(originalAutoCommit);
        }
        Map<String, Object> out = summarise(lat, (System.nanoTime() - start) / 1_000_000.0);
        out.put("transactions", txns);
        out.put("rows_affected", affected);
        out.put("errors", errors);
        out.put("error_sample", sample);
        return out;
    }

    static void insertRow(PreparedStatement ps, long i, long rows) throws SQLException {
        Object[] values = rowValues(i, rows);
        ps.setLong(1, (Long) values[0]);
        ps.setLong(2, (Long) values[1]);
        ps.setString(3, (String) values[2]);
        ps.setString(4, (String) values[3]);
        ps.executeUpdate();
    }

    static Map<String, Object> appOrderTxn(Connection conn, long iterations) throws SQLException {
        double[] lat = new double[(int) iterations];
        long committed = 0;
        long rolled = 0;
        int errors = 0;
        String sample = "";
        boolean originalAutoCommit = conn.getAutoCommit();
        long start = System.nanoTime();
        conn.setAutoCommit(false);
        String update = "UPDATE products SET stock = stock - ? WHERE id = ? AND stock >= ?";
        String insert = "INSERT INTO orders (id, customer_id, product_id, qty, unit_price, status) "
                + "VALUES (?,?,?,?,?::text::numeric,?)";
        try (PreparedStatement upd = conn.prepareStatement(update);
             PreparedStatement ins = conn.prepareStatement(insert)) {
            for (int i = 0; i < iterations; i++) {
                long product = ((i * 7) % 1000) + 1;
                long t0 = System.nanoTime();
                try {
                    upd.setLong(1, 1);
                    upd.setLong(2, product);
                    upd.setLong(3, 1);
                    if (upd.executeUpdate() == 1) {
                        ins.setLong(1, 2_000_000L + i);
                        ins.setLong(2, product);
                        ins.setLong(3, product);
                        ins.setLong(4, 1);
                        ins.setString(5, "19.99");
                        ins.setString(6, "new");
                        ins.executeUpdate();
                        conn.commit();
                        committed++;
                    } else {
                        conn.rollback();
                        rolled++;
                    }
                } catch (SQLException error) {
                    errors++;
                    if (sample.isEmpty()) {
                        sample = firstLine(error);
                    }
                    conn.rollback();
                }
                lat[i] = (System.nanoTime() - t0) / 1_000_000.0;
            }
        } finally {
            conn.setAutoCommit(originalAutoCommit);
        }
        Map<String, Object> out = summarise(lat, (System.nanoTime() - start) / 1_000_000.0);
        out.put("transactions", committed + rolled);
        out.put("rows_affected", committed);
        out.put("rolled_back", rolled);
        out.put("errors", errors);
        out.put("error_sample", sample);
        return out;
    }

    static HikariDataSource pool(int size) {
        HikariConfig config = new HikariConfig();
        config.setJdbcUrl(url);
        config.setUsername(user);
        config.setPassword(password);
        config.setMaximumPoolSize(size);
        config.setMinimumIdle(size);
        config.setConnectionTimeout(60_000);
        config.setPoolName("kode-bench");
        return new HikariDataSource(config);
    }

    static Map<String, Object> conc(JsonNode c, long rows, long ops) throws Exception {
        String mode = c.get("mode").asText();
        int workers = c.has("workers") ? c.get("workers").asInt() : 10;
        double[] lat = new double[(int) (workers * ops)];
        int[] errorsOut = {0};
        int[] deadlocksOut = {0};
        String[] sampleOut = {""};

        long setupStart = System.nanoTime();
        HikariDataSource ds = pool(workers);
        List<Connection> connections = new ArrayList<>();
        for (int w = 0; w < workers; w++) {
            connections.add(ds.getConnection());
        }
        double setupMs = (System.nanoTime() - setupStart) / 1_000_000.0;

        ExecutorService executor = Executors.newFixedThreadPool(workers);
        CountDownLatch ready = new CountDownLatch(workers);
        CountDownLatch go = new CountDownLatch(1);
        long start = System.nanoTime();
        for (int w = 0; w < workers; w++) {
            final int worker = w;
            final Connection conn = connections.get(w);
            executor.submit(() -> {
                try (PreparedStatement read = conn.prepareStatement(compile(
                        "SELECT id, payload FROM bench_events WHERE id = $1").sql());
                     PreparedStatement write = conn.prepareStatement(compile(INSERT_ROW).sql())) {
                    ready.countDown();
                    go.await();
                    for (int j = 0; j < ops; j++) {
                        boolean isWrite = mode.equals("write") || (mode.equals("mixed") && j % 10 >= 7);
                        long t0 = System.nanoTime();
                        try {
                            if (isWrite) {
                                write.setLong(1, (long) worker * 1_000_000 + j + 1);
                                write.setLong(2, (j * 37) % Math.max(rows, 1));
                                write.setString(3, "payload-w" + worker);
                                write.setString(4, "t" + (j % 10));
                                write.executeUpdate();
                            } else {
                                read.setLong(1, ((((long) worker * ops + j) * 7) % rows) + 1);
                                try (ResultSet rs = read.executeQuery()) {
                                    while (rs.next()) {
                                        rs.getObject(1);
                                    }
                                }
                            }
                        } catch (SQLException error) {
                            synchronized (errorsOut) {
                                errorsOut[0]++;
                                if (isDeadlock(error)) {
                                    deadlocksOut[0]++;
                                }
                                if (sampleOut[0].isEmpty()) {
                                    sampleOut[0] = firstLine(error);
                                }
                            }
                        }
                        lat[worker * (int) ops + j] = (System.nanoTime() - t0) / 1_000_000.0;
                    }
                } catch (Exception error) {
                    synchronized (errorsOut) {
                        errorsOut[0]++;
                        if (sampleOut[0].isEmpty()) {
                            sampleOut[0] = firstLine(error);
                        }
                    }
                }
                return null;
            });
        }
        ready.await();
        go.countDown();
        executor.shutdown();
        executor.awaitTermination(10, TimeUnit.MINUTES);
        double wallMs = (System.nanoTime() - start) / 1_000_000.0;
        for (Connection conn : connections) {
            conn.close();
        }
        ds.close();

        Map<String, Object> out = summarise(lat, wallMs);
        out.put("errors", errorsOut[0]);
        out.put("deadlocks", deadlocksOut[0]);
        out.put("workers", workers);
        out.put("connections", workers);
        out.put("setup_ms", round(setupMs, 3));
        out.put("error_sample", sampleOut[0]);
        return out;
    }

    static Map<String, Object> connCase(JsonNode c, long rows, long iterations) throws Exception {
        String scenario = c.get("scenario").asText();
        double[] lat = new double[(int) iterations];
        int errors = 0;
        String sample = "";
        double setupMs = 0;
        int connections = 1;

        switch (scenario) {
            case "connect" -> {
                for (int i = 0; i < iterations; i++) {
                    long t0 = System.nanoTime();
                    try (Connection conn = DriverManager.getConnection(url, user, password);
                         Statement st = conn.createStatement();
                         ResultSet rs = st.executeQuery("SELECT 1")) {
                        rs.next();
                    } catch (SQLException error) {
                        errors++;
                        if (sample.isEmpty()) {
                            sample = firstLine(error);
                        }
                    }
                    lat[i] = (System.nanoTime() - t0) / 1_000_000.0;
                }
            }
            case "reuse" -> {
                long t0 = System.nanoTime();
                try (Connection conn = DriverManager.getConnection(url, user, password)) {
                    setupMs = (System.nanoTime() - t0) / 1_000_000.0;
                    try (PreparedStatement ps = conn.prepareStatement(compile(
                            "SELECT id, payload FROM bench_events WHERE id = $1").sql())) {
                        for (int i = 0; i < iterations; i++) {
                            ps.setLong(1, (((i * 7L) % rows) + 1));
                            long t = System.nanoTime();
                            try (ResultSet rs = ps.executeQuery()) {
                                rs.next();
                            } catch (SQLException error) {
                                errors++;
                                if (sample.isEmpty()) {
                                    sample = firstLine(error);
                                }
                            }
                            lat[i] = (System.nanoTime() - t) / 1_000_000.0;
                        }
                    }
                }
            }
            case "pool" -> {
                long t0 = System.nanoTime();
                try (HikariDataSource ds = pool(10)) {
                    setupMs = (System.nanoTime() - t0) / 1_000_000.0;
                    try (Connection conn = ds.getConnection();
                         PreparedStatement ps = conn.prepareStatement(compile(
                                 "SELECT id, payload FROM bench_events WHERE id = $1").sql())) {
                        for (int i = 0; i < iterations; i++) {
                            ps.setLong(1, (((i * 7L) % rows) + 1));
                            long t = System.nanoTime();
                            try (ResultSet rs = ps.executeQuery()) {
                                rs.next();
                            } catch (SQLException error) {
                                errors++;
                                if (sample.isEmpty()) {
                                    sample = firstLine(error);
                                }
                            }
                            lat[i] = (System.nanoTime() - t) / 1_000_000.0;
                        }
                    }
                }
                connections = 10;
            }
            case "exhaust" -> {
                long t0 = System.nanoTime();
                try (HikariDataSource ds = pool(4)) {
                    List<Connection> warm = new ArrayList<>();
                    for (int i = 0; i < 4; i++) {
                        warm.add(ds.getConnection());
                    }
                    setupMs = (System.nanoTime() - t0) / 1_000_000.0;
                    for (Connection warmConn : warm) {
                        warmConn.close();  // returns it to the pool
                    }
                    AtomicInteger errs = new AtomicInteger();
                    AtomicReference<String> smp = new AtomicReference<>("");
                    ExecutorService executor = Executors.newFixedThreadPool(20);
                    CountDownLatch done = new CountDownLatch(20);
                    for (int w = 0; w < 20; w++) {
                        final int worker = w;
                        executor.submit(() -> {
                            try (Connection conn = ds.getConnection();
                                 PreparedStatement ps = conn.prepareStatement(
                                         "SELECT id FROM bench_events WHERE id = ?")) {
                                for (int j = 0; j < iterations; j++) {
                                    ps.setLong(1, ((worker * iterations + (long) j) % rows) + 1);
                                    long t = System.nanoTime();
                                    try (ResultSet rs = ps.executeQuery()) {
                                        rs.next();
                                    } catch (SQLException error) {
                                        errs.incrementAndGet();
                                        smp.compareAndSet("", firstLine(error));
                                    }
                                    lat[worker * (int) iterations + j] =
                                            (System.nanoTime() - t) / 1_000_000.0;
                                }
                            } catch (SQLException error) {
                                errs.incrementAndGet();
                                smp.compareAndSet("", firstLine(error));
                            } finally {
                                done.countDown();
                            }
                            return null;
                        });
                    }
                    done.await(10, TimeUnit.MINUTES);
                    executor.shutdown();
                    errors = errs.get();
                    sample = smp.get();
                }
                connections = 4;
            }
            case "recover" -> {
                try (Connection control = DriverManager.getConnection(url, user, password)) {
                    for (int i = 0; i < iterations; i++) {
                        long t0 = System.nanoTime();
                        try {
                            try (Connection victim = DriverManager.getConnection(url, user, password)) {
                                long pid;
                                try (Statement st = victim.createStatement();
                                     ResultSet rs = st.executeQuery("SELECT pg_backend_pid()")) {
                                    rs.next();
                                    pid = rs.getLong(1);
                                }
                                try (PreparedStatement ps = control.prepareStatement(
                                        "SELECT pg_terminate_backend(?)")) {
                                    ps.setLong(1, pid);
                                    ps.executeQuery().close();
                                }
                                try (Statement st = victim.createStatement()) {
                                    st.executeQuery("SELECT 1").close();
                                } catch (SQLException expected) {
                                    errors++; // expected: the backend was terminated
                                }
                            }
                            try (Statement st = control.createStatement();
                                 ResultSet rs = st.executeQuery(
                                         "SELECT count(*) FROM bench_events WHERE id = 1")) {
                                rs.next();
                            }
                        } catch (SQLException error) {
                            errors++;
                            if (sample.isEmpty()) {
                                sample = firstLine(error);
                            }
                        }
                        lat[i] = (System.nanoTime() - t0) / 1_000_000.0;
                    }
                }
                connections = 2;
            }
            default -> throw new IllegalArgumentException("unknown connection scenario: " + scenario);
        }

        double wall = 0;
        for (double v : lat) {
            wall += v;
        }
        Map<String, Object> out = summarise(lat, wall);
        out.put("errors", errors);
        out.put("error_sample", sample);
        out.put("setup_ms", round(setupMs, 3));
        out.put("connections", connections);
        out.put("reconnect_ok", scenario.equals("recover") && errors == iterations);
        return out;
    }

    static List<List<String>> runChecks(Connection conn, JsonNode c, long rows) throws SQLException {
        List<List<String>> out = new ArrayList<>();
        JsonNode checks = c.get("checks");
        if (checks == null || !checks.isArray()) {
            return out;
        }
        for (JsonNode check : checks) {
            Compiled compiled = compile(check.get("sql").asText());
            try (PreparedStatement ps = conn.prepareStatement(compiled.sql())) {
                bind(ps, compiled, paramsFor(names(check, "params"), 0, rows));
                try (ResultSet rs = ps.executeQuery()) {
                    ResultSetMetaData md = rs.getMetaData();
                    int cols = md.getColumnCount();
                    if (rs.next()) {
                        List<String> row = new ArrayList<>();
                        for (int col = 1; col <= cols; col++) {
                            Object value = rs.getObject(col);
                            row.add(value == null ? "NULL"
                                    : value instanceof Boolean b ? (b ? "true" : "false")
                                    : String.valueOf(value));
                        }
                        out.add(row);
                    } else {
                        out.add(new ArrayList<>());
                    }
                }
            }
        }
        return out;
    }

    // -------------------------------------------------------------------- main
    public static void main(String[] argv) throws Exception {
        parseArgs(argv);
        JsonNode spec = MAPPER.readTree(Files.readString(Path.of(casesPath), StandardCharsets.UTF_8));
        int chunk = spec.has("chunk") ? spec.get("chunk").asInt() : 500;

        List<JsonNode> cases = new ArrayList<>();
        for (JsonNode node : spec.get("cases")) {
            if (!node.get("workload").asText().equals(workload)) {
                continue;
            }
            if (!only.isEmpty() && !only.contains(node.get("id").asText())) {
                continue;
            }
            cases.add(node);
        }
        if (cases.isEmpty()) {
            System.out.println(MAPPER.writeValueAsString(Map.of(
                    "language", LANGUAGE, "workload", workload, "error", "no cases")));
            System.exit(2);
        }

        try (Connection conn = DriverManager.getConnection(url, user, password)) {
            for (JsonNode c : cases) {
                // Concurrency cases carry no iteration count; they recompute below.
                long iterations = c.has("iterations")
                        ? (c.get("iterations").isNumber() ? c.get("iterations").asLong() : datasetRows)
                        : datasetRows;
                if (c.has("setup")) {
                    try (Statement st = conn.createStatement()) {
                        st.execute(c.get("setup").asText()
                                .replace("{rows}", Long.toString(datasetRows)));
                    }
                }
                double startedCpu = cpuMs();
                long t0 = System.nanoTime();
                String shape = c.get("shape").asText();
                Map<String, Object> result;
                switch (shape) {
                    case "stmt_loop" -> result = stmtLoop(conn, c, datasetRows, iterations);
                    case "query_loop" -> result = queryLoop(conn, c, datasetRows, iterations);
                    case "chunk_insert" -> result = chunkInsert(conn, c, datasetRows, iterations, chunk, false);
                    case "chunk_insert_txn" -> result = chunkInsert(conn, c, datasetRows, iterations, chunk, true);
                    case "batch_update" -> result = batchUpdate(conn, c, datasetRows, iterations, chunk);
                    case "batch_delete" -> result = batchDelete(conn, c, iterations, chunk);
                    case "txn_loop" -> result = txnLoop(conn, c, datasetRows, iterations);
                    case "txn_case" -> result = txnCase(conn, c, datasetRows, iterations);
                    case "app_order_txn" -> result = appOrderTxn(conn, iterations);
                    case "conc" -> {
                        result = conc(c, datasetRows, opsPerWorker);
                        iterations = opsPerWorker * c.get("workers").asInt();
                    }
                    case "conn" -> result = connCase(c, datasetRows, iterations);
                    default -> throw new IllegalStateException("unknown shape: " + shape);
                }
                double caseCpu = cpuMs() - startedCpu;
                List<List<String>> checks = runChecks(conn, c, datasetRows);

                LinkedHashMap<String, Object> record = new LinkedHashMap<>();
                record.put("language", LANGUAGE);
                record.put("workload", workload);
                record.put("case", c.get("id").asText());
                record.put("shape", shape);
                record.put("rows", datasetRows);
                record.put("iterations", iterations);
                record.put("expect_errors", c.has("expect_errors") && c.get("expect_errors").asBoolean());
                record.put("checks", checks);
                record.put("cpu_ms", round(caseCpu, 3));
                record.put("peak_rss_kb", peakRssKb());
                record.put("elapsed_ms", round((System.nanoTime() - t0) / 1_000_000.0, 3));
                record.put("status", "ok");
                record.putIfAbsent("setup_ms", 0.0);
                record.putIfAbsent("error_sample", "");
                record.putAll(result);
                System.out.println(MAPPER.writeValueAsString(record));
            }
        }
    }

    private Client() { }
}
