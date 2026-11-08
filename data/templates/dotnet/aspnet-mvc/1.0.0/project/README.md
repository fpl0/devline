# ASP.NET Core MVC Task Board

Sign in with Keycloak to add, complete, and delete tasks on a personal board. Razor renders the pages. EF Core stores tasks in PostgreSQL, Redis caches each user's task list, and a RabbitMQ worker delivers a message when a task is added.

Create the project with Devline's PostgreSQL, Redis, RabbitMQ, and Keycloak services selected. Docker and the four local image archives are needed to run the app. The .NET restore and Razor build work offline from the bundled NuGet feed.

```sh
./devline.sh test
./devline.sh start
```

1. Open [the task board](http://127.0.0.1:5081) and choose **Open my tasks**.
2. Sign in as `demo` with password `demo-password`. Add a task. Refresh the board to see its delivery status change to **Delivered**.
3. Complete a task and sign out. Sign in as `reader` with password `reader-password` to see a separate board.
4. Sign in as `demo` again to see the saved task. A reload on a settled board can return `X-Task-Source: redis`. Writes invalidate the user's cached list.

The [readiness endpoint](http://127.0.0.1:5081/health/ready) reports the state of all four services. To stop the app and its services, use `./devline.sh stop` and `./devline.sh services down`. The service volumes retain the realm and tasks.

Devline passes the service endpoints and credentials to the app. On first launch it seeds a dedicated `devline-dotnet` Keycloak realm and two demo accounts. The confidential `task-board-dotnet` client uses authorization code flow with PKCE and exact login and logout callbacks. MVC forms use antiforgery tokens. Queries and writes scope tasks to the authenticated subject. The automatic demo setup is for loopback development.

A task remains pending until the worker receives a broker confirmation. The worker records delivery in PostgreSQL before acknowledging the message. Retries can redeliver a message, so the delivery update is idempotent. The demo creates its schema on an empty project database at startup.

For an existing realm and services, set `ConnectionStrings__Postgres`, `ConnectionStrings__RabbitMQ`, `Redis__Url`, and `Keycloak__Issuer`, `Keycloak__ClientId`, and `Keycloak__ClientSecret`. Register the app's exact `/signin-oidc` and `/signout-callback-oidc` URLs with that client. Devline uses the launch URL in its project configuration. When running directly with `dotnet run`, set `ASPNETCORE_URLS` to change the address.

[ASP.NET Core OpenID Connect](https://learn.microsoft.com/aspnet/core/security/authentication/configure-oidc-web-authentication?view=aspnetcore-10.0) · [Keycloak Admin REST API](https://www.keycloak.org/docs-api/latest/rest-api/index.html) · [RabbitMQ publisher confirms](https://www.rabbitmq.com/tutorials/tutorial-seven-dotnet)
