# Distributed systems

Reliable delivery requires an explicit acknowledgement policy. At-least-once delivery may repeat messages, so consumers should make processing idempotent.

## Reading questions

What is the transaction boundary? How can a worker recover after a crash? Which observations survive a restart?
