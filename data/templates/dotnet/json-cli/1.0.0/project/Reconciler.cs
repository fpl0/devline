using System.Globalization;
using System.Text;
using System.Text.RegularExpressions;
namespace InvoiceReconciliation;

public sealed record Invoice(string Reference, string Currency, decimal Amount);
public sealed record Payment(string Id, string Reference, string Currency, decimal Amount);
public sealed record Balance(string Reference, string Currency, decimal Invoiced, decimal Paid, decimal Outstanding, string Status);
public sealed record Unmatched(Payment Payment, string Reason);
public sealed record Totals(decimal Invoiced, decimal Received);
public sealed record Report(IReadOnlyList<Balance> Invoices, IReadOnlyList<Unmatched> Unmatched, IReadOnlyDictionary<string, Totals> Totals);

public static class Reconciler
{
    public static Report Reconcile(IReadOnlyList<Invoice> invoices, IReadOnlyList<Payment> payments)
    {
        var lookup = new Dictionary<string, Invoice>(StringComparer.Ordinal);
        var paid = new Dictionary<string, decimal>();
        var totals = new SortedDictionary<string, Totals>(StringComparer.Ordinal);
        foreach (var invoice in invoices)
        {
            if (!lookup.TryAdd(invoice.Reference, invoice))
                throw new ArgumentException($"Duplicate invoice reference: {invoice.Reference}");
            var old = totals.GetValueOrDefault(invoice.Currency, new Totals(0, 0));
            totals[invoice.Currency] = old with
            {
                Invoiced = old.Invoiced + invoice.Amount
            };
        }
        var ids = new HashSet<string>();
        var unmatched = new List<Unmatched>();
        foreach (var payment in payments)
        {
            if (!ids.Add(payment.Id))
                throw new ArgumentException($"Duplicate payment id: {payment.Id}");
            var old = totals.GetValueOrDefault(payment.Currency, new Totals(0, 0));
            totals[payment.Currency] = old with
            {
                Received = old.Received + payment.Amount
            };
            if (!lookup.TryGetValue(payment.Reference, out var invoice))
                unmatched.Add(new(payment, "unknown_reference"));
            else if (invoice.Currency != payment.Currency)
                unmatched.Add(new(payment, "currency_mismatch"));
            else
                paid[payment.Reference] = paid.GetValueOrDefault(payment.Reference) + payment.Amount;
        }
        return new(invoices.Select(i => { var received = paid.GetValueOrDefault(i.Reference); var balance = i.Amount - received; return new Balance(i.Reference, i.Currency, i.Amount, received, balance, received == 0 ? "unpaid" : balance > 0 ? "partial" : balance < 0 ? "overpaid" : "paid"); }).ToList(), unmatched, totals);
    }
}
public static class Csv
{
    public static List<Invoice> Invoices(string path) => Rows(path, ["reference", "currency", "amount"]).Select(r => new Invoice(Id(r[0]), Currency(r[1]), Money(r[2]))).ToList();
    public static List<Payment> Payments(string path) => Rows(path, ["id", "reference", "currency", "amount"]).Select(r => new Payment(Id(r[0]), Id(r[1]), Currency(r[2]), Money(r[3]))).ToList();
    static string Id(string s) => s.Length is > 0 and <= 100 ? s : throw new ArgumentException("Identifiers must contain 1–100 characters");
    static string Currency(string s) => Regex.IsMatch(s, "^[A-Z]{3}$") ? s : throw new ArgumentException($"Invalid currency: {s}");
    public static decimal Money(string s)
    {
        if (!Regex.IsMatch(s, @"^[0-9]+(\.[0-9]{1,2})?$"))
            throw new ArgumentException($"Invalid positive two-decimal amount: {s}");
        var amount = decimal.Parse(s, CultureInfo.InvariantCulture);
        return amount > 0 ? amount : throw new ArgumentException("Amount must be positive");
    }
    static List<List<string>> Rows(string path, string[] header)
    {
        var lines = File.ReadAllLines(path, Encoding.UTF8);
        if (lines.Length == 0 || !Parse(lines[0]).SequenceEqual(header))
            throw new ArgumentException($"{path}: expected header {string.Join(',', header)}");
        var rows = new List<List<string>>();
        for (var i = 1; i < lines.Length; i++)
        {
            if (string.IsNullOrWhiteSpace(lines[i]))
                continue;
            try
            {
                var row = Parse(lines[i]);
                if (row.Count != header.Length)
                    throw new ArgumentException("Wrong number of columns");
                rows.Add(row);
            }
            catch (ArgumentException e) { throw new ArgumentException($"{path}:{i + 1}: {e.Message}"); }
        }
        return rows;
    }
    public static List<string> Parse(string line)
    {
        var fields = new List<string>();
        var value = new StringBuilder();
        var quoted = false;
        var closed = false;
        for (var i = 0; i < line.Length; i++)
        {
            var c = line[i];
            if (quoted)
            {
                if (c == '"')
                {
                    if (i + 1 < line.Length && line[i + 1] == '"')
                    {
                        value.Append('"');
                        i++;
                    }
                    else
                    {
                        quoted = false;
                        closed = true;
                    }
                }
                else
                    value.Append(c);
            }
            else if (c == ',')
            {
                fields.Add(value.ToString().Trim());
                value.Clear();
                closed = false;
            }
            else if (c == '"' && value.Length == 0 && !closed)
                quoted = true;
            else if (c == '"' || closed && !char.IsWhiteSpace(c))
                throw new ArgumentException("Malformed CSV quoting");
            else if (!closed)
                value.Append(c);
        }
        if (quoted)
            throw new ArgumentException("Unclosed CSV quote; multiline fields are not supported");
        fields.Add(value.ToString().Trim());
        return fields;
    }
}
