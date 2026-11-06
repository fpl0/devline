package dev.offline.spring;

import java.util.Map;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RestController;

@RestController
class HomeController {
    @GetMapping("/")
    Map<String, String> home() {
        return Map.of("status", "offline");
    }
}
