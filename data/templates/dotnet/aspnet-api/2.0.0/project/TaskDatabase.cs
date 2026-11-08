using Microsoft.EntityFrameworkCore;

namespace TaskApi;

public sealed class TaskDatabase(DbContextOptions<TaskDatabase> options) : DbContext(options)
{
    public DbSet<TaskItem> Tasks => Set<TaskItem>();
}

public sealed class TaskItem
{
    public Guid Id { get; init; } = Guid.NewGuid();
    public required string Title { get; set; }
    public bool Completed { get; set; }
}
