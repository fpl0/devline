package dev.offline.board;

import java.io.IOException;
import java.net.URI;
import java.net.URLEncoder;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.time.Duration;
import java.util.HexFormat;
import java.util.Objects;

import tools.jackson.databind.JsonNode;
import tools.jackson.databind.json.JsonMapper;
import tools.jackson.databind.node.ObjectNode;

/** Seed the local demo realm before Spring Security reads its OIDC discovery document. */
final class KeycloakDemo {
    private static final String REALM = "/admin/realms/devline-java";
    private static final String CLIENTS = REALM + "/clients";
    private static final String MARKER = "java-task-board";
    private static final Duration TIMEOUT = Duration.ofSeconds(10);
    private static final JsonMapper JSON = JsonMapper.builder().build();

    private KeycloakDemo() {}

    static String prepare() throws IOException, InterruptedException {
        URI server = loopback("DEVLINE_KEYCLOAK_URL");
        URI board = loopback("TASK_BOARD_BASE_URL");
        String password = required("DEVLINE_KEYCLOAK_PASSWORD");
        String secret = clientSecret(password);
        try (var http = HttpClient.newBuilder()
                .proxy(HttpClient.Builder.NO_PROXY)
                .followRedirects(HttpClient.Redirect.NEVER)
                .connectTimeout(TIMEOUT)
                .build()) {
            var admin = send(http, server, "POST", "/realms/master/protocol/openid-connect/token",
                    "grant_type=password&client_id=admin-cli&username=devline&password="
                            + URLEncoder.encode(password, StandardCharsets.UTF_8),
                    "application/x-www-form-urlencoded", null, 200);
            String token = JSON.readTree(admin.body()).path("access_token").asString();
            if (token.isBlank()) {
                throw new IOException("Keycloak did not issue a bootstrap token");
            }

            ObjectNode realm;
            try (var input = KeycloakDemo.class.getResourceAsStream("/keycloak-realm.json")) {
                realm = (ObjectNode) JSON.readTree(Objects.requireNonNull(input, "Missing demo realm"));
            }
            var client = (ObjectNode) realm.path("clients").get(0);
            client.put("secret", secret);
            client.putArray("redirectUris").add(board.resolve("/login/oauth2/code/keycloak").toString());
            String logout = board.resolve("/").toString();
            ((ObjectNode) client.path("attributes")).put("post.logout.redirect.uris", logout);

            var existing = send(http, server, "GET", REALM, null, null, token, 200, 404);
            if (existing.statusCode() == 404) {
                send(http, server, "POST", "/admin/realms", JSON.writeValueAsString(realm),
                        "application/json", token, 201);
                return secret;
            }
            if (!MARKER.equals(JSON.readTree(existing.body()).path("attributes").path("devline.demo").asString())) {
                throw new IOException("The existing devline-java realm does not belong to this demo");
            }
            var search = send(http, server, "GET", CLIENTS + "?clientId=task-board-java", null, null, token, 200);
            var clients = JSON.readTree(search.body());
            if (!clients.isArray() || clients.size() != 1 || !(clients.get(0) instanceof ObjectNode current)) {
                throw new IOException("Expected one task-board-java client in the demo realm");
            }
            updateRedirects(http, server, token, current, client.path("redirectUris"), logout);
        }
        return secret;
    }

    private static void updateRedirects(HttpClient http, URI server, String token, ObjectNode client,
            JsonNode redirects, String logout) throws IOException, InterruptedException {
        if (redirects.equals(client.path("redirectUris"))
                && logout.equals(client.path("attributes").path("post.logout.redirect.uris").asString())
                && "S256".equals(client.path("attributes").path("pkce.code.challenge.method").asString())) {
            return;
        }
        String id = client.path("id").asString();
        if (!id.matches("[0-9a-fA-F-]+")) {
            throw new IOException("Keycloak returned an invalid client ID");
        }
        var attributes = client.path("attributes") instanceof ObjectNode node
                ? node.deepCopy() : JSON.createObjectNode();
        attributes.put("post.logout.redirect.uris", logout);
        attributes.put("pkce.code.challenge.method", "S256");
        var update = JSON.createObjectNode();
        update.set("redirectUris", redirects);
        update.set("attributes", attributes);
        send(http, server, "PUT", CLIENTS + "/" + id, JSON.writeValueAsString(update),
                "application/json", token, 204);
    }

    private static HttpResponse<byte[]> send(HttpClient http, URI server, String method, String path,
            String body, String contentType, String token, int... accepted) throws IOException, InterruptedException {
        var request = HttpRequest.newBuilder(server.resolve(path))
                .timeout(TIMEOUT)
                .method(method, body == null ? HttpRequest.BodyPublishers.noBody()
                        : HttpRequest.BodyPublishers.ofString(body));
        if (contentType != null) {
            request.header("Content-Type", contentType);
        }
        if (token != null) {
            request.header("Authorization", "Bearer " + token);
        }
        var response = http.send(request.build(), HttpResponse.BodyHandlers.ofByteArray());
        for (int status : accepted) {
            if (response.statusCode() == status) {
                return response;
            }
        }
        throw new IOException("Keycloak returned HTTP " + response.statusCode() + " for " + method + " " + path);
    }

    private static URI loopback(String key) {
        URI uri = URI.create(required(key));
        int port = uri.getPort();
        if (port < 1 || port > 65535 || !uri.toString().equals("http://127.0.0.1:" + port)) {
            throw new IllegalArgumentException(key + " must be a loopback origin such as http://127.0.0.1:8180");
        }
        return uri;
    }

    private static String required(String key) {
        String value = System.getenv(key);
        if (value == null || value.isBlank()) {
            throw new IllegalStateException("Run through Devline with all four services configured; missing " + key);
        }
        return value;
    }

    private static String clientSecret(String password) {
        try {
            byte[] material = ("devline-java-oidc:" + password).getBytes(StandardCharsets.UTF_8);
            return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(material));
        } catch (NoSuchAlgorithmException error) {
            throw new IllegalStateException("SHA-256 is required by the JDK", error);
        }
    }
}
