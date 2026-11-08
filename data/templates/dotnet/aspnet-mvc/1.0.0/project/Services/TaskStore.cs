using System.Text.Json;
using Microsoft.EntityFrameworkCore;
using StackExchange.Redis;
using TaskBoard.Data;
using TaskBoard.Models;

namespace TaskBoard.Services;

public sealed class TaskStore(
    TaskDatabase database,
    IConnectionMultiplexer redis,
    ILogger<TaskStore> logger)
{
    public async Task<(TaskRow[] Rows, string Source)> ListAsync(string owner, CancellationToken cancellation)
    {
        var cache = redis.GetDatabase();
        var key = CacheKey(owner);
        try
        {
            var value = await cache.StringGetAsync(key);
            if (value.HasValue && JsonSerializer.Deserialize<TaskRow[]>(value.ToString()) is { } rows)
                return (rows, "redis");
        }
        catch (RedisException)
        {
            logger.LogWarning("Redis unavailable. Reading tasks from PostgreSQL.");
        }
        catch (JsonException)
        {
            logger.LogWarning("Discarding an unreadable cached task list.");
        }

        var current = await database.Tasks.AsNoTracking().Where(row => row.Owner == owner)
            .OrderByDescending(row => row.CreatedAt).ThenBy(row => row.Id)
            .Select(row => new TaskRow(row.Id, row.Title, row.Completed, row.DeliveredAt))
            .ToArrayAsync(cancellation);

        // Pending deliveries change asynchronously. Read through to PostgreSQL until they settle.
        if (current.All(row => row.DeliveredAt is not null))
        {
            try
            {
                await cache.StringSetAsync(key, JsonSerializer.Serialize(current), TimeSpan.FromSeconds(20));
            }
            catch (RedisException)
            {
                logger.LogWarning("Redis unavailable. Skipping task list cache write.");
            }
        }

        return (current, "postgres");
    }

    public async Task CreateAsync(string owner, string title, CancellationToken cancellation)
    {
        // A newly committed task is also a pending TaskCreated event. The worker picks it up.
        database.Tasks.Add(new BoardTask { Owner = owner, Title = title.Trim() });
        await database.SaveChangesAsync(cancellation);
        await InvalidateAsync(owner);
    }

    public async Task<bool> CompleteAsync(string owner, Guid id, CancellationToken cancellation)
    {
        var count = await database.Tasks.Where(row => row.Owner == owner && row.Id == id)
            .ExecuteUpdateAsync(update => update.SetProperty(row => row.Completed, true), cancellation);
        if (count > 0)
            await InvalidateAsync(owner);
        return count > 0;
    }

    public async Task<bool> DeleteAsync(string owner, Guid id, CancellationToken cancellation)
    {
        var count = await database.Tasks.Where(row => row.Owner == owner && row.Id == id)
            .ExecuteDeleteAsync(cancellation);
        if (count > 0)
            await InvalidateAsync(owner);
        return count > 0;
    }

    public async Task MarkDeliveredAsync(Guid id, CancellationToken cancellation)
    {
        var row = await database.Tasks.FindAsync([id], cancellation);
        if (row is null || row.DeliveredAt is not null)
            return;
        // Only write this column, so a concurrent completion remains intact.
        var count = await database.Tasks.Where(candidate => candidate.Id == id && candidate.DeliveredAt == null)
            .ExecuteUpdateAsync(update => update.SetProperty(candidate => candidate.DeliveredAt, DateTime.UtcNow),
                cancellation);
        if (count > 0)
            await InvalidateAsync(row.Owner);
    }

    private async Task InvalidateAsync(string owner)
    {
        try
        {
            await redis.GetDatabase().KeyDeleteAsync(CacheKey(owner));
        }
        catch (RedisException)
        {
            logger.LogWarning("Redis unavailable. The cached task list will expire.");
        }
    }

    private static string CacheKey(string owner) => $"devline-dotnet:tasks:{owner}";
}
