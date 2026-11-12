using TaskBoard.Data;
namespace TaskBoard.Models;

public sealed record BoardModel(IReadOnlyList<Purchase> Requests, string Status, string Query);
public sealed record DetailModel(Purchase Request, IReadOnlyList<PurchaseEvent> Events);
