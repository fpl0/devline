package dev.offline.spring;

import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.boot.webmvc.test.autoconfigure.AutoConfigureMockMvc;
import org.springframework.test.web.servlet.MockMvc;

import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.delete;
import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.get;
import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.post;
import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.put;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.header;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.jsonPath;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.status;

@SpringBootTest
@AutoConfigureMockMvc
class OfflineSpringApplicationTests {
    @Autowired
    MockMvc http;

    @Test
    void createsReadsReplacesAndDeletesATask() throws Exception {
        var response = http.perform(post("/api/tasks")
                        .contentType("application/json").content("""
                                {"title":"  Read a chapter  "}
                                """))
                .andExpect(status().isCreated())
                .andExpect(header().exists("Location"))
                .andExpect(jsonPath("$.title").value("Read a chapter"))
                .andExpect(jsonPath("$.completed").value(false))
                .andReturn().getResponse();
        String task = response.getHeader("Location");

        http.perform(get(task)).andExpect(status().isOk());
        http.perform(put(task).contentType("application/json").content("""
                        {"title":"Read two chapters","completed":true}
                        """))
                .andExpect(status().isOk())
                .andExpect(jsonPath("$.title").value("Read two chapters"))
                .andExpect(jsonPath("$.completed").value(true));
        http.perform(get("/api/tasks")).andExpect(jsonPath("$[0].title").value("Read two chapters"));
        http.perform(delete(task)).andExpect(status().isNoContent());
        http.perform(get(task)).andExpect(status().isNotFound());
    }

    @Test
    void returnsAProblemForAnInvalidTitle() throws Exception {
        http.perform(post("/api/tasks").contentType("application/json").content("""
                        {"title":"  "}
                        """))
                .andExpect(status().isBadRequest())
                .andExpect(jsonPath("$.detail").value("title must be 1–120 characters"));
    }
}
