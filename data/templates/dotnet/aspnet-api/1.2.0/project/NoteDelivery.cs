using System.Text;
using Microsoft.EntityFrameworkCore;
using RabbitMQ.Client;
using RabbitMQ.Client.Events;

namespace OfflineApi;

// One worker per app, with separate publisher/consumer channels. A failed publish stays
// in the database; confirmed messages are acknowledged only after the database update.
public sealed class NoteDelivery(IServiceScopeFactory scopes, QueueOptions options, ILogger<NoteDelivery> logger) : BackgroundService
{
    private IConnection? connection;
    private volatile bool consuming;
    private string state = options.Enabled ? "connecting" : "not configured";
    public string State => Volatile.Read(ref state);

    public async Task<bool> ProbeAsync(CancellationToken cancellation)
    {
        var current = connection;
        if (!consuming || current is not { IsOpen: true }) return false;
        try
        {
            using var timeout = CancellationTokenSource.CreateLinkedTokenSource(cancellation);
            timeout.CancelAfter(TimeSpan.FromSeconds(2));
            await using var channel = await current.CreateChannelAsync(cancellationToken: timeout.Token);
            await channel.QueueDeclarePassiveAsync(options.Name, cancellationToken: timeout.Token);
            return true;
        }
        catch { return false; }
    }

    protected override async Task ExecuteAsync(CancellationToken stoppingToken)
    {
        if (!options.Enabled) return;
        while (!stoppingToken.IsCancellationRequested)
        {
            try
            {
                var factory = new ConnectionFactory
                {
                    Uri = new Uri(options.ConnectionString!),
                    AutomaticRecoveryEnabled = false,
                    RequestedConnectionTimeout = TimeSpan.FromSeconds(3),
                    ClientProvidedName = "offline-api:notes"
                };
                await using var current = await factory.CreateConnectionAsync(stoppingToken);
                connection = current;
                await using var publisher = await current.CreateChannelAsync(new CreateChannelOptions(
                    publisherConfirmationsEnabled: true, publisherConfirmationTrackingEnabled: true), stoppingToken);
                await using var consumerChannel = await current.CreateChannelAsync(cancellationToken: stoppingToken);
                await publisher.QueueDeclareAsync(options.Name, durable: true, exclusive: false, autoDelete: false, cancellationToken: stoppingToken);
                await consumerChannel.BasicQosAsync(0, 1, false, stoppingToken);
                var consumer = new AsyncEventingBasicConsumer(consumerChannel);
                consumer.ReceivedAsync += async (_, message) =>
                {
                    try
                    {
                        // Copy/parse the body while the callback owns it.
                        if (!Guid.TryParse(Encoding.UTF8.GetString(message.Body.Span), out var id))
                        {
                            await consumerChannel.BasicNackAsync(message.DeliveryTag, false, false, stoppingToken);
                            return;
                        }
                        using var scope = scopes.CreateScope();
                        await scope.ServiceProvider.GetRequiredService<NoteService>().MarkProcessedAsync(id, stoppingToken);
                        await consumerChannel.BasicAckAsync(message.DeliveryTag, false, stoppingToken);
                    }
                    catch (OperationCanceledException) when (stoppingToken.IsCancellationRequested) { }
                    catch (Exception error)
                    {
                        Volatile.Write(ref state, "delivery paused; retrying");
                        logger.LogWarning("Note delivery failed ({ErrorType}); the message will be retried.", error.GetType().Name);
                        if (consumerChannel.IsOpen)
                            await consumerChannel.BasicNackAsync(message.DeliveryTag, false, true, stoppingToken);
                    }
                };
                await consumerChannel.BasicConsumeAsync(options.Name, autoAck: false, consumer: consumer, cancellationToken: stoppingToken);
                consuming = true;
                Volatile.Write(ref state, "connected");
                while (!stoppingToken.IsCancellationRequested && current.IsOpen && publisher.IsOpen && consumerChannel.IsOpen)
                {
                    using var scope = scopes.CreateScope();
                    var database = scope.ServiceProvider.GetRequiredService<NotesDb>();
                    var pending = await database.Notes.AsNoTracking()
                        .Where(note => note.QueueRequested && note.PublishedAtUtc == null)
                        .OrderBy(note => note.CreatedAtUtc).Take(20).Select(note => note.Id).ToListAsync(stoppingToken);
                    foreach (var id in pending)
                    {
                        using var timeout = CancellationTokenSource.CreateLinkedTokenSource(stoppingToken);
                        timeout.CancelAfter(TimeSpan.FromSeconds(5));
                        var properties = new BasicProperties { Persistent = true, MessageId = id.ToString(), ContentType = "text/plain" };
                        // Awaiting a confirmed publish can throw. Leave the outbox pending on failure.
                        await publisher.BasicPublishAsync("", options.Name, mandatory: true, properties,
                            Encoding.UTF8.GetBytes(id.ToString()), timeout.Token);
                        await database.Notes.Where(note => note.Id == id).ExecuteUpdateAsync(
                            update => update.SetProperty(note => note.PublishedAtUtc, DateTime.UtcNow), stoppingToken);
                    }
                    await Task.Delay(250, stoppingToken);
                }
            }
            catch (OperationCanceledException) when (stoppingToken.IsCancellationRequested) { break; }
            catch (Exception error)
            {
                logger.LogWarning("RabbitMQ is unavailable ({ErrorType}); saved notes remain pending and will be retried.", error.GetType().Name);
            }
            finally
            {
                consuming = false;
                connection = null;
                Volatile.Write(ref state, "unavailable; retrying saved notes");
            }
            try { await Task.Delay(2000, stoppingToken); }
            catch (OperationCanceledException) when (stoppingToken.IsCancellationRequested) { break; }
        }
    }
}
