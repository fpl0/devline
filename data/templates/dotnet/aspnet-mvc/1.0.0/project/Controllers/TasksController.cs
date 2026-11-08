using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using TaskBoard.Models;
using TaskBoard.Services;

namespace TaskBoard.Controllers;

[Authorize(Policy = "TaskOwner")]
[ResponseCache(NoStore = true, Location = ResponseCacheLocation.None)]
[Route("tasks")]
public sealed class TasksController(TaskStore tasks) : Controller
{
    // The policy requires the stable OIDC subject. Usernames can change.
    private string Owner => User.FindFirst("sub")!.Value;

    [HttpGet("")]
    public async Task<IActionResult> Index(CancellationToken cancellation) =>
        View(await BoardAsync(cancellation));

    [HttpPost("")]
    public async Task<IActionResult> Create(TaskInput input, CancellationToken cancellation)
    {
        if (!ModelState.IsValid)
        {
            Response.StatusCode = StatusCodes.Status400BadRequest;
            return View(nameof(Index), await BoardAsync(cancellation));
        }

        await tasks.CreateAsync(Owner, input.Title, cancellation);
        return RedirectToAction(nameof(Index));
    }

    [HttpPost("{id:guid}/complete")]
    public async Task<IActionResult> Complete(Guid id, CancellationToken cancellation) =>
        await tasks.CompleteAsync(Owner, id, cancellation) ? RedirectToAction(nameof(Index)) : NotFound();

    [HttpPost("{id:guid}/delete")]
    public async Task<IActionResult> Delete(Guid id, CancellationToken cancellation) =>
        await tasks.DeleteAsync(Owner, id, cancellation) ? RedirectToAction(nameof(Index)) : NotFound();

    private async Task<TaskBoardModel> BoardAsync(CancellationToken cancellation)
    {
        var (rows, source) = await tasks.ListAsync(Owner, cancellation);
        Response.Headers["X-Task-Source"] = source;
        return new TaskBoardModel(rows);
    }
}
