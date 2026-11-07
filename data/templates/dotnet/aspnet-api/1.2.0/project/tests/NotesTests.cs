using Microsoft.Data.Sqlite;
using Microsoft.EntityFrameworkCore;
using OfflineApi;
using Xunit;

public sealed class NotesTests
{
    [Fact]
    public async Task QueuedNoteIsDurableBeforeDeliveryAndConsumerIsIdempotent()
    {
        using var connection = new SqliteConnection("Data Source=:memory:");
        await connection.OpenAsync(TestContext.Current.CancellationToken);
        var options = new DbContextOptionsBuilder<NotesDb>().UseSqlite(connection).Options;
        Guid id;
        await using (var database = new NotesDb(options))
        {
            await database.Database.EnsureCreatedAsync(TestContext.Current.CancellationToken);
            var note = await new NoteService(database).AddAsync("  A real saved note  ", true, TestContext.Current.CancellationToken);
            id = note.Id;
            Assert.Equal("waiting for queue", note.Delivery);
        }
        // A new context sees the outbox after the request context has gone away.
        await using (var database = new NotesDb(options))
        {
            var persisted = await database.Notes.AsNoTracking().SingleAsync(TestContext.Current.CancellationToken);
            Assert.Equal("A real saved note", persisted.Text);
            Assert.True(persisted.QueueRequested);
            Assert.Null(persisted.PublishedAtUtc);
            var service = new NoteService(database);
            Assert.Equal(1, await service.MarkProcessedAsync(id, TestContext.Current.CancellationToken));
            var delivered = await database.Notes.AsNoTracking().SingleAsync(TestContext.Current.CancellationToken);
            Assert.Equal("processed", delivered.Delivery);
            Assert.Equal(0, await service.MarkProcessedAsync(id, TestContext.Current.CancellationToken));
            Assert.Equal(delivered.ProcessedAtUtc, (await database.Notes.AsNoTracking().SingleAsync(TestContext.Current.CancellationToken)).ProcessedAtUtc);
            Assert.Equal(0, await service.MarkProcessedAsync(Guid.NewGuid(), TestContext.Current.CancellationToken));
        }
    }

    [Fact]
    public async Task StandaloneNoteDoesNotPretendToUseAQueue()
    {
        using var connection = new SqliteConnection("Data Source=:memory:");
        await connection.OpenAsync(TestContext.Current.CancellationToken);
        await using var database = new NotesDb(new DbContextOptionsBuilder<NotesDb>().UseSqlite(connection).Options);
        await database.Database.EnsureCreatedAsync(TestContext.Current.CancellationToken);
        var service = new NoteService(database);
        var note = await service.AddAsync("Local only", false, TestContext.Current.CancellationToken);
        Assert.Equal("saved locally", note.Delivery);
        Assert.False(note.QueueRequested);
        Assert.Null(note.ProcessedAtUtc);
        await Assert.ThrowsAsync<ArgumentException>(() => service.AddAsync("   ", false, TestContext.Current.CancellationToken));
        await Assert.ThrowsAsync<ArgumentException>(() => service.AddAsync(new string('a', 1001), false, TestContext.Current.CancellationToken));
        Assert.Equal(1, await database.Notes.CountAsync(TestContext.Current.CancellationToken));
    }

    [Fact]
    public void PostgreSqlProviderIsAvailableOffline()
    {
        using var database = new NotesDb(new DbContextOptionsBuilder<NotesDb>()
            .UseNpgsql("Host=127.0.0.1;Database=devline;Username=devline;Password=example").Options);
        Assert.Equal("Npgsql.EntityFrameworkCore.PostgreSQL", database.Database.ProviderName);
    }
}
