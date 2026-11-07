package main

import (
	"bufio"
	"errors"
	"fmt"
	"io"
	"net"
	"net/url"
	"os"
	"strconv"
	"strings"
)

type configuration struct{ postgres, redis, listen string }

func loopback(host string) bool {
	return host == "localhost" || net.ParseIP(strings.Trim(host, "[]")).IsLoopback()
}

func loadConfig() (configuration, error) {
	listen := os.Getenv("TASKBOARD_LISTEN")
	if listen == "" {
		listen = "127.0.0.1:8080"
	}
	host, port, err := net.SplitHostPort(listen)
	if err != nil || !loopback(host) {
		return configuration{}, errors.New("TASKBOARD_LISTEN must be a loopback address such as 127.0.0.1:8080")
	}
	if _, err := strconv.ParseUint(port, 10, 16); err != nil {
		return configuration{}, errors.New("TASKBOARD_LISTEN needs a numeric port")
	}
	postgres, redis := os.Getenv("TASKBOARD_DATABASE_URL"), os.Getenv("TASKBOARD_REDIS_URL")
	if postgres == "" || redis == "" {
		values, err := readCredentials()
		if err != nil {
			return configuration{}, err
		}
		if postgres == "" {
			postgres, err = localServiceURL(values, "POSTGRES", "DATABASE", "postgres", "devline", "/devline")
			if err != nil {
				return configuration{}, err
			}
			postgres += "?sslmode=disable"
		}
		if redis == "" {
			redis, err = localServiceURL(values, "REDIS", "REDIS", "redis", "", "/0")
			if err != nil {
				return configuration{}, err
			}
		}
	}
	if !localURL(postgres, "postgres", "postgresql") || !localURL(redis, "redis", "rediss") {
		return configuration{}, errors.New("PostgreSQL and Redis URLs must point to loopback services")
	}
	return configuration{postgres: postgres, redis: redis, listen: listen}, nil
}

func localURL(raw string, schemes ...string) bool {
	parsed, err := url.Parse(raw)
	if err != nil || !loopback(parsed.Hostname()) {
		return false
	}
	for _, scheme := range schemes {
		if parsed.Scheme == scheme {
			return true
		}
	}
	return false
}

func localServiceURL(values map[string]string, service, portID, scheme, user, path string) (string, error) {
	password, port := values["DEVLINE_"+service+"_PASSWORD"], values["DEVLINE_"+service+"_"+portID+"_PORT"]
	portNumber, err := strconv.ParseUint(port, 10, 16)
	if password == "" || err != nil || portNumber == 0 {
		return "", fmt.Errorf("the project needs %s credentials. Configure the matching release with devline infra configure", service)
	}
	return (&url.URL{Scheme: scheme, User: url.UserPassword(user, password), Host: net.JoinHostPort("127.0.0.1", port), Path: path}).String(), nil
}

func readCredentials() (map[string]string, error) {
	file, err := os.Open(".devline-infra/credentials.env")
	if err != nil {
		return nil, errors.New("local credentials are unavailable. Run devline infra configure and devline infra run in the project folder")
	}
	defer file.Close()
	values := make(map[string]string)
	scanner := bufio.NewScanner(io.LimitReader(file, 16<<10))
	for scanner.Scan() {
		name, value, found := strings.Cut(strings.TrimSpace(scanner.Text()), "=")
		if found && strings.HasPrefix(name, "DEVLINE_") {
			values[name] = strings.Trim(strings.TrimSpace(value), "'\"")
		}
	}
	if err := scanner.Err(); err != nil {
		return nil, errors.New("could not read local credentials")
	}
	return values, nil
}
