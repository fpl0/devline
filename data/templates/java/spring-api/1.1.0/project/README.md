# __PROJECT_NAME__

Spring Boot HTTP API using Java 26 and Maven 3.10. Requires the JDK and Maven to be installed before going offline.

Run `./test-offline.sh` to build and test against the bundled Maven repository.
Launch the resulting executable JAR with `java -jar target/__PROJECT_NAME__-0.0.1-SNAPSHOT.jar`.

## Included local packages

The local Maven repository includes Spring Boot Web MVC, PostgreSQL, MySQL
and SQLite JDBC drivers, Lettuce for Redis, RabbitMQ's AMQP client, AWS SDK
for S3, Spring Security JOSE with Nimbus, and the Spring Boot JUnit test stack.
Spring Boot supplies HTTP clients and structured logging. The POM pins Spring
Boot's dependency BOM and the AWS SDK.

`./test-offline.sh` runs Maven offline. The suite boots a Spring test
context, uses an in-memory SQLite database, configures clients, signs an S3
request, and verifies a JWT. PostgreSQL, MySQL, Redis, RabbitMQ and S3
operations require service endpoints and credentials.
