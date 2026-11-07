using Microsoft.EntityFrameworkCore;
using OfflineApi;

var builder = WebApplication.CreateBuilder(args);
var postgres = builder.Configuration.GetConnectionString("Postgres");
var rabbit = builder.Configuration.GetConnectionString("RabbitMQ");
var queue = new QueueOptions(rabbit, builder.Configuration["RabbitMQ:Queue"] ?? "offline-api.notes");
builder.Services.AddSingleton(queue);
builder.Services.AddDbContext<NotesDb>(options =>
{
    if (string.IsNullOrWhiteSpace(postgres))
        options.UseSqlite(builder.Configuration.GetConnectionString("Sqlite") ?? "Data Source=notes.db");
    else
        options.UseNpgsql(postgres);
});
builder.Services.AddScoped<NoteService>();
builder.Services.AddSingleton<NoteDelivery>();
builder.Services.AddHostedService(services => services.GetRequiredService<NoteDelivery>());
var app = builder.Build();
using (var scope = app.Services.CreateScope())
    await scope.ServiceProvider.GetRequiredService<NotesDb>().Database.EnsureCreatedAsync();
app.UseDefaultFiles();
app.UseStaticFiles();
app.MapGet("/api/notes", async (NotesDb database, CancellationToken cancellation) =>
    await database.Notes.OrderByDescending(note => note.CreatedAtUtc).Take(100).ToListAsync(cancellation));
app.MapPost("/api/notes", async (NewNote input, NoteService notes, CancellationToken cancellation) =>
{
    if (!NoteService.ValidText(input.Text))
        return Results.ValidationProblem(new Dictionary<string, string[]> { ["text"] = ["Write between 1 and 1,000 characters."] });
    var note = await notes.AddAsync(input.Text!, queue.Enabled, cancellation);
    return Results.Created($"/api/notes/{note.Id}", note);
});
app.MapGet("/api/notes/{id:guid}", async (Guid id, NotesDb database, CancellationToken cancellation) =>
    await database.Notes.FindAsync([id], cancellation) is { } note ? Results.Ok(note) : Results.NotFound());
app.MapGet("/health/ready", async (NotesDb database, NoteDelivery delivery, CancellationToken cancellation) =>
{
    var databaseReady = await database.Database.CanConnectAsync(cancellation);
    var queueReady = !queue.Enabled || await delivery.ProbeAsync(cancellation);
    return Results.Json(new
    {
        ready = databaseReady && queueReady,
        database = new { provider = string.IsNullOrWhiteSpace(postgres) ? "sqlite" : "postgresql", ready = databaseReady },
        queue = new { configured = queue.Enabled, ready = queueReady, state = delivery.State }
    }, statusCode: databaseReady && queueReady ? 200 : 503);
});
await app.RunAsync();

public sealed record NewNote(string? Text);
