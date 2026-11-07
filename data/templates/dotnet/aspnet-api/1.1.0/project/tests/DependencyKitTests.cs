using System.IdentityModel.Tokens.Jwt;
using System.Security.Claims;
using Amazon.Runtime;
using Amazon.S3;
using Amazon.S3.Model;
using Microsoft.IdentityModel.Tokens;
using MySqlConnector;
using Npgsql;
using RabbitMQ.Client;
using StackExchange.Redis;
using Xunit;

public sealed class DependencyKitTests
{
    [Fact]
    public void ServiceClientsCanBeConfiguredWithoutConnections()
    {
        Assert.Equal("devline", new NpgsqlConnectionStringBuilder("Host=localhost;Database=devline").Database);
        Assert.Equal("devline", new MySqlConnectionStringBuilder("Server=localhost;Database=devline").Database);
        Assert.Single(ConfigurationOptions.Parse("127.0.0.1:6379").EndPoints);
        var rabbit = new ConnectionFactory { Uri = new Uri("amqp://guest:guest@localhost/devline") };
        Assert.Equal("devline", rabbit.VirtualHost);
    }

    [Fact]
    public async Task SignsAS3RequestLocally()
    {
        using var s3 = new AmazonS3Client(new BasicAWSCredentials("offline-example", "test-only-secret"),
            new AmazonS3Config { ServiceURL = "http://127.0.0.1:9000", AuthenticationRegion = "us-east-1", ForcePathStyle = true });
        var url = await s3.GetPreSignedURLAsync(new GetPreSignedUrlRequest
        {
            BucketName = "devline-test",
            Key = "readme.txt",
            Expires = DateTime.UtcNow.AddMinutes(5),
        });
        Assert.Contains("X-Amz-Signature=", url);
    }

    [Fact]
    public void VerifiesASignedJwtLocally()
    {
        var key = new SymmetricSecurityKey(new byte[32]);
        var codec = new JwtSecurityTokenHandler();
        var token = codec.CreateEncodedJwt(new SecurityTokenDescriptor
        {
            Subject = new ClaimsIdentity(new[] { new Claim("sub", "offline-user") }),
            SigningCredentials = new SigningCredentials(key, SecurityAlgorithms.HmacSha256),
        });
        var principal = codec.ValidateToken(token, new TokenValidationParameters
        {
            IssuerSigningKey = key,
            ValidateIssuer = false,
            ValidateAudience = false,
        }, out _);
        Assert.Equal("offline-user", principal.FindFirstValue(ClaimTypes.NameIdentifier));
    }
}
