var builder = WebApplication.CreateBuilder(args);
builder.Logging.ClearProviders();

var app = builder.Build();

app.MapGet("/json", () => Results.Json(new { message = "Hello, World!" }));
app.MapGet("/plaintext", () => "Hello, World!");

var port = Environment.GetEnvironmentVariable("PORT") ?? "8092";
Console.WriteLine($"ASP.NET Core benchmark server starting on port {port}");

app.Run($"http://0.0.0.0:{port}");

