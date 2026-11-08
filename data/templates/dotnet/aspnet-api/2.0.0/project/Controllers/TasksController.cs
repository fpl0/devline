using System.ComponentModel.DataAnnotations;
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;

namespace TaskApi.Controllers;

[ApiController]
[Route("api/tasks")]
public sealed class TasksController(TaskDatabase database) : ControllerBase
{
    [HttpGet]
    public async Task<ActionResult<IReadOnlyList<TaskItem>>> List(CancellationToken cancellation) =>
        Ok(await database.Tasks.AsNoTracking()
            .OrderBy(task => task.Title).ThenBy(task => task.Id).ToListAsync(cancellation));

    [HttpGet("{id:guid}")]
    public async Task<ActionResult<TaskItem>> Get(Guid id, CancellationToken cancellation)
    {
        var task = await database.Tasks.AsNoTracking().SingleOrDefaultAsync(task => task.Id == id, cancellation);
        return task is null ? NotFound() : Ok(task);
    }

    [HttpPost]
    public async Task<ActionResult<TaskItem>> Create(TaskInput input, CancellationToken cancellation)
    {
        var task = new TaskItem { Title = input.Title.Trim(), Completed = input.Completed };
        database.Tasks.Add(task);
        await database.SaveChangesAsync(cancellation);
        return CreatedAtAction(nameof(Get), new { id = task.Id }, task);
    }

    [HttpPut("{id:guid}")]
    public async Task<IActionResult> Replace(Guid id, TaskInput input, CancellationToken cancellation)
    {
        var title = input.Title.Trim();
        var changed = await database.Tasks.Where(task => task.Id == id).ExecuteUpdateAsync(update => update
            .SetProperty(task => task.Title, title)
            .SetProperty(task => task.Completed, input.Completed), cancellation);
        return changed == 0 ? NotFound() : NoContent();
    }

    [HttpDelete("{id:guid}")]
    public async Task<IActionResult> Delete(Guid id, CancellationToken cancellation)
    {
        var changed = await database.Tasks.Where(task => task.Id == id).ExecuteDeleteAsync(cancellation);
        return changed == 0 ? NotFound() : NoContent();
    }
}

public sealed class TaskInput
{
    [Required, StringLength(160)]
    public string Title { get; init; } = "";

    public bool Completed { get; init; }
}
