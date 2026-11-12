namespace InvoiceReconciliation;

internal static class ReconciliationTests
{
    public static void Run()
    {
        var report = Reconciler.Reconcile([new("A", "EUR", 10.30m), new("B", "EUR", 12m), new("C", "EUR", 5m), new("D", "EUR", 2m)], [new("1", "A", "EUR", 10.10m), new("2", "A", "EUR", .20m), new("3", "B", "EUR", 2m), new("4", "C", "EUR", 6m), new("5", "X", "EUR", 1m), new("6", "D", "USD", 2m)]);
        Check(report.Invoices.Select(i => i.Status).SequenceEqual(new[] { "paid", "partial", "overpaid", "unpaid" }), "payment states");
        Check(report.Invoices[0].Outstanding == 0, "exact arithmetic");
        Check(report.Unmatched.Select(u => u.Reason).SequenceEqual(new[] { "unknown_reference", "currency_mismatch" }), "unmatched classification");
        Throws(() => Reconciler.Reconcile([new("A", "EUR", 1), new("A", "EUR", 2)], []));
        Throws(() => Reconciler.Reconcile([], [new("1", "A", "EUR", 1), new("1", "B", "EUR", 1)]));
        Check(Csv.Parse("\"A,B\",\"quoted \"\"name\"\"\",12").SequenceEqual(new[] { "A,B", "quoted \"name\"", "12" }), "quoted CSV");
        Throws(() => Csv.Parse("\"unfinished"));
        Throws(() => Csv.Parse("\"valid\"bad"));
        foreach (var bad in new[] { "0", "-1", "1.001", "NaN", "1e2" })
            Throws(() => Csv.Money(bad));
        Console.WriteLine("Domain checks passed: settlement states, precision, duplicate detection, unmatched currencies and CSV parsing.");
    }
    static void Check(bool condition, string name)
    {
        if (!condition)
            throw new InvalidOperationException($"Test failed: {name}");
    }
    static void Throws(Action action)
    {
        try
        {
            action();
        }
        catch (ArgumentException) { return; }
        throw new InvalidOperationException("Expected invalid input to be rejected");
    }
}
