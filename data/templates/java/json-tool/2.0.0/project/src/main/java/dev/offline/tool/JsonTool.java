package dev.offline.tool;

import java.math.BigDecimal;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.*;
import java.util.concurrent.Callable;
import picocli.CommandLine;
import picocli.CommandLine.Command;
import picocli.CommandLine.Option;
import tools.jackson.databind.json.JsonMapper;

@Command(
    name = "reconcile",
    mixinStandardHelpOptions = true,
    description = "Reconcile invoice balances against payments.")
public final class JsonTool implements Callable<Integer> {
  @Option(names = "--invoices", required = true)
  Path invoices;

  @Option(names = "--payments", required = true)
  Path payments;

  @Option(names = "--output", required = true)
  Path output;

  public record Invoice(String reference, String currency, BigDecimal amount) {}

  public record Payment(String id, String reference, String currency, BigDecimal amount) {}

  public record Balance(
      String reference,
      String currency,
      BigDecimal invoiced,
      BigDecimal paid,
      BigDecimal balance,
      String status) {}

  public record Unmatched(Payment payment, String reason) {}

  public record Totals(BigDecimal invoiced, BigDecimal received) {}

  public record Report(
      List<Balance> invoices, List<Unmatched> unmatched, Map<String, Totals> totals) {}

  public static void main(String[] args) {
    System.exit(new CommandLine(new JsonTool()).execute(args));
  }

  @Override
  public Integer call() {
    try {
      var report = reconcile(readInvoices(invoices), readPayments(payments));
      Path destination = output.toAbsolutePath();
      Files.createDirectories(destination.getParent());
      Path temporary = Files.createTempFile(destination.getParent(), ".reconcile-", ".json");
      try {
        Files.writeString(
            temporary,
            JsonMapper.builder().build().writerWithDefaultPrettyPrinter().writeValueAsString(report)
                + "\n");
        Files.move(
            temporary,
            destination,
            StandardCopyOption.ATOMIC_MOVE,
            StandardCopyOption.REPLACE_EXISTING);
      } finally {
        Files.deleteIfExists(temporary);
      }
      System.out.printf(
          "Reconciled %d invoices; %d unmatched payments. Report: %s%n",
          report.invoices.size(), report.unmatched.size(), output);
      return 0;
    } catch (Exception error) {
      System.err.println("reconcile: " + error.getMessage());
      return 2;
    }
  }

  public static Report reconcile(List<Invoice> invoices, List<Payment> payments) {
    var byReference = new LinkedHashMap<String, Invoice>();
    var paid = new HashMap<String, BigDecimal>();
    var totals = new TreeMap<String, Totals>();
    for (var invoice : invoices) {
      if (byReference.putIfAbsent(invoice.reference, invoice) != null)
        throw new IllegalArgumentException("Duplicate invoice reference: " + invoice.reference);
      var old = totals.getOrDefault(invoice.currency, new Totals(BigDecimal.ZERO, BigDecimal.ZERO));
      totals.put(invoice.currency, new Totals(old.invoiced.add(invoice.amount), old.received));
    }
    var ids = new HashSet<String>();
    var unmatched = new ArrayList<Unmatched>();
    for (var payment : payments) {
      if (!ids.add(payment.id))
        throw new IllegalArgumentException("Duplicate payment id: " + payment.id);
      var old = totals.getOrDefault(payment.currency, new Totals(BigDecimal.ZERO, BigDecimal.ZERO));
      totals.put(payment.currency, new Totals(old.invoiced, old.received.add(payment.amount)));
      var invoice = byReference.get(payment.reference);
      if (invoice == null) unmatched.add(new Unmatched(payment, "unknown_reference"));
      else if (!invoice.currency.equals(payment.currency))
        unmatched.add(new Unmatched(payment, "currency_mismatch"));
      else paid.merge(invoice.reference, payment.amount, BigDecimal::add);
    }
    var balances =
        invoices.stream()
            .map(i -> {
              var received = paid.getOrDefault(i.reference, BigDecimal.ZERO.setScale(2));
              var balance = i.amount.subtract(received);
              String status =
                  received.signum() == 0
                      ? "unpaid"
                      : balance.signum() > 0
                          ? "partial"
                          : balance.signum() < 0 ? "overpaid" : "paid";
              return new Balance(i.reference, i.currency, i.amount, received, balance, status);
            })
            .toList();
    return new Report(balances, List.copyOf(unmatched), Collections.unmodifiableMap(totals));
  }

  static List<Invoice> readInvoices(Path path) throws Exception {
    return rows(path, List.of("reference", "currency", "amount")).stream()
        .map(r -> new Invoice(identifier(r.get(0)), currency(r.get(1)), money(r.get(2))))
        .toList();
  }

  static List<Payment> readPayments(Path path) throws Exception {
    return rows(path, List.of("id", "reference", "currency", "amount")).stream()
        .map(r ->
            new Payment(
                identifier(r.get(0)), identifier(r.get(1)), currency(r.get(2)), money(r.get(3))))
        .toList();
  }

  static String identifier(String value) {
    if (value.isBlank() || value.length() > 100)
      throw new IllegalArgumentException("Identifiers must contain 1–100 characters");
    return value;
  }

  static String currency(String value) {
    if (!value.matches("[A-Z]{3}"))
      throw new IllegalArgumentException("Currency must contain three uppercase letters: " + value);
    return value;
  }

  static BigDecimal money(String value) {
    if (!value.matches("[0-9]+(\\.[0-9]{1,2})?"))
      throw new IllegalArgumentException("Invalid positive two-decimal amount: " + value);
    var amount = new BigDecimal(value).setScale(2);
    if (amount.signum() <= 0) throw new IllegalArgumentException("Amount must be positive");
    return amount;
  }

  static List<List<String>> rows(Path path, List<String> header) throws Exception {
    var lines = Files.readAllLines(path, StandardCharsets.UTF_8);
    if (lines.isEmpty() || !csv(lines.getFirst()).equals(header))
      throw new IllegalArgumentException(path + ": expected header " + String.join(",", header));
    var result = new ArrayList<List<String>>();
    for (int n = 1; n < lines.size(); n++) {
      if (lines.get(n).isBlank()) continue;
      try {
        var row = csv(lines.get(n));
        if (row.size() != header.size())
          throw new IllegalArgumentException("Wrong number of columns");
        result.add(row);
      } catch (IllegalArgumentException e) {
        throw new IllegalArgumentException(path + ":" + (n + 1) + ": " + e.getMessage());
      }
    }
    return result;
  }

  static List<String> csv(String line) {
    var fields = new ArrayList<String>();
    var value = new StringBuilder();
    boolean quoted = false, closed = false;
    for (int i = 0; i < line.length(); i++) {
      char c = line.charAt(i);
      if (quoted) {
        if (c == '"') {
          if (i + 1 < line.length() && line.charAt(i + 1) == '"') {
            value.append('"');
            i++;
          } else {
            quoted = false;
            closed = true;
          }
        } else value.append(c);
      } else if (c == ',') {
        fields.add(value.toString().strip());
        value.setLength(0);
        closed = false;
      } else if (c == '"' && value.isEmpty() && !closed) quoted = true;
      else if (c == '"' || closed && !Character.isWhitespace(c))
        throw new IllegalArgumentException("Malformed CSV quoting");
      else if (!closed) value.append(c);
    }
    if (quoted)
      throw new IllegalArgumentException("Unclosed CSV quote; multiline fields are not supported");
    fields.add(value.toString().strip());
    return fields;
  }
}
