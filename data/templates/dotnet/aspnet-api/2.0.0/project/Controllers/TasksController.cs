using System.ComponentModel.DataAnnotations;
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;
namespace TaskApi.Controllers;

[ApiController, Route("api/issues")]
public sealed class IssuesController(TaskDatabase database) : ControllerBase
{
    [HttpGet]
    public async Task<IActionResult> List(string? status = null, string q = "", int limit = 50, int offset = 0, CancellationToken cancellation = default)
    {
        if (status is not (null or "open" or "in_progress" or "resolved") || limit is < 1 or > 100 || offset < 0 || q.Length > 120)
            return Problem(statusCode: 400, detail: "Invalid status, search or pagination; limit must be 1–100 and offset nonnegative");
        var query = database.Issues.AsNoTracking();
        if (status is not null)
            query = query.Where(i => i.Status == status);
        q = q.Trim().ToLowerInvariant();
        if (q.Length > 0)
            query = query.Where(i => i.Title.ToLower().Contains(q) || i.Description.ToLower().Contains(q));
        return Ok(await query.OrderByDescending(i => i.Id).Skip(offset).Take(limit).ToListAsync(cancellation));
    }
    [HttpGet("{id:long}")]
    public async Task<IActionResult> Get(long id, CancellationToken cancellation) => await database.Issues.AsNoTracking().SingleOrDefaultAsync(i => i.Id == id, cancellation) is { } issue ? Ok(issue) : Problem(statusCode: 404, detail: "Issue not found");
    [HttpPost]
    public async Task<IActionResult> Create(IssueInput input, CancellationToken cancellation)
    {
        var issue = new Issue { Title = input.Title.Trim(), Description = input.Description.Trim(), Priority = input.Priority };
        await using var transaction = await database.Database.BeginTransactionAsync(cancellation);
        database.Issues.Add(issue);
        await database.SaveChangesAsync(cancellation);
        database.Events.Add(new()
        {
            IssueId = issue.Id,
            Version = 1,
            Action = "created"
        });
        await database.SaveChangesAsync(cancellation);
        await transaction.CommitAsync(cancellation);
        return CreatedAtAction(nameof(Get), new
        {
            id = issue.Id
        }, issue);
    }
    [HttpPut("{id:long}")]
    public async Task<IActionResult> Update(long id, IssueInput input, CancellationToken cancellation)
    {
        var issue = await database.Issues.SingleOrDefaultAsync(i => i.Id == id, cancellation);
        if (issue is null)
            return Problem(statusCode: 404, detail: "Issue not found");
        if (input.Version is null or < 1)
            return Problem(statusCode: 400, detail: "version is required and must be positive");
        if (issue.Version != input.Version)
            return Problem(statusCode: 409, detail: "Issue changed; reload before saving");
        if (input.Status is not ("open" or "in_progress" or "resolved"))
            return Problem(statusCode: 400, detail: "status must be open, in_progress or resolved");
        if (issue.Status == "open" && input.Status == "resolved")
            return Problem(statusCode: 409, detail: "Start work before resolving an issue");
        if (issue.Status == "resolved" && input.Status == "in_progress")
            return Problem(statusCode: 409, detail: "Reopen a resolved issue before starting work");
        var action = issue.Status == input.Status ? "edited" : $"{issue.Status} → {input.Status}";
        issue.Title = input.Title.Trim();
        issue.Description = input.Description.Trim();
        issue.Priority = input.Priority;
        issue.Status = input.Status;
        issue.Version++;
        issue.UpdatedAt = DateTimeOffset.UtcNow.ToString("O");
        database.Events.Add(new()
        {
            IssueId = id,
            Version = issue.Version,
            Action = action
        });
        try
        {
            await database.SaveChangesAsync(cancellation);
        }
        catch (DbUpdateConcurrencyException) { return Problem(statusCode: 409, detail: "Issue changed; reload before saving"); }
        catch (DbUpdateException)
        { // SQLite can report the audit uniqueness conflict before the version conflict.
            database.ChangeTracker.Clear();
            var current = await database.Issues.AsNoTracking().SingleAsync(i => i.Id == id, cancellation);
            if (current.Version != input.Version)
                return Problem(statusCode: 409, detail: "Issue changed; reload before saving");
            throw;
        }
        return Ok(issue);
    }
    [HttpGet("{id:long}/events")]
    public async Task<IActionResult> Events(long id, CancellationToken cancellation)
    {
        if (!await database.Issues.AnyAsync(i => i.Id == id, cancellation))
            return Problem(statusCode: 404, detail: "Issue not found");
        return Ok(await database.Events.AsNoTracking().Where(e => e.IssueId == id).OrderBy(e => e.Version).Select(e => new { e.Version, e.Action, e.HappenedAt }).ToListAsync(cancellation));
    }
}
public sealed class IssueInput
{
    [Required, StringLength(160)] public string Title { get; init; } = "";
    [StringLength(4000)] public string Description { get; init; } = "";
    [Required, RegularExpression("^(low|normal|high|urgent)$")] public string Priority { get; init; } = "normal";
    public string? Status
    {
        get; init;
    }
    public long? Version
    {
        get; init;
    }
}
