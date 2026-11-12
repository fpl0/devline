using System.Text.Json;
using InvoiceReconciliation;

try
{
    if (args.Length == 1 && args[0] == "--self-test")
    {
        ReconciliationTests.Run();
        return 0;
    }
    if (args.Length == 1 && args[0] is "--help" or "-h")
    {
        Console.WriteLine("reconcile --invoices FILE --payments FILE --output FILE");
        return 0;
    }
    if (args.Length != 6)
        throw new ArgumentException("Use --invoices FILE --payments FILE --output FILE");
    var options = new Dictionary<string, string>();
    for (var i = 0; i < args.Length; i += 2)
    if (!options.TryAdd(args[i], args[i + 1]))
        throw new ArgumentException("Duplicate option");
    if (options.Keys.Any(k => k is not ("--invoices" or "--payments" or "--output")))
        throw new ArgumentException("Unknown option");
    var report = Reconciler.Reconcile(Csv.Invoices(options["--invoices"]), Csv.Payments(options["--payments"]));
    var destination = Path.GetFullPath(options["--output"]);
    Directory.CreateDirectory(Path.GetDirectoryName(destination)!);
    var temporary = destination + "." + Guid.NewGuid() + ".tmp";
    try
    {
        File.WriteAllText(temporary, JsonSerializer.Serialize(report, new JsonSerializerOptions { WriteIndented = true, PropertyNamingPolicy = JsonNamingPolicy.CamelCase }) + "\n");
        File.Move(temporary, destination, true);
    }
    finally { if (File.Exists(temporary)) File.Delete(temporary); }
    Console.WriteLine($"Reconciled {report.Invoices.Count} invoices; {report.Unmatched.Count} unmatched payments. Report: {destination}");
    return 0;
}
catch (Exception error) when (error is ArgumentException or IOException or FormatException or OverflowException or KeyNotFoundException)
{
    Console.Error.WriteLine($"reconcile: {error.Message}");
    return 2;
}
