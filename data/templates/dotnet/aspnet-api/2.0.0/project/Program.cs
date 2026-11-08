using Microsoft.EntityFrameworkCore;
using TaskApi;

var builder = WebApplication.CreateBuilder(args);
builder.Services.AddControllers();
builder.Services.AddDbContext<TaskDatabase>(options => options.UseSqlite(
    builder.Configuration.GetConnectionString("Tasks") ?? "Data Source=tasks.db"));

var app = builder.Build();
await using (var scope = app.Services.CreateAsyncScope())
    await scope.ServiceProvider.GetRequiredService<TaskDatabase>().Database.EnsureCreatedAsync();

app.MapControllers();
app.MapGet("/", () => Results.Ok(new { name = "ASP.NET Core Task API", tasks = "/api/tasks", health = "/health/ready" }));
app.MapGet("/health/ready", async (TaskDatabase database, CancellationToken cancellation) =>
    await database.Database.CanConnectAsync(cancellation)
        ? Results.Ok(new { status = "ready", database = "sqlite" })
        : Results.StatusCode(StatusCodes.Status503ServiceUnavailable));
await app.RunAsync();
