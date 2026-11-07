using Microsoft.Data.Sqlite;
using Microsoft.EntityFrameworkCore;

if (args.Contains("--smoke-test"))
{
    using var connection = new SqliteConnection("Data Source=:memory:");
    connection.Open();
    var options = new DbContextOptionsBuilder<NotesDb>().UseSqlite(connection).Options;
    using var notes = new NotesDb(options);
    notes.Database.EnsureCreated();
    notes.Add(new Note(0, "offline"));
    notes.SaveChanges();
    if (notes.Notes.Single().Text != "offline")
        throw new InvalidOperationException("EF Core SQLite check failed.");
    Console.WriteLine("EF Core SQLite check passed.");
    return;
}

var builder = WebApplication.CreateBuilder(args);
builder.Services.AddDbContext<NotesDb>(options => options.UseSqlite("Data Source=notes.db"));
var app = builder.Build();

using (var scope = app.Services.CreateScope())
    scope.ServiceProvider.GetRequiredService<NotesDb>().Database.EnsureCreated();

app.MapGet("/", () => Results.Ok(new { status = "offline" }));
app.MapGet("/notes", (NotesDb database) => database.Notes.ToListAsync());
app.Run();

sealed record Note(int Id, string Text);

sealed class NotesDb(DbContextOptions<NotesDb> options) : DbContext(options)
{
    public DbSet<Note> Notes => Set<Note>();
}
