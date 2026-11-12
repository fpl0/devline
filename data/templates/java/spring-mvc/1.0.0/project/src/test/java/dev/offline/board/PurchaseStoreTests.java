package dev.offline.board;

import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.web.server.ResponseStatusException;

@SpringBootTest(properties = "spring.datasource.url=jdbc:h2:mem:purchase-tests;DB_CLOSE_DELAY=-1")
class PurchaseStoreTests {
  @Autowired PurchaseStore store;

  @Test
  void requiresIndependentApprovalAndMakesDecisionsFinal() {
    var request = store.create("alice", "Keyboard", "Replace broken keyboard", "120.50");
    assertEquals(12050, request.amountCents());
    assertThrows(
        ResponseStatusException.class,
        () -> store.decide(request.id(), "alice", true, 1, "approved", "Needed"));
    assertThrows(
        ResponseStatusException.class,
        () -> store.decide(request.id(), "bob", false, 1, "approved", "Needed"));
    store.decide(request.id(), "bob", true, 1, "approved", "Within budget");
    assertEquals("approved", store.get(request.id()).status());
    assertEquals(2, store.events(request.id()).size());
    assertThrows(
        ResponseStatusException.class,
        () -> store.decide(request.id(), "bob", true, 2, "rejected", "Changed my mind"));
  }

  @Test
  void rejectsInvalidMoneyAndMissingReasons() {
    for (String amount : new String[] {"0", "-1", "1.001", "1000000.01", "NaN"})
      assertThrows(
          ResponseStatusException.class, () -> store.create("alice", "A", "Reason", amount));
    assertThrows(ResponseStatusException.class, () -> store.create("alice", "A", " ", "10.00"));
  }
}
