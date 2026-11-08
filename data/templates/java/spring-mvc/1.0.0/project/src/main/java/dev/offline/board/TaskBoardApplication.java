package dev.offline.board;

import java.io.IOException;
import java.util.Map;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.scheduling.annotation.EnableScheduling;

@EnableScheduling
@SpringBootApplication
public class TaskBoardApplication {
    public static void main(String[] args) throws IOException, InterruptedException {
        var application = new SpringApplication(TaskBoardApplication.class);
        application.setDefaultProperties(Map.of("TASK_BOARD_CLIENT_SECRET", KeycloakDemo.prepare()));
        application.run(args);
    }
}
