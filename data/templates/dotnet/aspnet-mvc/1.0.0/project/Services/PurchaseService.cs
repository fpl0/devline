using System.Globalization;
using System.Text.RegularExpressions;
using Microsoft.EntityFrameworkCore;
using TaskBoard.Data;
namespace TaskBoard.Services;

public sealed class WorkflowException(int status, string message) : Exception(message)
{
    public int Status { get; } = status;
}
public sealed class PurchaseService(PurchaseDatabase db)
{
    static string Text(string? value, int max, string field) => value is not null && value.Trim().Length is > 0 && value.Trim().Length <= max ? value.Trim() : throw new WorkflowException(400, $"{field} must contain 1–{max} characters");
    public static long Cents(string? amount)
    {
        if (amount is null || !Regex.IsMatch(amount, @"^[0-9]+(\.[0-9]{1,2})?$") || !decimal.TryParse(amount, NumberStyles.AllowDecimalPoint, CultureInfo.InvariantCulture, out var value) || value <= 0 || value > 1000000)
            throw new WorkflowException(400, "Enter an EUR amount from 0.01 to 1,000,000.00 with at most two decimals");
        return decimal.ToInt64(value * 100);
    }
    public async Task<List<Purchase>> List(string status, string q, CancellationToken cancellation)
    {
        if (status is not ("" or "pending" or "approved" or "rejected"))
            throw new WorkflowException(400, "Invalid status");
        var query = db.Purchases.AsNoTracking();
        if (status != "")
            query = query.Where(r => r.Status == status);
        q = q.Trim().ToLowerInvariant();
        if (q != "")
            query = query.Where(r => r.Title.ToLower().Contains(q) || r.RequestedBy.ToLower().Contains(q));
        return await query.OrderByDescending(r => r.Id).ToListAsync(cancellation);
    }
    public async Task<Purchase> Get(long id, CancellationToken cancellation) => await db.Purchases.AsNoTracking().SingleOrDefaultAsync(r => r.Id == id, cancellation) ?? throw new WorkflowException(404, "Request not found");
    public async Task<Purchase> Create(string actor, string? title, string? reason, string? amount, CancellationToken cancellation)
    {
        var purchase = new Purchase { Title = Text(title, 160, "Title"), Reason = Text(reason, 2000, "Reason"), AmountCents = Cents(amount), RequestedBy = actor };
        await using var transaction = await db.Database.BeginTransactionAsync(cancellation);
        db.Purchases.Add(purchase);
        await db.SaveChangesAsync(cancellation);
        db.Events.Add(new()
        {
            PurchaseId = purchase.Id,
            Actor = actor,
            Action = "submitted",
            Note = purchase.Reason
        });
        await db.SaveChangesAsync(cancellation);
        await transaction.CommitAsync(cancellation);
        return purchase;
    }
    public async Task Decide(long id, string actor, bool approver, long version, string? decision, string? note, CancellationToken cancellation)
    {
        if (!approver)
            throw new WorkflowException(403, "Only an approver can record a decision");
        var purchase = await Get(id, cancellation);
        if (purchase.RequestedBy == actor)
            throw new WorkflowException(403, "You cannot approve or reject your own request");
        if (decision is not ("approved" or "rejected"))
            throw new WorkflowException(400, "Choose approve or reject");
        var checkedNote = Text(note, 500, "Decision note");
        await using var transaction = await db.Database.BeginTransactionAsync(cancellation);
        var changed = await db.Purchases.Where(r => r.Id == id && r.Version == version && r.Status == "pending").ExecuteUpdateAsync(u => u.SetProperty(r => r.Status, decision).SetProperty(r => r.Version, r => r.Version + 1).SetProperty(r => r.DecidedBy, actor).SetProperty(r => r.DecisionNote, checkedNote), cancellation);
        if (changed != 1)
            throw new WorkflowException(409, "Request already decided or changed; reload before trying again");
        db.Events.Add(new()
        {
            PurchaseId = id,
            Actor = actor,
            Action = decision,
            Note = checkedNote
        });
        await db.SaveChangesAsync(cancellation);
        await transaction.CommitAsync(cancellation);
    }
    public async Task<List<PurchaseEvent>> Events(long id, CancellationToken cancellation) => await db.Events.AsNoTracking().Where(e => e.PurchaseId == id).OrderBy(e => e.Id).ToListAsync(cancellation);
}
