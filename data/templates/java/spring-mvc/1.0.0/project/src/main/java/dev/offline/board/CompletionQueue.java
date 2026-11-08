package dev.offline.board;

import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.amqp.AmqpException;
import org.springframework.amqp.AmqpRejectAndDontRequeueException;
import org.springframework.amqp.core.MessageDeliveryMode;
import org.springframework.amqp.core.Queue;
import org.springframework.amqp.core.QueueBuilder;
import org.springframework.amqp.rabbit.annotation.RabbitListener;
import org.springframework.amqp.rabbit.connection.CorrelationData;
import org.springframework.amqp.rabbit.core.RabbitTemplate;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.scheduling.annotation.Scheduled;

@Configuration(proxyBeanMethods = false)
class CompletionQueue {
    static final String NAME = "devline.java.task-completed";
    private static final Logger LOG = LoggerFactory.getLogger(CompletionQueue.class);

    private final TaskRepository tasks;
    private final TaskBoard board;
    private final RabbitTemplate broker;

    CompletionQueue(TaskRepository tasks, TaskBoard board, RabbitTemplate broker) {
        this.tasks = tasks;
        this.board = board;
        this.broker = broker;
    }

    @Bean
    Queue completions() {
        return QueueBuilder.durable(NAME).build();
    }

    // A completed row is the outbox. A broker outage leaves it eligible for retry.
    @Scheduled(fixedDelay = 2000)
    void publish() {
        for (BoardTask task : tasks.pendingCompletions()) {
            try {
                var delivery = new CorrelationData();
                broker.convertAndSend("", NAME, task.id().toString(), message -> {
                    message.getMessageProperties().setDeliveryMode(MessageDeliveryMode.PERSISTENT);
                    return message;
                }, delivery);
                var confirmation = delivery.getFuture().get(2, TimeUnit.SECONDS);
                if (!confirmation.ack() || delivery.getReturned() != null) {
                    LOG.warn("Completion was not routed; the next poll will retry");
                    return;
                }
                tasks.markPublished(task.id());
            } catch (InterruptedException error) {
                Thread.currentThread().interrupt();
                return;
            } catch (AmqpException | ExecutionException | TimeoutException error) {
                LOG.warn("Completion was not confirmed; the next poll will retry", error);
                return;
            }
        }
    }

    @RabbitListener(queues = NAME)
    void consume(String message) {
        final long id;
        try {
            id = Long.parseLong(message);
        } catch (NumberFormatException error) {
            throw new AmqpRejectAndDontRequeueException("Expected a numeric task ID", error);
        }
        // The listener acknowledges after the database update. Repeated delivery is safe.
        board.recordCompletion(id);
    }
}
