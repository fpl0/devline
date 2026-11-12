using Microsoft.EntityFrameworkCore;
namespace TaskBoard.Data;

public sealed class PurchaseDatabase(DbContextOptions<PurchaseDatabase> options) : DbContext(options)
{
    public DbSet<Purchase> Purchases => Set<Purchase>(); public DbSet<PurchaseEvent> Events => Set<PurchaseEvent>();
    protected override void OnModelCreating(ModelBuilder model)
    {
        model.Entity<Purchase>().Property(p => p.Version).IsConcurrencyToken();
        model.Entity<PurchaseEvent>().HasOne<Purchase>().WithMany().HasForeignKey(e => e.PurchaseId);
    }
}
public sealed class Purchase
{
    public long Id
    {
        get; set;
    }
    public required string Title
    {
        get; set;
    }
    public required string Reason
    {
        get; set;
    }
    public long AmountCents
    {
        get; set;
    }
    public required string RequestedBy
    {
        get; set;
    }
    public string Status { get; set; } = "pending"; public long Version { get; set; } = 1;
    public string? DecidedBy
    {
        get; set;
    }
    public string? DecisionNote
    {
        get; set;
    }
    public string CreatedAt { get; set; } = DateTimeOffset.UtcNow.ToString("O");
    public string Amount => (AmountCents / 100m).ToString("F2", System.Globalization.CultureInfo.InvariantCulture);
}
public sealed class PurchaseEvent
{
    public long Id
    {
        get; set;
    }
    public long PurchaseId
    {
        get; set;
    }
    public required string Actor
    {
        get; set;
    }
    public required string Action
    {
        get; set;
    }
    public required string Note
    {
        get; set;
    }
    public string HappenedAt { get; set; } = DateTimeOffset.UtcNow.ToString("O");
}
