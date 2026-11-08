using Microsoft.EntityFrameworkCore;

namespace TaskBoard.Data;

public sealed class TaskDatabase(DbContextOptions<TaskDatabase> options) : DbContext(options)
{
    public DbSet<BoardTask> Tasks => Set<BoardTask>();

    protected override void OnModelCreating(ModelBuilder builder)
    {
        var task = builder.Entity<BoardTask>();
        task.ToTable("tasks", "task_board");
        task.HasKey(row => row.Id);
        task.HasIndex(row => row.Owner);
        task.Property(row => row.Id).HasColumnName("id");
        task.Property(row => row.Owner).HasColumnName("owner").HasMaxLength(255);
        task.Property(row => row.Title).HasColumnName("title").HasMaxLength(160);
        task.Property(row => row.CreatedAt).HasColumnName("created_at");
        task.Property(row => row.Completed).HasColumnName("completed");
        task.Property(row => row.PublishedAt).HasColumnName("published_at");
        task.Property(row => row.DeliveredAt).HasColumnName("delivered_at");
    }
}

public sealed class BoardTask
{
    public Guid Id { get; init; } = Guid.NewGuid();
    public required string Owner { get; init; }
    public required string Title { get; init; }
    public DateTime CreatedAt { get; init; } = DateTime.UtcNow;
    public bool Completed { get; set; }
    public DateTime? PublishedAt { get; set; }
    public DateTime? DeliveredAt { get; set; }
}
