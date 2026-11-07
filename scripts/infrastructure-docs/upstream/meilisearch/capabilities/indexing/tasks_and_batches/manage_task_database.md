# Managing the task database

> Meilisearch uses a task queue to handle asynchronous operations. This document describes how to navigate long task queues with filters and pagination.

By default, Meilisearch returns a list of 20 tasks for each request when you query the [get tasks endpoint](https://www.meilisearch.com/docs/reference/api/tasks/list-tasks). This guide shows you how to navigate the task list using query parameters.

> [!TIP]
>
> Paginating batches with [the `/batches` route](https://www.meilisearch.com/docs/reference/api/batches/list-batches) follows the same rules as paginating tasks.

## Configuring the number of returned tasks

Use the `limit` parameter to change the number of returned tasks:

```bash
curl \
  -X GET 'MEILISEARCH_URL/tasks?limit=2&from=10 \
  -H 'Authorization: Bearer API_KEY'
```

```javascript
client.tasks.getTasks({ limit: 2, from: 10 })
```

```python
client.get_tasks({
  'limit': 2,
  'from': 10
})
```

```php
$taskQuery = (new TasksQuery())->setLimit(2)->setFrom(10));
$client->getTasks($taskQuery);
```

```java
TasksQuery query = new TasksQuery()
      .setLimit(2)
      .setFrom(10);

client.index("movies").getTasks(query);
```

```ruby
client.tasks(limit: 2, from: 10)
```

```go
client.GetTasks(&meilisearch.TasksQuery{
  Limit: 2,
  From: 10,
});
```

```csharp
ResourceResults<TaskInfo> taskResult = await client.GetTasksAsync(new TasksQuery { Limit = 2, From = 10 });
```

```rust
let mut query = TasksSearchQuery::new(&client)
    .with_limit(2)
    .with_from(10)
    .execute()
    .await
    .unwrap();
```

```swift
client.getTasks(params: TasksQuery(limit: 2, from: 10)) { result in
  switch result {
  case .success(let taskResult):
    print(taskResult)
  case .failure(let error):
    print(error)
  }
}
```

```dart
await client.getTasks(params: TasksQuery(limit: 2, from: 10));
```

Meilisearch will return a batch of tasks. Each batch of returned tasks is often called a "page" of tasks, and the size of that page is determined by `limit`:

```json
{
  "results": [
    …
  ],
  "total": 50,
  "limit": 2,
  "from": 10,
  "next": 8
}
```

It is possible none of the returned tasks are the ones you are looking for. In that case, you will need to use the [get all tasks request response](https://www.meilisearch.com/docs/reference/api/tasks/list-tasks) to navigate the results.

## Navigating the task list with `from` and `next`

Use the `next` value included in the response to your previous query together with `from` to fetch the next set of results:

```bash
curl \
  -X GET 'MEILISEARCH_URL/tasks?limit=2&from=8 \
  -H 'Authorization: Bearer API_KEY'
```

```javascript
client.tasks.getTasks({ limit: 2, from: 8 })
```

```python
client.get_tasks({
  'limit': 2,
  'from': 8
})
```

```php
$taskQuery = (new TasksQuery())->setLimit(2)->setFrom(8));
$client->getTasks($taskQuery);
```

```java
TasksQuery query = new TasksQuery()
      .setLimit(2)
      .setFrom(8);

client.index("movies").getTasks(query);
```

```ruby
client.tasks(limit: 2, from: 8)
```

```go
client.GetTasks(&meilisearch.TasksQuery{
  Limit: 2,
  From: 8,
});
```

```csharp
ResourceResults<TaskInfo> taskResult = await client.GetTasksAsync(new TasksQuery { Limit = 2, From = 8 });
```

```rust
let mut query = TasksSearchQuery::new(&client)
    .with_limit(2)
    .from(8)
    .execute()
    .await
    .unwrap();
```

```swift
client.getTasks(params: TasksQuery(limit: 2, from: 8)) { result in
  switch result {
  case .success(let taskResult):
    print(taskResult)
  case .failure(let error):
    print(error)
  }
}
```

```dart
await client.getTasks(params: TasksQuery(limit: 2, from: 8));
```

This will return a new batch of tasks:

```json
{
  "results": [
    …
  ],
  "total": 50,
  "limit": 2,
  "from": 8,
  "next": 6
}
```

When the value of `next` is `null`, you have reached the final set of results.

> [!TIP]
>
> Use `from` and `limit` together with task filtering parameters to navigate filtered task lists.

## Deleting tasks from the database

Use the [delete tasks endpoint](https://www.meilisearch.com/docs/reference/api/tasks/delete-tasks) to remove tasks from the database based on `uid`, `status`, `type`, `indexUid`, `canceledBy`, or date.

> [!WARNING]
>
> You can only delete **finished** tasks (`succeeded`, `failed`, or `canceled`). `enqueued` and `processing` tasks cannot be deleted: cancel them first using the [cancel tasks endpoint](https://www.meilisearch.com/docs/reference/api/tasks/cancel-tasks).

Task deletion is an atomic transaction: either all matched tasks are successfully deleted, or none are.

## Automatic task cleanup

Meilisearch stores up to **1 million tasks** in the task database. If enqueuing a new task would exceed this limit, Meilisearch automatically attempts to delete the oldest **100,000 finished tasks** to make room.

If there are no finished tasks in the database, Meilisearch does not delete anything and enqueues the new task as usual. This automatic cleanup keeps the task database bounded without interrupting ongoing work.

## Next steps

> [card] [Filter tasks](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/filter_tasks)
>
> Use query parameters to filter tasks by status, type, and more.

> [card] [Monitor tasks](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/monitor_tasks)
>
> Check the status of asynchronous operations in real time.

> [card] [Asynchronous operations](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/async_operations)
>
> Understand how Meilisearch processes tasks in the background.


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.