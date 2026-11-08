using Microsoft.AspNetCore.Authentication.Cookies;
using Microsoft.AspNetCore.Authentication.OpenIdConnect;
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;
using Microsoft.IdentityModel.Protocols.OpenIdConnect;
using StackExchange.Redis;
using TaskBoard.Data;
using TaskBoard.Services;

var builder = WebApplication.CreateBuilder(args);
var config = builder.Configuration;
var appUrl = new Uri(config["ASPNETCORE_URLS"] ?? "http://127.0.0.1:5081");
var postgres = config.GetConnectionString("Postgres")
    ?? throw new InvalidOperationException("Configure PostgreSQL with Devline or set ConnectionStrings__Postgres.");
var redisAddress = config["DEVLINE_REDIS_URL"] ?? config["Redis:Url"]
    ?? throw new InvalidOperationException("Configure Redis with Devline or set Redis__Url.");
var rabbitAddress = config.GetConnectionString("RabbitMQ")
    ?? throw new InvalidOperationException("Configure RabbitMQ with Devline or set ConnectionStrings__RabbitMQ.");

var oidc = await KeycloakDemo.ConfigureAsync(config, appUrl);
var redisUri = new Uri(redisAddress);
if (redisUri.Scheme is not ("redis" or "rediss"))
    throw new InvalidOperationException("Redis URL must use redis:// or rediss://.");
var redisUserInfo = redisUri.UserInfo.Split(':', 2);
var redisOptions = new ConfigurationOptions
{
    EndPoints = { { redisUri.Host, redisUri.IsDefaultPort ? 6379 : redisUri.Port } },
    User = redisUserInfo.Length == 2 && redisUserInfo[0].Length > 0 ? Uri.UnescapeDataString(redisUserInfo[0]) : null,
    Password = redisUserInfo.Length == 2 ? Uri.UnescapeDataString(redisUserInfo[1]) : null,
    DefaultDatabase = int.TryParse(redisUri.AbsolutePath.Trim('/'), out var redisDatabase) ? redisDatabase : 0,
    Ssl = redisUri.Scheme == "rediss",
    AbortOnConnectFail = false,
    ConnectTimeout = 2500,
    AsyncTimeout = 2500
};
using var redis = await ConnectionMultiplexer.ConnectAsync(redisOptions);

builder.WebHost.UseUrls(appUrl.GetLeftPart(UriPartial.Authority));
builder.Services.AddControllersWithViews(options =>
    options.Filters.Add(new AutoValidateAntiforgeryTokenAttribute()));
builder.Services.AddAuthentication(options =>
{
    options.DefaultScheme = CookieAuthenticationDefaults.AuthenticationScheme;
    options.DefaultChallengeScheme = OpenIdConnectDefaults.AuthenticationScheme;
})
.AddCookie(options =>
{
    options.Cookie.Name = "TaskBoard.Session";
    options.Cookie.HttpOnly = true;
    options.Cookie.SecurePolicy = appUrl.Scheme == "https" ? CookieSecurePolicy.Always : CookieSecurePolicy.SameAsRequest;
})
.AddOpenIdConnect(options =>
{
    options.Authority = oidc.Issuer.AbsoluteUri;
    options.ClientId = oidc.ClientId;
    options.ClientSecret = oidc.ClientSecret;
    options.RequireHttpsMetadata = !oidc.Issuer.IsLoopback || oidc.Issuer.Scheme == "https";
    options.ResponseType = OpenIdConnectResponseType.Code;
    options.ResponseMode = OpenIdConnectResponseMode.Query;
    options.UsePkce = true;
    options.SaveTokens = true;
    options.MapInboundClaims = false;
    options.TokenValidationParameters.NameClaimType = "preferred_username";

    if (appUrl.IsLoopback && appUrl.Scheme == "http")
    {
        options.CorrelationCookie.SecurePolicy = CookieSecurePolicy.SameAsRequest;
        options.CorrelationCookie.SameSite = SameSiteMode.Lax;
        options.NonceCookie.SecurePolicy = CookieSecurePolicy.SameAsRequest;
        options.NonceCookie.SameSite = SameSiteMode.Lax;
    }
});
builder.Services.AddAuthorization(options =>
    options.AddPolicy("TaskOwner", policy => policy.RequireAuthenticatedUser().RequireClaim("sub")));
builder.Services.AddDbContext<TaskDatabase>(options => options.UseNpgsql(postgres));
builder.Services.AddSingleton<IConnectionMultiplexer>(redis);
builder.Services.AddScoped<TaskStore>();
builder.Services.AddSingleton(new BrokerOptions(new Uri(rabbitAddress)));
builder.Services.AddSingleton<TaskDelivery>();
builder.Services.AddHostedService(services => services.GetRequiredService<TaskDelivery>());
builder.Services.AddHttpClient("keycloak-health", client => client.Timeout = TimeSpan.FromSeconds(2));

var app = builder.Build();
await using (var scope = app.Services.CreateAsyncScope())
    await scope.ServiceProvider.GetRequiredService<TaskDatabase>().Database.EnsureCreatedAsync();

app.UseExceptionHandler("/error");
app.UseStaticFiles();
app.UseAuthentication();
app.UseAuthorization();
app.MapControllers();
app.MapGet("/health/ready", async (
    TaskDatabase database, TaskDelivery delivery, IHttpClientFactory clients, CancellationToken cancellation) =>
{
    var postgresReady = await database.Database.CanConnectAsync(cancellation);
    var redisReady = redis.IsConnected;
    var rabbitReady = delivery.Connected;
    var keycloakReady = false;
    try
    {
        using var response = await clients.CreateClient("keycloak-health").GetAsync(oidc.DiscoveryUrl, cancellation);
        keycloakReady = response.IsSuccessStatusCode;
    }
    catch (HttpRequestException) { }
    catch (OperationCanceledException) when (!cancellation.IsCancellationRequested) { }
    var ready = postgresReady && redisReady && rabbitReady && keycloakReady;
    return Results.Json(new
    {
        ready,
        postgres = postgresReady,
        redis = redisReady,
        rabbitmq = rabbitReady,
        keycloak = keycloakReady
    }, statusCode: ready ? 200 : 503);
});
app.Map("/error", () => Results.Problem("The task board could not complete this request."));
await app.RunAsync();
