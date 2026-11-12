using System.Security.Claims;
using Microsoft.AspNetCore.Authentication;
using Microsoft.AspNetCore.Authentication.Cookies;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
namespace TaskBoard.Controllers;

public sealed class AccountController : Controller
{
    [HttpGet("/account/login")] public IActionResult Login() => View();
    [HttpPost("/account/login")]
    public async Task<IActionResult> Login(string? username, string? password)
    {
        if (username is not ("alice" or "bob") || password != "devline-local")
        {
            Response.StatusCode = 400;
            ViewBag.Error = "Use alice or bob with the local demo password.";
            return View();
        }
        var identity = new ClaimsIdentity([new(ClaimTypes.Name, username), new(ClaimTypes.Role, username == "bob" ? "Approver" : "Buyer")], CookieAuthenticationDefaults.AuthenticationScheme);
        await HttpContext.SignInAsync(CookieAuthenticationDefaults.AuthenticationScheme, new ClaimsPrincipal(identity));
        return Redirect("/");
    }
    [Authorize, HttpPost("/account/logout")]
    public async Task<IActionResult> Logout()
    {
        await HttpContext.SignOutAsync();
        return Redirect("/account/login");
    }
    [HttpGet("/account/denied")] public IActionResult Denied() => StatusCode(403);
}
