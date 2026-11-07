// HTTP benchmark server, JDK standard library only (com.sun.net.httpserver).
import com.sun.net.httpserver.HttpExchange;
import com.sun.net.httpserver.HttpServer;

import java.io.IOException;
import java.io.OutputStream;
import java.net.InetSocketAddress;
import java.nio.charset.StandardCharsets;
import java.util.concurrent.Executors;

public class Server {
    private static final byte[] HELLO = "hello".getBytes(StandardCharsets.UTF_8);
    private static final byte[] JSON =
            "{\"message\":\"hello\",\"ok\":true,\"n\":1}".getBytes(StandardCharsets.UTF_8);

    public static void main(String[] args) throws Exception {
        int port = args.length > 0 ? Integer.parseInt(args[0].replaceFirst("^:", "")) : 8080;

        String env = System.getenv("THREADS");
        int threads = (env != null && !env.isBlank())
                ? Integer.parseInt(env.trim())
                : Math.max(1, Runtime.getRuntime().availableProcessors());

        HttpServer server = HttpServer.create(new InetSocketAddress(port), 4096);
        server.createContext("/hello", ex -> send(ex, "text/plain", HELLO));
        server.createContext("/json", ex -> send(ex, "application/json", JSON));
        server.setExecutor(Executors.newFixedThreadPool(threads));
        server.start();

        System.out.println("listening on " + port + " threads=" + threads);
    }

    private static void send(HttpExchange ex, String type, byte[] body) throws IOException {
        ex.getResponseHeaders().set("Content-Type", type);
        ex.sendResponseHeaders(200, body.length);
        try (OutputStream os = ex.getResponseBody()) {
            os.write(body);
        }
    }
}