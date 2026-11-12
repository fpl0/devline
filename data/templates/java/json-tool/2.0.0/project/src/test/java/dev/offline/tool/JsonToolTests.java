package dev.offline.tool;

import static org.junit.jupiter.api.Assertions.*;

import java.math.BigDecimal;
import java.util.List;
import org.junit.jupiter.api.Test;

class JsonToolTests {
  JsonTool.Invoice invoice(String ref, String amount) {
    return new JsonTool.Invoice(ref, "EUR", new BigDecimal(amount));
  }

  JsonTool.Payment payment(String id, String ref, String currency, String amount) {
    return new JsonTool.Payment(id, ref, currency, new BigDecimal(amount));
  }

  @Test
  void reconcilesPartialExcessAndUnknownPaymentsWithoutRounding() {
    var result =
        JsonTool.reconcile(
            List.of(
                invoice("A", "10.30"),
                invoice("B", "12.00"),
                invoice("C", "5.00"),
                invoice("D", "2.00")),
            List.of(
                payment("1", "A", "EUR", "10.10"),
                payment("2", "A", "EUR", "0.20"),
                payment("3", "B", "EUR", "2.00"),
                payment("4", "C", "EUR", "6.00"),
                payment("5", "X", "EUR", "1.00"),
                payment("6", "D", "USD", "2.00")));
    assertEquals(
        List.of("paid", "partial", "overpaid", "unpaid"),
        result.invoices().stream().map(JsonTool.Balance::status).toList());
    assertEquals(new BigDecimal("0.00"), result.invoices().getFirst().balance());
    assertEquals(
        List.of("unknown_reference", "currency_mismatch"),
        result.unmatched().stream().map(JsonTool.Unmatched::reason).toList());
  }

  @Test
  void rejectsDuplicateReferencesAndPayments() {
    assertThrows(
        IllegalArgumentException.class,
        () -> JsonTool.reconcile(List.of(invoice("A", "1"), invoice("A", "2")), List.of()));
    assertThrows(
        IllegalArgumentException.class,
        () ->
            JsonTool.reconcile(
                List.of(), List.of(payment("1", "A", "EUR", "1"), payment("1", "B", "EUR", "1"))));
  }

  @Test
  void parsesQuotedCsvAndRejectsMalformedRows() {
    assertEquals(
        List.of("A,B", "quoted \"name\"", "12"),
        JsonTool.csv("\"A,B\",\"quoted \"\"name\"\"\",12"));
    assertThrows(IllegalArgumentException.class, () -> JsonTool.csv("\"unfinished"));
    assertThrows(IllegalArgumentException.class, () -> JsonTool.csv("\"valid\"bad"));
  }

  @Test
  void enforcesCurrencyPrecision() {
    for (String bad : List.of("0", "-1", "1.001", "NaN", "1e2"))
      assertThrows(IllegalArgumentException.class, () -> JsonTool.money(bad));
    assertEquals(new BigDecimal("0.10"), JsonTool.money("0.1"));
  }
}
