# Filtering tasks

> This guide shows you how to use query parameters to filter tasks and obtain a more readable list of asynchronous operations.

Querying the [get tasks endpoint](https://www.meilisearch.com/docs/reference/api/tasks/list-tasks) returns all tasks that have not been deleted. This unfiltered list may be difficult to parse in large projects.

This guide shows you how to use query parameters to filter tasks and obtain a more readable list of asynchronous operations.

> [!TIP]
>
> Filtering batches with [the `/batches` route](https://www.meilisearch.com/docs/reference/api/batches/list-batches) follows the same rules as filtering tasks. Keep in mind that many `/batches` parameters such as `uids` target the tasks included in batches, instead of the batches themselves.

## Filtering tasks with a single parameter

Use the get tasks endpoint to fetch all `canceled` tasks:

```bash
curl \
  -X GET 'MEILISEARCH_URL/tasks?statuses=failed' \
  -H 'Authorization: Bearer API_KEY'
```

```javascript
client.tasks.getTasks({ statuses: ['failed', 'canceled'] })
```

```python
client.get_tasks({'statuses': ['failed', 'canceled']})
```

```php
$client->getTasks((new TasksQuery())->setStatuses(['failed', 'canceled']));
```

```java
TasksQuery query = new TasksQuery().setStatuses(new String[] {"failed", "canceled"});
client.getTasks(query);
```

```ruby
client.get_tasks(statuses: ['failed', 'canceled'])
```

```go
client.GetTasks(&meilisearch.TasksQuery{
  Statuses: []meilisearch.TaskStatus{
    meilisearch.TaskStatusFailed,
    meilisearch.TaskStatusCanceled,
  },
})
```

```csharp
await client.GetTasksAsync(new TasksQuery { Statuses = new List<TaskInfoStatus> { TaskInfoStatus.Failed, TaskInfoStatus.Canceled } });
```

```rust
let mut query = TasksQuery::new(&client);
let tasks = query
  .with_statuses(["failed"])
  .execute()
  .await
  .unwrap();
```

```swift
client.getTasks(params: TasksQuery(statuses: [.failed, .canceled])) { result in
  switch result {
  case .success(let taskResult):
    print(taskResult)
  case .failure(let error):
    print(error)
  }
}
```

```dart
await client.getTasks(
  params: TasksQuery(
    statuses: ['failed', 'canceled'],
  ),
);
```

Use a comma to separate multiple values and fetch both `canceled` and `failed` tasks:

```bash
curl \
  -X GET 'MEILISEARCH_URL/tasks?statuses=failed,canceled' \
  -H 'Authorization: Bearer API_KEY'
```

```rust
let mut query = TasksQuery::new(&client);
let tasks = query
  .with_statuses(["failed", "canceled"])
  .execute()
  .await
  .unwrap();
```

You may filter tasks based on `uid`, `status`, `type`, `indexUid`, `canceledBy`, or date. Consult the [API reference](https://www.meilisearch.com/docs/reference/api/tasks/list-tasks) for a full list of task filtering parameters.

## Available filter parameters

All parameters below accept comma-separated values. Filters of different types are combined with a logical `AND`.

| Parameter | Description |
| - | - |
| `uids` | Filter tasks by their `uid`. Separate multiple `uids` with a comma (`,`). |
| `batchUids` | Filter tasks by their `batchUid`. Separate multiple `batchUids` with a comma (`,`). |
| `statuses` | Filter tasks by their `status`: `enqueued`, `processing`, `succeeded`, `failed`, or `canceled`. |
| `types` | Filter tasks by their `type`, for example `documentAdditionOrUpdate` or `indexDeletion`. |
| `indexUids` | Filter tasks by their `indexUid`. Case-sensitive. |
| `canceledBy` | Filter tasks by their `canceledBy` field. Separate multiple task `uids` with a comma (`,`). |

### Date filters

Use the following parameters to filter tasks by one of their timestamp fields. All values must be valid RFC 3339 dates.

| Parameter | Filters tasks whose... |
| - | - |
| `beforeEnqueuedAt` | `enqueuedAt` is earlier than the provided date |
| `afterEnqueuedAt` | `enqueuedAt` is later than the provided date |
| `beforeStartedAt` | `startedAt` is earlier than the provided date |
| `afterStartedAt` | `startedAt` is later than the provided date |
| `beforeFinishedAt` | `finishedAt` is earlier than the provided date |
| `afterFinishedAt` | `finishedAt` is later than the provided date |

> [!NOTE]
>
> Date filters are equivalent to `<` or `>` operations. There is currently no way to perform `≤` or `≥` comparisons with a date filter.

### Pagination parameters

Combine the filters above with the following parameters to paginate results.

| Parameter | Description |
| - | - |
| `limit` | Number of tasks to return. Defaults to `20`. |
| `from` | `uid` of the first task returned. Defaults to the `uid` of the last created task. |
| `reverse` | If `true`, returns results in reverse order, from oldest to most recent. Defaults to `false`. |

## Combining filters

Use the ampersand character (`&`) to combine filters, equivalent to a logical `AND`:

```bash
curl \
  -X GET 'MEILISEARCH_URL/tasks?indexUids=movies&types=documentAdditionOrUpdate,documentDeletion&statuses=processing' \
  -H 'Authorization: Bearer API_KEY'
```

```javascript
client.tasks.getTasks({
  indexUids: ['movies'],
  types: ['documentAdditionOrUpdate','documentDeletion'],
  statuses: ['processing']
})
```

```python
client.get_tasks(
  {
      'indexUids': 'movies',
      'types': ['documentAdditionOrUpdate', 'documentDeletion'],
      'statuses': ['processing'],
  }
)
```

```php
$client->getTasks(
  (new TasksQuery())
    ->setStatuses(['processing'])
    ->setUids(['movies'])
    ->setTypes(['documentAdditionOrUpdate', 'documentDeletion'])
);
```

```java
TasksQuery query =
        new TasksQuery()
                .setStatuses(new String[] {"processing"})
                .setTypes(new String[] {"documentAdditionOrUpdate", "documentDeletion"})
                .setIndexUids(new String[] {"movies"});

client.getTasks(query);
```

```ruby
client.get_tasks(index_uids: ['movies'], types: ['documentAdditionOrUpdate', 'documentDeletion'], statuses: ['processing'])
```

```go
client.GetTasks(&meilisearch.TasksQuery{
  IndexUIDS: []string{"movie"},
  Types:     []meilisearch.TaskType{
    meilisearch.TaskTypeDocumentAdditionOrUpdate,
    meilisearch.TaskTypeDocumentDeletion,
  },
  Statuses:  []meilisearch.TaskStatus{
    meilisearch.TaskStatusProcessing,
  },
})
```

```csharp
var query = new TasksQuery { IndexUids = new List<string> { "movies" }, Types = new List<TaskInfo> { TaskInfo.DocumentAdditionOrUpdate, TaskInfo.DocumentDeletion }, Statuses = new List<TaskInfoStatus> { TaskInfoStatus.Processing } };

await client.GetTasksAsync(query);
```

```rust
let mut query = TasksQuery::new(&client);
let tasks = query
  .with_index_uids(["movies"])
  .with_types(["documentAdditionOrUpdate","documentDeletion"])
  .with_statuses(["processing"])
  .execute()
  .await
  .unwrap();
```

```swift
client.getTasks(params: TasksQuery(indexUids: "movies", types: ["documentAdditionOrUpdate", "documentDeletion"], statuses: ["processing"])) { result in
  switch result {
  case .success(let taskResult):
    print(taskResult)
  case .failure(let error):
    print(error)
  }
}
```

```dart
await client.getTasks(
  params: TasksQuery(
    indexUids: ['movies'],
    types: ['documentAdditionOrUpdate', 'documentDeletion'],
    statuses: ['processing'],
  ),
);
```

This code sample returns all tasks in the `movies` index that have the type `documentAdditionOrUpdate` or `documentDeletion` and have the `status` of `processing`.

> [!WARNING]
>
> **`OR` operations between different filters are not supported.** For example, you cannot view tasks which have a type of `documentAddition` **or** a status of `failed`.

## Next steps

> [card] [Monitor tasks](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/monitor_tasks)
>
> Check the status of asynchronous operations in real time.

> [card] [Manage the task database](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/manage_task_database)
>
> Navigate long task lists with pagination and query parameters.

> [card] [Asynchronous operations](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/async_operations)
>
> Understand how Meilisearch processes tasks in the background.


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.