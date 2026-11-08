using System.Net;
using System.Net.Http.Headers;
using System.Net.Http.Json;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json.Nodes;

namespace TaskBoard.Services;

public sealed class KeycloakSettings(Uri issuer, string clientId, string clientSecret)
{
    public Uri Issuer { get; } = issuer;
    public string ClientId { get; } = clientId;
    public string ClientSecret { get; } = clientSecret;
    public string DiscoveryUrl => $"{Issuer.AbsoluteUri.TrimEnd('/')}/.well-known/openid-configuration";
}

public static class KeycloakDemo
{
    private const string Realm = "devline-dotnet";
    private const string ClientId = "task-board-dotnet";

    public static async Task<KeycloakSettings> ConfigureAsync(IConfiguration configuration, Uri appUrl)
    {
        if (configuration["Keycloak:Issuer"] is { Length: > 0 } externalIssuer)
        {
            return new KeycloakSettings(
                new Uri(externalIssuer),
                configuration["Keycloak:ClientId"] ?? ClientId,
                configuration["Keycloak:ClientSecret"]
                    ?? throw new InvalidOperationException("Set Keycloak__ClientSecret for your configured realm."));
        }

        var server = new Uri(configuration["DEVLINE_KEYCLOAK_URL"]
            ?? throw new InvalidOperationException("Run this app through Devline with Keycloak configured."));
        var password = configuration["DEVLINE_KEYCLOAK_PASSWORD"]
            ?? throw new InvalidOperationException("Devline's Keycloak bootstrap credential is missing.");
        if (!server.IsLoopback || !appUrl.IsLoopback)
            throw new InvalidOperationException("The seeded Keycloak realm requires loopback addresses.");

        // Each local Devline stack has a different admin password. Derive a dedicated
        // client secret so restarts and app-port changes can reuse the imported realm.
        var clientSecret = Convert.ToHexString(HMACSHA256.HashData(
            Encoding.UTF8.GetBytes(password), Encoding.UTF8.GetBytes($"{Realm}/{ClientId}")));
        using var client = new HttpClient(new HttpClientHandler { AllowAutoRedirect = false })
        {
            BaseAddress = server,
            Timeout = TimeSpan.FromSeconds(10)
        };

        using var credentials = new FormUrlEncodedContent(new Dictionary<string, string>
        {
            ["grant_type"] = "password",
            ["client_id"] = "admin-cli",
            ["username"] = "devline",
            ["password"] = password
        });
        using var login = await client.PostAsync("/realms/master/protocol/openid-connect/token", credentials);
        login.EnsureSuccessStatusCode();
        var token = (await login.Content.ReadFromJsonAsync<JsonObject>())?["access_token"]?.GetValue<string>()
            ?? throw new InvalidOperationException("Keycloak did not issue an admin token.");
        client.DefaultRequestHeaders.Authorization = new AuthenticationHeaderValue("Bearer", token);

        var callback = new Uri(appUrl, "/signin-oidc").AbsoluteUri;
        var signoutCallback = new Uri(appUrl, "/signout-callback-oidc").AbsoluteUri;
        using var existing = await client.GetAsync($"/admin/realms/{Realm}");
        if (existing.StatusCode == HttpStatusCode.NotFound)
        {
            using var created = await client.PostAsJsonAsync("/admin/realms", new
            {
                realm = Realm,
                enabled = true,
                sslRequired = "external",
                registrationAllowed = false,
                clients = new[]
                {
                    new
                    {
                        clientId = ClientId,
                        name = "C# Task Board",
                        enabled = true,
                        protocol = "openid-connect",
                        publicClient = false,
                        standardFlowEnabled = true,
                        directAccessGrantsEnabled = false,
                        implicitFlowEnabled = false,
                        secret = clientSecret,
                        redirectUris = new[] { callback },
                        attributes = new Dictionary<string, string>
                        {
                            ["pkce.code.challenge.method"] = "S256",
                            ["post.logout.redirect.uris"] = signoutCallback
                        }
                    }
                },
                users = new[] { DemoUser("demo", "Demo"), DemoUser("reader", "Reader") }
            });
            if (created.StatusCode != HttpStatusCode.Conflict)
                created.EnsureSuccessStatusCode();
        }
        else
        {
            existing.EnsureSuccessStatusCode();
        }

        // The managed app's port may be changed after the initial realm import.
        // Keep the existing client and users, and update its two exact redirects.
        var clients = await client.GetFromJsonAsync<JsonArray>($"/admin/realms/{Realm}/clients?clientId={ClientId}");
        if (clients is not { Count: 1 } || clients[0] is not JsonObject appClient ||
            appClient["id"]?.GetValue<string>() is not { } internalId)
            throw new InvalidOperationException($"Expected one {ClientId} client in the seeded {Realm} realm.");

        var attributes = appClient["attributes"]?.AsObject() ?? new JsonObject();
        var registered = appClient["redirectUris"]?.AsArray();
        if (registered is not { Count: 1 } || registered[0]?.GetValue<string>() != callback ||
            attributes["post.logout.redirect.uris"]?.GetValue<string>() != signoutCallback)
        {
            var update = new JsonObject
            {
                ["redirectUris"] = new JsonArray(JsonValue.Create(callback)),
                ["attributes"] = attributes.DeepClone()
            };
            update["attributes"]!["post.logout.redirect.uris"] = signoutCallback;
            using var response = await client.PutAsJsonAsync($"/admin/realms/{Realm}/clients/{internalId}", update);
            response.EnsureSuccessStatusCode();
        }

        return new KeycloakSettings(new Uri(server, $"/realms/{Realm}"), ClientId, clientSecret);
    }

    private static object DemoUser(string username, string firstName) => new
    {
        username,
        firstName,
        lastName = "Task Board",
        email = $"{username}@example.invalid",
        emailVerified = true,
        enabled = true,
        credentials = new[] { new { type = "password", value = $"{username}-password", temporary = false } }
    };
}
