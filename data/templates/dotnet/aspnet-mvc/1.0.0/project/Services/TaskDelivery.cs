using System.Text;
using Microsoft.EntityFrameworkCore;
using RabbitMQ.Client;
using TaskBoard.Data;

namespace TaskBoard.Services;

public sealed record BrokerOptions(Uri Address)
{
    public const string Queue = "devline-dotnet.tasks.created";
}

public sealed class TaskDelivery(
    BrokerOptions options,
    IServiceScopeFactory scopes,
    ILogger<TaskDelivery> logger) : BackgroundService
{
    public bool Connected => connection is { IsOpen: true };
    private volatile IConnection? connection;

    protected override async Task ExecuteAsync(CancellationToken stoppingToken)
    {
        var factory = new ConnectionFactory
        {
            Uri = options.Address,
            AutomaticRecoveryEnabled = false,
            RequestedConnectionTimeout = TimeSpan.FromSeconds(5),
            ClientProvidedName = "devline-dotnet:task-board"
        };

        while (!stoppingToken.IsCancellationRequested)
        {
            try
            {
                await using var broker = await factory.CreateConnectionAsync(stoppingToken);
                await using var channel = await broker.CreateChannelAsync(new CreateChannelOptions(
                    publisherConfirmationsEnabled: true, publisherConfirmationTrackingEnabled: true), stoppingToken);
                await channel.QueueDeclareAsync(BrokerOptions.Queue, durable: true, exclusive: false,
                    autoDelete: false, cancellationToken: stoppingToken);
                connection = broker;

                while (broker.IsOpen && channel.IsOpen && !stoppingToken.IsCancellationRequested)
                {
                    await using var scope = scopes.CreateAsyncScope();
                    var database = scope.ServiceProvider.GetRequiredService<TaskDatabase>();
                    var tasks = scope.ServiceProvider.GetRequiredService<TaskStore>();
                    await PublishPendingAsync(database, channel, stoppingToken);
                    await ConsumeBatchAsync(tasks, channel, stoppingToken);
                    await Task.Delay(500, stoppingToken);
                }
            }
            catch (OperationCanceledException) when (stoppingToken.IsCancellationRequested)
            {
                break;
            }
            catch (Exception error)
            {
                logger.LogWarning("Task delivery will retry after {ErrorType}.", error.GetType().Name);
            }
            finally
            {
                connection = null;
            }

            try
            {
                await Task.Delay(TimeSpan.FromSeconds(2), stoppingToken);
            }
            catch (OperationCanceledException) when (stoppingToken.IsCancellationRequested)
            {
                break;
            }
        }
    }

    private static async Task PublishPendingAsync(
        TaskDatabase database, IChannel channel, CancellationToken cancellation)
    {
        var pending = await database.Tasks.Where(row => row.PublishedAt == null)
            .OrderBy(row => row.CreatedAt).Take(20).Select(row => row.Id).ToArrayAsync(cancellation);
        foreach (var id in pending)
        {
            using var timeout = CancellationTokenSource.CreateLinkedTokenSource(cancellation);
            timeout.CancelAfter(TimeSpan.FromSeconds(5));
            var properties = new BasicProperties
            {
                ContentType = "text/plain",
                MessageId = id.ToString(),
                Type = "TaskCreated",
                Persistent = true
            };
            await channel.BasicPublishAsync("", BrokerOptions.Queue, mandatory: true, properties,
                Encoding.UTF8.GetBytes(id.ToString()), timeout.Token);
            // Mark the task only after the broker confirms the publish.
            await database.Tasks.Where(row => row.Id == id)
                .ExecuteUpdateAsync(update => update.SetProperty(row => row.PublishedAt, DateTime.UtcNow), cancellation);
        }
    }

    private static async Task ConsumeBatchAsync(TaskStore tasks, IChannel channel, CancellationToken cancellation)
    {
        // The demo uses a single worker and bounded batches to keep the delivery loop easy to follow.
        for (var count = 0; count < 20; count++)
        {
            var message = await channel.BasicGetAsync(BrokerOptions.Queue, autoAck: false, cancellation);
            if (message is null)
                return;

            if (!Guid.TryParse(Encoding.UTF8.GetString(message.Body.Span), out var id))
            {
                await channel.BasicRejectAsync(message.DeliveryTag, requeue: false, cancellation);
                continue;
            }

            // A database failure closes the channel and RabbitMQ can redeliver the unacknowledged message.
            await tasks.MarkDeliveredAsync(id, cancellation);
            await channel.BasicAckAsync(message.DeliveryTag, multiple: false, cancellation);
        }
    }
}
