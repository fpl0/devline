package dev.offline.board;

import java.util.List;

import org.springframework.data.annotation.Id;
import org.springframework.data.jdbc.repository.query.Modifying;
import org.springframework.data.jdbc.repository.query.Query;
import org.springframework.data.relational.core.mapping.Table;
import org.springframework.data.repository.CrudRepository;

@Table("board_task")
record BoardTask(@Id Long id, String owner, String title, boolean completed, boolean published, boolean recorded) {}

interface TaskRepository extends CrudRepository<BoardTask, Long> {
    List<BoardTask> findAllByOwnerOrderById(String owner);

    @Modifying
    @Query("UPDATE board_task SET completed = true WHERE id = :id AND owner = :owner")
    int complete(long id, String owner);

    @Modifying
    @Query("DELETE FROM board_task WHERE id = :id AND owner = :owner")
    int remove(long id, String owner);

    @Query("SELECT * FROM board_task WHERE completed AND NOT published ORDER BY id LIMIT 20")
    List<BoardTask> pendingCompletions();

    @Modifying
    @Query("UPDATE board_task SET published = true WHERE id = :id")
    void markPublished(long id);

    @Query("""
            UPDATE board_task SET recorded = true
            WHERE id = :id AND completed
            RETURNING owner
            """)
    String recordCompletion(long id);
}
