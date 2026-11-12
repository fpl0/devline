using Microsoft.EntityFrameworkCore;
namespace TaskApi;

public sealed class TaskDatabase(DbContextOptions<TaskDatabase> options) : DbContext(options)
{
    public DbSet<Issue> Issues => Set<Issue>();
    public DbSet<IssueEvent> Events => Set<IssueEvent>();
    protected override void OnModelCreating(ModelBuilder model)
    {
        model.Entity<Issue>().Property(i => i.Version).IsConcurrencyToken();
        model.Entity<Issue>().HasIndex(i => new { i.Status, i.Id });
        model.Entity<IssueEvent>().HasKey(e => new { e.IssueId, e.Version });
        model.Entity<IssueEvent>().HasOne<Issue>().WithMany().HasForeignKey(e => e.IssueId);
    }
}
public sealed class Issue
{
    public long Id
    {
        get; set;
    }
    public required string Title
    {
        get; set;
    }
    public string Description { get; set; } = "";
    public required string Priority
    {
        get; set;
    }
    public string Status { get; set; } = "open";
    public long Version { get; set; } = 1;
    public string CreatedAt { get; set; } = DateTimeOffset.UtcNow.ToString("O");
    public string UpdatedAt { get; set; } = DateTimeOffset.UtcNow.ToString("O");
}
public sealed class IssueEvent
{
    public long IssueId
    {
        get; set;
    }
    public long Version
    {
        get; set;
    }
    public required string Action
    {
        get; set;
    }
    public string HappenedAt { get; set; } = DateTimeOffset.UtcNow.ToString("O");
}
