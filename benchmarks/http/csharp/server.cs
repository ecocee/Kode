// HTTP benchmark server: ASP.NET Core (Kestrel) minimal API.
var port = args.Length > 0 ? int.Parse(args[0].TrimStart(':')) : 8080;

var threads = int.TryParse(Environment.GetEnvironmentVariable("THREADS"), out var t) && t > 0
    ? t
    : Environment.ProcessorCount;

// Min threads = THREADS; DOTNET_PROCESSOR_COUNT (set by run.sh) bounds real parallelism.
ThreadPool.SetMinThreads(threads, threads);

var builder = WebApplication.CreateSlimBuilder();
builder.Logging.ClearProviders();
builder.WebHost.ConfigureKestrel(o =>
{
    o.ListenAnyIP(port);
    o.AddServerHeader = false;
});

var app = builder.Build();

const string Json = "{\"message\":\"hello\",\"ok\":true,\"n\":1}";

app.MapGet("/hello", () => Results.Text("hello", "text/plain"));
app.MapGet("/json", () => Results.Text(Json, "application/json"));

Console.WriteLine($"listening on {port} threads={threads}");
app.Run();