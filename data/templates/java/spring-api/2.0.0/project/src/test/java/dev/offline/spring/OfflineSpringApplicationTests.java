package dev.offline.spring;

import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.*;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.*;

import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.boot.webmvc.test.autoconfigure.AutoConfigureMockMvc;
import org.springframework.test.web.servlet.MockMvc;

@SpringBootTest(properties = "spring.datasource.url=jdbc:h2:mem:issue-tests;DB_CLOSE_DELAY=-1")
@AutoConfigureMockMvc
class OfflineSpringApplicationTests {
  @Autowired MockMvc http;

  @Test
  void enforcesWorkflowAndOptimisticConcurrency() throws Exception {
    var response =
        http.perform(
                post("/api/issues")
                    .contentType("application/json")
                    .content(
                        "{\"title\":\" Offline indexing \",\"description\":\"Reproduce on a fresh"
                            + " cache\",\"priority\":\"high\"}"))
            .andExpect(status().isCreated())
            .andExpect(jsonPath("$.title").value("Offline indexing"))
            .andExpect(jsonPath("$.version").value(1))
            .andReturn()
            .getResponse();
    String url = response.getHeader("Location");
    http.perform(
            put(url)
                .contentType("application/json")
                .content(
                    "{\"title\":\"Offline"
                        + " indexing\",\"priority\":\"high\",\"status\":\"resolved\",\"version\":1}"))
        .andExpect(status().isConflict());
    String update =
        "{\"title\":\"Offline"
            + " indexing\",\"priority\":\"high\",\"status\":\"in_progress\",\"version\":1}";
    http.perform(put(url).contentType("application/json").content(update))
        .andExpect(status().isOk())
        .andExpect(jsonPath("$.version").value(2));
    http.perform(put(url).contentType("application/json").content(update))
        .andExpect(status().isConflict());
    http.perform(get(url + "/events")).andExpect(jsonPath("$.length()").value(2));
    http.perform(get("/api/issues?q=indexing&status=in_progress"))
        .andExpect(jsonPath("$[0].title").value("Offline indexing"));
  }

  @Test
  void rejectsInvalidInputAndMissingResources() throws Exception {
    http.perform(
            post("/api/issues")
                .contentType("application/json")
                .content("{\"title\":\" \",\"priority\":\"high\"}"))
        .andExpect(status().isBadRequest())
        .andExpect(jsonPath("$.detail").exists());
    http.perform(get("/api/issues?limit=1000")).andExpect(status().isBadRequest());
    http.perform(get("/api/issues/9999999")).andExpect(status().isNotFound());
  }
}
