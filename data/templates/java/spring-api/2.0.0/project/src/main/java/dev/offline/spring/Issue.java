package dev.offline.spring;

public record Issue(
    long id,
    String title,
    String description,
    String priority,
    String status,
    long version,
    String createdAt,
    String updatedAt) {}
