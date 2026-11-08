package main

import (
	"bytes"
	"context"
	"database/sql"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"mime"
	"net/http"
	"net/url"
	"os"
	"regexp"
	"strings"
	"time"

	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/service/s3"
	"github.com/go-chi/chi/v5"
	"github.com/go-sql-driver/mysql"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/nats-io/nats.go"
	amqp "github.com/rabbitmq/amqp091-go"
	"github.com/redis/go-redis/v9"
)

type release struct {
	ID       string `json:"id"`
	Notes    string `json:"notes"`
	Complete bool   `json:"complete"`
}

type releaseLab struct {
	db         *pgxpool.Pool
	audit      *sql.DB
	cache      *redis.Client
	objects    *s3.Client
	brokerURL  string
	events     nats.Options
	mailURL    string
	searchURL  string
	searchKey  string
	httpClient *http.Client
}

var releaseID = regexp.MustCompile(`^[a-z0-9][a-z0-9-]{0,39}$`)

func newReleaseLab(ctx context.Context) (*releaseLab, error) {
	values := make(map[string]string)
	for _, key := range []string{
		"POSTGRES_URL", "MYSQL_URL", "REDIS_URL", "RABBITMQ_URL", "NATS_URL",
		"MAILPIT_WEB_PORT", "MEILISEARCH_URL", "MEILISEARCH_PASSWORD",
		"SEAWEEDFS_URL", "SEAWEEDFS_PASSWORD",
	} {
		values[key] = os.Getenv("DEVLINE_" + key)
		if values[key] == "" {
			return nil, fmt.Errorf("missing DEVLINE_%s. Start the generated project with ./devline.sh run", key)
		}
	}
	lab := &releaseLab{
		brokerURL: values["RABBITMQ_URL"],
		mailURL:   "http://127.0.0.1:" + values["MAILPIT_WEB_PORT"],
		searchURL: values["MEILISEARCH_URL"],
		searchKey: values["MEILISEARCH_PASSWORD"],
		httpClient: &http.Client{
			Timeout:       5 * time.Second,
			CheckRedirect: func(*http.Request, []*http.Request) error { return http.ErrUseLastResponse },
			Transport:     &http.Transport{},
		},
	}
	if err := lab.openStores(ctx, values); err != nil {
		lab.close()
		return nil, err
	}
	return lab, nil
}

func (lab *releaseLab) openStores(ctx context.Context, values map[string]string) error {
	var err error
	lab.db, err = PostgreSQL(ctx, values["POSTGRES_URL"])
	if err != nil {
		return errors.New("invalid PostgreSQL settings")
	}
	mysqlURL, err := url.Parse(values["MYSQL_URL"])
	if err != nil || mysqlURL.User == nil {
		return errors.New("invalid MySQL settings")
	}
	mysqlSettings := mysql.NewConfig()
	mysqlSettings.User = mysqlURL.User.Username()
	mysqlSettings.Passwd, _ = mysqlURL.User.Password()
	mysqlSettings.Net = "tcp"
	mysqlSettings.Addr = mysqlURL.Host
	mysqlSettings.DBName = strings.TrimPrefix(mysqlURL.Path, "/")
	lab.audit, err = MySQL(mysqlSettings.FormatDSN())
	if err != nil {
		return errors.New("invalid MySQL settings")
	}
	cacheOptions, err := redis.ParseURL(values["REDIS_URL"])
	if err != nil {
		return errors.New("invalid Redis settings")
	}
	cacheOptions.ContextTimeoutEnabled = true
	lab.cache = redis.NewClient(cacheOptions)
	lab.events, err = NATSOptions(values["NATS_URL"])
	if err != nil {
		return errors.New("invalid NATS settings")
	}
	lab.objects, err = ObjectStorage(values["SEAWEEDFS_URL"], "us-east-1", "devline", values["SEAWEEDFS_PASSWORD"])
	if err != nil {
		return errors.New("invalid SeaweedFS settings")
	}
	if _, err := lab.db.Exec(ctx, `CREATE TABLE IF NOT EXISTS releases (
		id text PRIMARY KEY, notes text NOT NULL, complete boolean NOT NULL DEFAULT false
	)`); err != nil {
		return errors.New("could not initialize releases in PostgreSQL")
	}
	if _, err := lab.audit.ExecContext(ctx, `CREATE TABLE IF NOT EXISTS release_audit (
		id varchar(40) PRIMARY KEY, recorded_at timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
	)`); err != nil {
		return errors.New("could not initialize audit in MySQL")
	}
	return nil
}

func (lab *releaseLab) close() {
	if lab.db != nil {
		lab.db.Close()
	}
	if lab.audit != nil {
		_ = lab.audit.Close()
	}
	if lab.cache != nil {
		_ = lab.cache.Close()
	}
	lab.httpClient.CloseIdleConnections()
}

func (lab *releaseLab) routes(router chi.Router) {
	router.Post("/releases/{id}", lab.create)
	router.Get("/releases/{id}", lab.show)
	router.Get("/releases/{id}/artifact", lab.download)
	router.Get("/search", lab.search)
}

func reply(w http.ResponseWriter, status int, payload any) {
	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Cache-Control", "no-store")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(payload)
}

func validReleaseID(w http.ResponseWriter, r *http.Request) string {
	id := chi.URLParam(r, "id")
	if !releaseID.MatchString(id) {
		http.Error(w, "Use 1 to 40 lowercase letters, numbers or hyphens for the release ID", http.StatusBadRequest)
		return ""
	}
	return id
}

func (lab *releaseLab) create(w http.ResponseWriter, r *http.Request) {
	id := validReleaseID(w, r)
	if id == "" {
		return
	}
	mediaType, _, err := mime.ParseMediaType(r.Header.Get("Content-Type"))
	if err != nil || mediaType != "application/json" {
		http.Error(w, "Use application/json", http.StatusUnsupportedMediaType)
		return
	}
	decoder := json.NewDecoder(http.MaxBytesReader(w, r.Body, 4096))
	decoder.DisallowUnknownFields()
	item := release{ID: id}
	var input struct {
		Notes string `json:"notes"`
	}
	if err := decoder.Decode(&input); err != nil || decoder.Decode(new(any)) != io.EOF {
		http.Error(w, "Provide one JSON object with release notes", http.StatusBadRequest)
		return
	}
	item.Notes = strings.TrimSpace(input.Notes)
	if item.Notes == "" || len(item.Notes) > 1024 || strings.ContainsRune(item.Notes, 0) {
		http.Error(w, "Provide notes between 1 and 1024 bytes", http.StatusBadRequest)
		return
	}
	ctx, cancel := context.WithTimeout(r.Context(), 25*time.Second)
	defer cancel()
	result, err := lab.db.Exec(ctx, "INSERT INTO releases (id, notes) VALUES ($1, $2) ON CONFLICT DO NOTHING", item.ID, item.Notes)
	if err != nil {
		reply(w, http.StatusServiceUnavailable, map[string]string{"failed_stage": "PostgreSQL"})
		return
	}
	if result.RowsAffected() == 0 {
		http.Error(w, "That release ID already exists. Inspect its receipt or use a new ID", http.StatusConflict)
		return
	}
	steps := []struct {
		service string
		perform func(context.Context, release) error
	}{
		{"SeaweedFS", lab.upload},
		{"RabbitMQ", lab.deliver},
		{"NATS", lab.broadcast},
		{"MySQL", lab.record},
		{"Meilisearch", lab.index},
		{"Mailpit", lab.notify},
	}
	for _, step := range steps {
		if err := step.perform(ctx, item); err != nil {
			reply(w, http.StatusServiceUnavailable, map[string]string{"id": item.ID, "failed_stage": step.service})
			return
		}
	}
	if _, err := lab.db.Exec(ctx, "UPDATE releases SET complete = true WHERE id = $1", item.ID); err != nil {
		reply(w, http.StatusServiceUnavailable, map[string]string{"id": item.ID, "failed_stage": "PostgreSQL"})
		return
	}
	item.Complete = true
	w.Header().Set("Location", "/releases/"+item.ID)
	reply(w, http.StatusCreated, item)
}

func (lab *releaseLab) show(w http.ResponseWriter, r *http.Request) {
	id := validReleaseID(w, r)
	if id == "" {
		return
	}
	ctx, cancel := context.WithTimeout(r.Context(), 5*time.Second)
	defer cancel()
	key := "release:" + id
	var item release
	cacheCtx, stop := context.WithTimeout(ctx, 500*time.Millisecond)
	cached, err := lab.cache.Get(cacheCtx, key).Bytes()
	stop()
	if err == nil && json.Unmarshal(cached, &item) == nil && item.ID == id && item.Complete {
		w.Header().Set("X-Release-Source", "redis")
		reply(w, http.StatusOK, item)
		return
	}
	item.ID = id
	err = lab.db.QueryRow(ctx, "SELECT notes, complete FROM releases WHERE id = $1", id).Scan(&item.Notes, &item.Complete)
	switch {
	case errors.Is(err, pgx.ErrNoRows):
		http.NotFound(w, r)
		return
	case err != nil:
		http.Error(w, "Could not read PostgreSQL receipt", http.StatusServiceUnavailable)
		return
	}
	if item.Complete {
		if encoded, err := json.Marshal(item); err == nil {
			cacheCtx, stop := context.WithTimeout(ctx, 500*time.Millisecond)
			_ = lab.cache.Set(cacheCtx, key, encoded, 30*time.Second).Err()
			stop()
		}
	}
	w.Header().Set("X-Release-Source", "postgres")
	reply(w, http.StatusOK, item)
}

func (lab *releaseLab) upload(ctx context.Context, item release) error {
	_, err := lab.objects.PutObject(ctx, &s3.PutObjectInput{
		Bucket: aws.String("devline"), Key: aws.String("releases/" + item.ID + ".txt"),
		Body: strings.NewReader(item.Notes), ContentType: aws.String("text/plain"),
	})
	return err
}

func (lab *releaseLab) download(w http.ResponseWriter, r *http.Request) {
	id := validReleaseID(w, r)
	if id == "" {
		return
	}
	ctx, cancel := context.WithTimeout(r.Context(), 5*time.Second)
	defer cancel()
	object, err := lab.objects.GetObject(ctx, &s3.GetObjectInput{
		Bucket: aws.String("devline"), Key: aws.String("releases/" + id + ".txt"),
	})
	if err != nil {
		var missing interface{ ErrorCode() string }
		if errors.As(err, &missing) && missing.ErrorCode() == "NoSuchKey" {
			http.NotFound(w, r)
		} else {
			http.Error(w, "Could not read the SeaweedFS object", http.StatusServiceUnavailable)
		}
		return
	}
	defer object.Body.Close()
	w.Header().Set("Content-Type", "text/plain; charset=utf-8")
	w.Header().Set("X-Content-Type-Options", "nosniff")
	_, _ = io.Copy(w, io.LimitReader(object.Body, 1024))
}

func (lab *releaseLab) deliver(ctx context.Context, item release) error {
	connection, err := amqp.DialConfig(lab.brokerURL, amqp.Config{Dial: amqp.DefaultDial(5 * time.Second)})
	if err != nil {
		return err
	}
	defer connection.Close()
	// Close the dedicated connection if this request times out mid-delivery.
	stop := context.AfterFunc(ctx, func() { _ = connection.CloseDeadline(time.Now()) })
	defer stop()
	channel, err := connection.Channel()
	if err != nil {
		return err
	}
	queue, err := channel.QueueDeclare("", false, true, true, false, nil)
	if err != nil {
		return err
	}
	if err := channel.Publish("", queue.Name, true, false, amqp.Publishing{Body: []byte(item.ID)}); err != nil {
		return err
	}
	// The synchronous get follows the publish on the same channel and proves
	// that the broker routed the job to this rehearsal's isolated queue.
	delivery, found, err := channel.Get(queue.Name, false)
	if err != nil {
		return err
	}
	if !found || string(delivery.Body) != item.ID {
		return errors.New("RabbitMQ delivery did not match the release")
	}
	return delivery.Ack(false)
}

func (lab *releaseLab) broadcast(ctx context.Context, item release) error {
	connection, err := lab.events.Connect()
	if err != nil {
		return err
	}
	defer connection.Close()
	subject := "release." + item.ID
	sub, err := connection.SubscribeSync(subject)
	if err != nil {
		return err
	}
	if err := connection.Publish(subject, []byte(item.ID)); err != nil {
		return err
	}
	message, err := sub.NextMsgWithContext(ctx)
	if err != nil {
		return err
	}
	if string(message.Data) != item.ID {
		return errors.New("NATS event did not match the release")
	}
	return nil
}

func (lab *releaseLab) record(ctx context.Context, item release) error {
	_, err := lab.audit.ExecContext(ctx, "INSERT INTO release_audit (id) VALUES (?)", item.ID)
	return err
}

func (lab *releaseLab) index(ctx context.Context, item release) error {
	var task struct {
		UID int `json:"taskUid"`
	}
	path := lab.searchURL + "/indexes/releases/documents?primaryKey=id"
	document := map[string]string{"id": item.ID, "notes": item.Notes}
	if err := lab.request(ctx, http.MethodPost, path, lab.searchKey, []map[string]string{document}, &task); err != nil {
		return err
	}
	ticker := time.NewTicker(200 * time.Millisecond)
	defer ticker.Stop()
	for {
		var progress struct {
			Status string `json:"status"`
		}
		path := fmt.Sprintf("%s/tasks/%d", lab.searchURL, task.UID)
		if err := lab.request(ctx, http.MethodGet, path, lab.searchKey, nil, &progress); err != nil {
			return err
		}
		switch progress.Status {
		case "succeeded":
			return nil
		case "failed", "canceled":
			return errors.New("Meilisearch rejected indexing")
		}
		select {
		case <-ctx.Done():
			return ctx.Err()
		case <-ticker.C:
		}
	}
}

func (lab *releaseLab) search(w http.ResponseWriter, r *http.Request) {
	query := strings.TrimSpace(r.URL.Query().Get("q"))
	if query == "" || len(query) > 120 {
		http.Error(w, "Use a search query between 1 and 120 bytes", http.StatusBadRequest)
		return
	}
	path := lab.searchURL + "/indexes/releases/search?" + url.Values{"q": {query}}.Encode()
	var results struct {
		Hits []struct {
			ID    string `json:"id"`
			Notes string `json:"notes"`
		} `json:"hits"`
	}
	err := lab.request(r.Context(), http.MethodGet, path, lab.searchKey, nil, &results)
	if err != nil {
		http.Error(w, "Could not search releases. Rehearse a release to populate the index.", http.StatusServiceUnavailable)
		return
	}
	reply(w, http.StatusOK, results.Hits)
}

func (lab *releaseLab) notify(ctx context.Context, item release) error {
	message := map[string]any{
		"From":    map[string]string{"Email": "rehearsal@example.test"},
		"To":      []map[string]string{{"Email": "reviewer@example.test"}},
		"Subject": "Release rehearsal " + item.ID,
		"Text":    "Local rehearsal receipt for " + item.ID + "\n\n" + item.Notes,
	}
	return lab.request(ctx, http.MethodPost, lab.mailURL+"/api/v1/send", "", message, nil)
}

func (lab *releaseLab) request(ctx context.Context, method, endpoint, token string, input, output any) error {
	var body io.Reader
	if input != nil {
		encoded, err := json.Marshal(input)
		if err != nil {
			return err
		}
		body = bytes.NewReader(encoded)
	}
	request, err := http.NewRequestWithContext(ctx, method, endpoint, body)
	if err != nil {
		return err
	}
	if input != nil {
		request.Header.Set("Content-Type", "application/json")
	}
	if token != "" {
		request.Header.Set("Authorization", "Bearer "+token)
	}
	response, err := lab.httpClient.Do(request)
	if err != nil {
		return err
	}
	defer response.Body.Close()
	if response.StatusCode < 200 || response.StatusCode >= 300 {
		return fmt.Errorf("local service returned HTTP %d", response.StatusCode)
	}
	if output != nil {
		return json.NewDecoder(io.LimitReader(response.Body, 1<<20)).Decode(output)
	}
	_, err = io.Copy(io.Discard, io.LimitReader(response.Body, 1<<20))
	return err
}
