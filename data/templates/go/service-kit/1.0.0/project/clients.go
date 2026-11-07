package main

import (
	"context"
	"database/sql"
	"errors"
	"net/http"
	"net/url"
	"strings"
	"time"

	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/credentials"
	"github.com/aws/aws-sdk-go-v2/service/s3"
	"github.com/go-sql-driver/mysql"
	"github.com/golang-jwt/jwt/v5"
	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/nats-io/nats.go"
	_ "github.com/ncruces/go-sqlite3/driver"
	amqp "github.com/rabbitmq/amqp091-go"
	"github.com/redis/go-redis/v9"
)

// PostgreSQL returns a pool that connects when used. Close it when done.
func PostgreSQL(ctx context.Context, dsn string) (*pgxpool.Pool, error) {
	config, err := pgxpool.ParseConfig(dsn)
	if err != nil {
		return nil, err
	}
	config.MinConns = 0
	config.MinIdleConns = 0
	config.ConnConfig.ConnectTimeout = 5 * time.Second
	return pgxpool.NewWithConfig(ctx, config)
}

func MySQL(dsn string) (*sql.DB, error) {
	config, err := mysql.ParseDSN(dsn)
	if err != nil {
		return nil, err
	}
	config.Timeout = 5 * time.Second
	connector, err := mysql.NewConnector(config)
	if err != nil {
		return nil, err
	}
	return sql.OpenDB(connector), nil
}

func SQLite(path string) (*sql.DB, error) {
	db, err := sql.Open("sqlite3", path)
	if err != nil {
		return nil, err
	}
	db.SetMaxOpenConns(1)
	return db, nil
}

func Redis(dsn string) (*redis.Client, error) {
	options, err := redis.ParseURL(dsn)
	if err != nil {
		return nil, err
	}
	options.DialTimeout = 5 * time.Second
	return redis.NewClient(options), nil
}

// RabbitMQURL validates settings without opening a connection. Use amqp.Dial
// with the returned URI when the configured service is running.
func RabbitMQURL(dsn string) (amqp.URI, error) { return amqp.ParseURI(dsn) }

// NATSOptions can be used with Options.Connect when the configured service runs.
func NATSOptions(dsn string) (nats.Options, error) {
	endpoint, err := url.Parse(dsn)
	if err != nil || endpoint.Hostname() == "" || !strings.Contains("|nats|tls|ws|wss|", "|"+endpoint.Scheme+"|") {
		return nats.Options{}, errors.New("provide a NATS URL such as nats://127.0.0.1:4222")
	}
	options := nats.GetDefaultOptions()
	options.Servers = []string{dsn}
	options.Timeout = 5 * time.Second
	return options, nil
}

// ObjectStorage sets static credentials explicitly. It does not query a cloud
// metadata service or discover credentials from the current machine.
func ObjectStorage(endpoint, region, key, secret string) (*s3.Client, error) {
	address, err := url.Parse(endpoint)
	if err != nil || address.Hostname() == "" || (address.Scheme != "http" && address.Scheme != "https") || region == "" || key == "" || secret == "" {
		return nil, errors.New("provide an S3 endpoint, region, access key and secret")
	}
	return s3.New(s3.Options{
		Region: region, BaseEndpoint: aws.String(endpoint), UsePathStyle: true,
		Credentials: credentials.NewStaticCredentialsProvider(key, secret, ""),
		HTTPClient:  HTTPClient(),
	}), nil
}

func HTTPClient() *http.Client { return &http.Client{Timeout: 5 * time.Second} }

func MintToken(key []byte, subject string) (string, error) {
	if len(key) < 32 || subject == "" {
		return "", errors.New("provide a subject and a signing key of at least 32 bytes")
	}
	now := time.Now()
	claims := jwt.RegisteredClaims{Issuer: "service-kit", Subject: subject, IssuedAt: jwt.NewNumericDate(now), ExpiresAt: jwt.NewNumericDate(now.Add(time.Hour))}
	return jwt.NewWithClaims(jwt.SigningMethodHS256, claims).SignedString(key)
}

func VerifyToken(key []byte, token string) (*jwt.RegisteredClaims, error) {
	if len(key) < 32 {
		return nil, errors.New("signing key is too short")
	}
	claims := new(jwt.RegisteredClaims)
	_, err := jwt.ParseWithClaims(token, claims, func(*jwt.Token) (any, error) { return key, nil },
		jwt.WithValidMethods([]string{"HS256"}), jwt.WithIssuer("service-kit"), jwt.WithExpirationRequired())
	if err != nil {
		return nil, err
	}
	return claims, nil
}
