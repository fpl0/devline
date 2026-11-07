using Microsoft.EntityFrameworkCore;

namespace OfflineApi;

public sealed class Note
{
    public Guid Id { get; set; } = Guid.NewGuid();
    public string Text { get; set; } = "";
    public DateTime CreatedAtUtc { get; set; } = DateTime.UtcNow;
    public bool QueueRequested { get; set; }
    public DateTime? PublishedAtUtc { get; set; }
    public DateTime? ProcessedAtUtc { get; set; }
    public string Delivery => !QueueRequested ? "saved locally" : ProcessedAtUtc.HasValue ? "processed" : PublishedAtUtc.HasValue ? "queued" : "waiting for queue";
}

public sealed class NotesDb(DbContextOptions<NotesDb> options) : DbContext(options)
{
    public DbSet<Note> Notes => Set<Note>();
    protected override void OnModelCreating(ModelBuilder model)
    {
        model.Entity<Note>().ToTable("OfflineNotes");
        model.Entity<Note>().Property(note => note.Text).HasMaxLength(1000);
        model.Entity<Note>().Ignore(note => note.Delivery);
        model.Entity<Note>().HasIndex(note => new { note.QueueRequested, note.PublishedAtUtc });
    }
}

public sealed class NoteService(NotesDb database)
{
    public static bool ValidText(string? text) => !string.IsNullOrWhiteSpace(text) && text.Trim().Length <= 1000;

    public async Task<Note> AddAsync(string text, bool useQueue, CancellationToken cancellation = default)
    {
        if (!ValidText(text)) throw new ArgumentException("Write between 1 and 1,000 characters.", nameof(text));
        var note = new Note { Text = text.Trim(), QueueRequested = useQueue };
        database.Notes.Add(note);
        // The note itself is a durable outbox record. A broker outage never loses the write.
        await database.SaveChangesAsync(cancellation);
        return note;
    }

    public Task<int> MarkProcessedAsync(Guid id, CancellationToken cancellation = default) =>
        database.Notes.Where(note => note.Id == id && note.ProcessedAtUtc == null)
            .ExecuteUpdateAsync(update => update.SetProperty(note => note.ProcessedAtUtc, DateTime.UtcNow), cancellation);
}

public sealed record QueueOptions(string? ConnectionString, string Name)
{
    public bool Enabled => !string.IsNullOrWhiteSpace(ConnectionString);
}
