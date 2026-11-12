using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using TaskBoard.Models;
using TaskBoard.Services;
namespace TaskBoard.Controllers;

[Authorize]
public sealed class PurchasesController(PurchaseService service) : Controller
{
    [HttpGet("/")]
    public async Task<IActionResult> Index(string status = "", string q = "", CancellationToken cancellation = default)
    {
        try
        {
            return View(new BoardModel(await service.List(status, q, cancellation), status, q));
        }
        catch (WorkflowException e) { return Failure(e); }
    }
    [HttpGet("/requests/{id:long}")]
    public async Task<IActionResult> Detail(long id, CancellationToken cancellation)
    {
        try
        {
            return View(new DetailModel(await service.Get(id, cancellation), await service.Events(id, cancellation)));
        }
        catch (WorkflowException e) { return Failure(e); }
    }
    [HttpPost("/requests/create")]
    public async Task<IActionResult> Create(string? title, string? reason, string? amount, CancellationToken cancellation)
    {
        try
        {
            var request = await service.Create(User.Identity!.Name!, title, reason, amount, cancellation);
            return Redirect($"/requests/{request.Id}");
        }
        catch (WorkflowException e) { return Failure(e); }
    }
    [HttpPost("/requests/{id:long}/decision")]
    public async Task<IActionResult> Decide(long id, long version, string? decision, string? note, CancellationToken cancellation)
    {
        try
        {
            await service.Decide(id, User.Identity!.Name!, User.IsInRole("Approver"), version, decision, note, cancellation);
            return Redirect($"/requests/{id}");
        }
        catch (WorkflowException e) { return Failure(e); }
    }
    IActionResult Failure(WorkflowException error)
    {
        Response.StatusCode = error.Status;
        return View("Error", error.Message);
    }
}
