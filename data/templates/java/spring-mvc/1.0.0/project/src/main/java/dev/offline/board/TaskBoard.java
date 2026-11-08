package dev.offline.board;

import java.time.Duration;
import java.util.List;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.dao.DataAccessException;
import org.springframework.data.redis.core.StringRedisTemplate;
import org.springframework.http.HttpStatus;
import org.springframework.stereotype.Service;
import org.springframework.web.server.ResponseStatusException;
import tools.jackson.core.JacksonException;
import tools.jackson.core.type.TypeReference;
import tools.jackson.databind.json.JsonMapper;

@Service
class TaskBoard {
    private static final Logger LOG = LoggerFactory.getLogger(TaskBoard.class);
    private static final Duration CACHE_TTL = Duration.ofSeconds(30);
    private static final TypeReference<List<BoardTask>> TASKS = new TypeReference<>() {};

    private final TaskRepository tasks;
    private final StringRedisTemplate redis;
    private final JsonMapper json;

    TaskBoard(TaskRepository tasks, StringRedisTemplate redis, JsonMapper json) {
        this.tasks = tasks;
        this.redis = redis;
        this.json = json;
    }

    record Snapshot(List<BoardTask> tasks, String source) {}

    // Keep cache fills and invalidations ordered within this local app process.
    synchronized Snapshot list(String owner) {
        String key = cacheKey(owner);
        try {
            String cached = redis.opsForValue().get(key);
            if (cached != null) {
                return new Snapshot(json.readValue(cached, TASKS), "redis");
            }
        } catch (DataAccessException | JacksonException error) {
            LOG.debug("Task cache read failed", error);
        }
        var rows = tasks.findAllByOwnerOrderById(owner);
        try {
            redis.opsForValue().set(key, json.writeValueAsString(rows), CACHE_TTL);
        } catch (DataAccessException | JacksonException error) {
            LOG.debug("Task cache fill failed", error);
        }
        return new Snapshot(rows, "postgres");
    }

    synchronized void add(String owner, String rawTitle) {
        String title = rawTitle.strip();
        if (title.isEmpty() || title.codePointCount(0, title.length()) > 120) {
            throw new IllegalArgumentException("Enter a title between 1 and 120 characters.");
        }
        tasks.save(new BoardTask(null, owner, title, false, false, false));
        evict(owner);
    }

    synchronized void complete(String owner, long id) {
        requireTask(tasks.complete(id, owner));
        evict(owner);
    }

    synchronized void delete(String owner, long id) {
        requireTask(tasks.remove(id, owner));
        evict(owner);
    }

    synchronized void recordCompletion(long id) {
        String owner = tasks.recordCompletion(id);
        if (owner != null) {
            evict(owner);
        }
    }

    private static void requireTask(int affected) {
        if (affected == 0) {
            throw new ResponseStatusException(HttpStatus.NOT_FOUND, "Task not found");
        }
    }

    private void evict(String owner) {
        try {
            redis.delete(cacheKey(owner));
        } catch (DataAccessException error) {
            LOG.debug("Task cache invalidation failed", error);
        }
    }

    private static String cacheKey(String owner) {
        return "devline:java:task-board:v1:" + owner;
    }
}
