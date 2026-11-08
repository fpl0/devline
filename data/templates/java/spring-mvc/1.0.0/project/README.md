# __PROJECT_NAME__

A Spring Boot MVC task board with Thymeleaf views and Spring Security sign-in. Spring Data JDBC stores each user's tasks in PostgreSQL. Redis caches each user's list for 30 seconds. RabbitMQ delivers completion events to a listener that records the result in PostgreSQL.

Use JDK 26, Maven 3.10 or later, and a running Docker engine. Devline configures the four services for generated projects. If an image is missing from your offline library, import an archive or prepare it while connected.

```sh
devline infra prepare postgres@18.6
devline infra prepare redis@8.10.1
devline infra prepare rabbitmq@4.3.7
devline infra prepare keycloak@26.8.2
```

Run the app from the generated project. Devline starts the services from their local images, passes their connection settings, and builds the app with the bundled Maven repository. The app configures the local `devline-java` Keycloak realm on first run and preserves its users on subsequent launches.

```sh
./devline.sh run
```

Open [the task board](http://127.0.0.1:8081). Sign in with `demo` / `demo-password`, then try the following.

1. Add a task and refresh twice. The footer shows whether the list came from PostgreSQL or Redis. Writes invalidate the cached list.
2. Complete the task. The completion is queued, then the listener marks it **Recorded**. Refresh after a few seconds to see the update.
3. Sign out and sign in with `reader` / `reader-password`. This account has its own task list. Sign back in as `demo` and delete the task.

`./devline.sh test` builds the app offline without starting the services. The browser walkthrough exercises the live integrations. Use `./devline.sh services down` after stopping the app to stop the containers and keep their data. To change the app port, edit `run.url` in `.devline/config.json` and restart. Open the loopback URL shown there so it matches the registered Keycloak callback.

The realm and credentials are for local development. Spring Security uses authorization code flow with PKCE and CSRF protected forms. The completion publisher retries unconfirmed events from PostgreSQL, and its listener can handle repeated deliveries. See [Spring Security OIDC login](https://docs.spring.io/spring-security/reference/servlet/oauth2/login/core.html), [Spring AMQP publisher confirms](https://docs.spring.io/spring-amqp/reference/amqp/template.html), and the [Keycloak Admin REST API](https://www.keycloak.org/docs-api/latest/rest-api/index.html).
