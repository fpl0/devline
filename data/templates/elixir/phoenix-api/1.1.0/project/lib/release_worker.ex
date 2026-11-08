defmodule ReleaseDesk.Worker do
  @moduledoc "Dispatches PostgreSQL outbox rows and consumes RabbitMQ jobs."
  use GenServer

  alias AMQP.{Basic, Channel, Confirm, Connection, Queue}
  alias ReleaseDesk.{Delivery, Store}
  require Logger

  @queue "elixir-release-desk"

  def start_link(_opts), do: GenServer.start_link(__MODULE__, [], name: __MODULE__)

  @impl true
  def init([]) do
    Store.migrate!()
    {:ok, connection} = Connection.open(System.fetch_env!("DEVLINE_RABBITMQ_URL"))
    Process.monitor(connection.pid)
    {:ok, channel} = Channel.open(connection)
    Process.monitor(channel.pid)
    {:ok, _} = Queue.declare(channel, @queue, durable: true)
    :ok = Confirm.select(channel)
    :ok = Basic.qos(channel, prefetch_count: 1)
    {:ok, _} = Basic.consume(channel, @queue)
    Process.send_after(self(), :dispatch, 500)
    {:ok, %{channel: channel, connection: connection}}
  end

  @impl true
  def handle_info(:dispatch, state) do
    try do
      Enum.each(Store.unpublished(), fn id ->
        :ok = Basic.publish(state.channel, "", @queue, id, persistent: true)
        true = Confirm.wait_for_confirms(state.channel, 5)
        Store.published(id)
      end)
    rescue
      _error -> Logger.warning("Release dispatch failed. Pending rows will be retried.")
    end

    Process.send_after(self(), :dispatch, 500)
    {:noreply, state}
  end

  def handle_info({:basic_deliver, id, %{delivery_tag: tag}}, state) do
    try do
      case Store.release(id) do
        %{status: "queued"} = release ->
          Delivery.ship(release)
          Store.complete(id)

        _ ->
          :ok
      end

      :ok = Basic.ack(state.channel, tag)
    rescue
      _error ->
        Logger.warning("A release step failed. Retrying the queued job.")
        Process.send_after(self(), {:retry, tag}, 1_000)
    end

    {:noreply, state}
  end

  def handle_info({:retry, tag}, state) do
    :ok = Basic.nack(state.channel, tag, requeue: true)
    {:noreply, state}
  end

  def handle_info({:DOWN, _reference, :process, _pid, _reason}, state),
    do: {:stop, :rabbitmq_connection_closed, state}

  def handle_info({:basic_consume_ok, _meta}, state), do: {:noreply, state}
  def handle_info({:basic_cancel, _meta}, state), do: {:stop, :rabbitmq_consumer_closed, state}

  @impl true
  def terminate(_reason, state) do
    if Process.alive?(state.connection.pid), do: Connection.close(state.connection)
    :ok
  end
end
