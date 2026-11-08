using System.ComponentModel.DataAnnotations;

namespace TaskBoard.Models;

public sealed record TaskRow(Guid Id, string Title, bool Completed, DateTime? DeliveredAt);

public sealed record TaskBoardModel(IReadOnlyList<TaskRow> Tasks);

public sealed class TaskInput
{
    [Required, StringLength(160)]
    public string Title { get; init; } = "";
}
