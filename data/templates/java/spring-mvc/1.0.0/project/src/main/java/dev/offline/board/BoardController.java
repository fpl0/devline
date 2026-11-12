package dev.offline.board;

import jakarta.servlet.http.HttpServletResponse;
import java.math.BigDecimal;
import java.security.Principal;
import java.util.Map;
import org.springframework.security.core.Authentication;
import org.springframework.stereotype.Controller;
import org.springframework.ui.Model;
import org.springframework.web.bind.annotation.*;
import org.springframework.web.server.ResponseStatusException;

@Controller
class BoardController {
  private final PurchaseStore store;

  BoardController(PurchaseStore store) {
    this.store = store;
  }

  @GetMapping("/health")
  @ResponseBody
  Map<String, String> health() {
    store.list("", "");
    return Map.of("status", "ready");
  }

  @GetMapping("/")
  String index(
      @RequestParam(defaultValue = "") String status,
      @RequestParam(defaultValue = "") String q,
      Principal principal,
      Model model) {
    var requests = store.list(status, q);
    model.addAttribute("requests", requests);
    model.addAttribute("status", status);
    model.addAttribute("q", q);
    model.addAttribute("user", principal.getName());
    model.addAttribute(
        "pending", requests.stream().filter(r -> r.status().equals("pending")).count());
    model.addAttribute(
        "pendingAmount",
        BigDecimal.valueOf(
            requests.stream()
                .filter(r -> r.status().equals("pending"))
                .mapToLong(PurchaseStore.Purchase::amountCents)
                .sum(),
            2));
    return "index";
  }

  @PostMapping("/requests/create")
  String create(
      Principal principal,
      @RequestParam String title,
      @RequestParam String reason,
      @RequestParam String amount) {
    return "redirect:/requests/" + store.create(principal.getName(), title, reason, amount).id();
  }

  @GetMapping("/requests/{id}")
  String detail(@PathVariable long id, Authentication auth, Model model) {
    model.addAttribute("request", store.get(id));
    model.addAttribute("events", store.events(id));
    model.addAttribute("user", auth.getName());
    model.addAttribute(
        "approver",
        auth.getAuthorities().stream().anyMatch(a -> a.getAuthority().equals("ROLE_APPROVER")));
    return "detail";
  }

  @PostMapping("/requests/{id}/decision")
  String decide(
      @PathVariable long id,
      Authentication auth,
      @RequestParam long version,
      @RequestParam String decision,
      @RequestParam String note) {
    store.decide(
        id,
        auth.getName(),
        auth.getAuthorities().stream().anyMatch(a -> a.getAuthority().equals("ROLE_APPROVER")),
        version,
        decision,
        note);
    return "redirect:/requests/" + id;
  }

  @ExceptionHandler(ResponseStatusException.class)
  String error(ResponseStatusException error, Model model, HttpServletResponse response) {
    response.setStatus(error.getStatusCode().value());
    model.addAttribute("message", error.getReason());
    return "error";
  }
}
