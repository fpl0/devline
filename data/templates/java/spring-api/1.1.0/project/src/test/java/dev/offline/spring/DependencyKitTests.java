package dev.offline.spring;

import java.net.URI;
import java.sql.DriverManager;
import java.time.Duration;

import com.nimbusds.jose.JOSEObjectType;
import com.nimbusds.jose.JWSAlgorithm;
import com.nimbusds.jose.JWSHeader;
import com.nimbusds.jose.crypto.MACSigner;
import com.nimbusds.jose.crypto.MACVerifier;
import com.nimbusds.jwt.JWTClaimsSet;
import com.nimbusds.jwt.SignedJWT;
import com.rabbitmq.client.ConnectionFactory;
import io.lettuce.core.RedisURI;
import org.junit.jupiter.api.Test;
import software.amazon.awssdk.auth.credentials.AwsBasicCredentials;
import software.amazon.awssdk.auth.credentials.StaticCredentialsProvider;
import software.amazon.awssdk.regions.Region;
import software.amazon.awssdk.services.s3.S3Configuration;
import software.amazon.awssdk.services.s3.presigner.S3Presigner;

import static org.assertj.core.api.Assertions.assertThat;

class DependencyKitTests {
    @Test
    void createsAnInMemorySqliteDatabase() throws Exception {
        try (var connection = DriverManager.getConnection("jdbc:sqlite::memory:");
             var statement = connection.createStatement()) {
            statement.executeUpdate("CREATE TABLE notes (body TEXT NOT NULL)");
            statement.executeUpdate("INSERT INTO notes VALUES ('offline')");
            try (var result = statement.executeQuery("SELECT body FROM notes")) {
                assertThat(result.next()).isTrue();
                assertThat(result.getString("body")).isEqualTo("offline");
            }
        }
    }

    @Test
    void configuresDatabaseCacheAndQueueClientsWithoutConnections() throws Exception {
        assertThat(new org.postgresql.Driver().acceptsURL("jdbc:postgresql://localhost/devline")).isTrue();
        assertThat(new com.mysql.cj.jdbc.Driver().acceptsURL("jdbc:mysql://localhost/devline")).isTrue();
        assertThat(RedisURI.create("redis://localhost:6379").getPort()).isEqualTo(6379);
        var rabbit = new ConnectionFactory();
        rabbit.setUri("amqp://guest:guest@localhost/devline");
        assertThat(rabbit.getVirtualHost()).isEqualTo("devline");
    }

    @Test
    void signsS3RequestsAndVerifiesTokensLocally() throws Exception {
        try (var signer = S3Presigner.builder()
                .region(Region.US_EAST_1)
                .endpointOverride(URI.create("http://127.0.0.1:9000"))
                .credentialsProvider(StaticCredentialsProvider.create(
                        AwsBasicCredentials.create("offline-example", "test-only-secret")))
                .serviceConfiguration(S3Configuration.builder().pathStyleAccessEnabled(true).build())
                .build()) {
            var signed = signer.presignGetObject(request -> request
                    .signatureDuration(Duration.ofMinutes(5))
                    .getObjectRequest(object -> object.bucket("devline-test").key("readme.txt")));
            assertThat(signed.url().getQuery()).contains("X-Amz-Signature=");
        }

        var key = new byte[32];
        var token = new SignedJWT(new JWSHeader.Builder(JWSAlgorithm.HS256).type(JOSEObjectType.JWT).build(),
                new JWTClaimsSet.Builder().subject("offline-user").build());
        token.sign(new MACSigner(key));
        var decoded = SignedJWT.parse(token.serialize());
        assertThat(decoded.verify(new MACVerifier(key))).isTrue();
        assertThat(decoded.getJWTClaimsSet().getSubject()).isEqualTo("offline-user");
    }
}
