package dev.offline.board;

import java.util.Map;

import jakarta.servlet.http.HttpServletResponse;
import org.springframework.http.HttpStatus;
import org.springframework.security.core.annotation.AuthenticationPrincipal;
import org.springframework.security.oauth2.core.oidc.user.OidcUser;
import org.springframework.stereotype.Controller;
import org.springframework.ui.Model;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.ResponseBody;
import org.springframework.web.servlet.mvc.support.RedirectAttributes;
import org.springframework.web.servlet.view.RedirectView;

@Controller
class BoardController {
    private final TaskBoard board;

    BoardController(TaskBoard board) {
        this.board = board;
    }

    @GetMapping("/")
    String home() {
        return "home";
    }

    @GetMapping("/health")
    @ResponseBody
    Map<String, String> health() {
        return Map.of("status", "up");
    }

    @GetMapping("/tasks")
    String tasks(@AuthenticationPrincipal OidcUser user, Model model, HttpServletResponse response) {
        var snapshot = board.list(user.getSubject());
        model.addAttribute("username", user.getPreferredUsername());
        model.addAttribute("tasks", snapshot.tasks());
        model.addAttribute("source", snapshot.source());
        response.setHeader("X-Task-Source", snapshot.source());
        return "board";
    }

    @PostMapping("/tasks")
    RedirectView add(@AuthenticationPrincipal OidcUser user, @RequestParam String title,
                     RedirectAttributes feedback) {
        try {
            board.add(user.getSubject(), title);
        } catch (IllegalArgumentException error) {
            feedback.addFlashAttribute("error", error.getMessage());
        }
        return backToBoard();
    }

    @PostMapping("/tasks/{id}/complete")
    RedirectView complete(@AuthenticationPrincipal OidcUser user, @PathVariable long id) {
        board.complete(user.getSubject(), id);
        return backToBoard();
    }

    @PostMapping("/tasks/{id}/delete")
    RedirectView delete(@AuthenticationPrincipal OidcUser user, @PathVariable long id) {
        board.delete(user.getSubject(), id);
        return backToBoard();
    }

    private static RedirectView backToBoard() {
        var redirect = new RedirectView("/tasks", true);
        redirect.setStatusCode(HttpStatus.SEE_OTHER);
        return redirect;
    }
}
