use crate::task::{Priority, Task};
use anyhow::{Context, Result};
use rusqlite::{params, Connection, Row};

#[derive(Debug)]
pub struct DB {
    connection: Connection,
}

impl DB {
    /// Create and return a connection to a database located at path
    /// if path is an empty string creates and in memory db instance
    pub fn create_and_return_connection(path: &str) -> DB {
        let conn: Connection = if path.is_empty() {
            Connection::open_in_memory()
                .context("Can't open in-memory DB.")
                .unwrap()
        } else {
            Connection::open(path)
                .context("Can't open database")
                .unwrap()
        };
        let mut db = DB { connection: conn };
        db.init();
        db
    }

    fn init(&mut self) {
        self.connection
            .execute(
                "CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            completed BOOLEAN NOT NULL,
            priority INTEGER NOT NULL
        )",
                [],
            )
            .context("Can't create the DB")
            .unwrap();
    }

    pub fn insert_task(&self, task: &Task) {
        self.connection
            .execute(
                "INSERT INTO tasks (title, description, completed, priority) VALUES (?1,?2, 0, ?3)",
                params![
                    task.title.trim(),
                    task.description.trim(),
                    task.priority.to_usize()
                ],
            )
            .context("Can't add task to DB.")
            .unwrap();
    }

    pub fn get_all_tasks(&self) -> Vec<Task> {
        let mut stmt = self
            .connection
            .prepare("SELECT id, title, description, completed, priority FROM tasks")
            .unwrap();
        let task_row_iter = stmt
            .query_map([], |row| {
                Task::try_from(row)
            })
            .context("Can't get results from DB.")
            .unwrap();
        let mut tasks = Vec::new();
        for task in task_row_iter {
            tasks.push(task.unwrap());
        }
        tasks
    }

    pub fn get_task_by_id(&self, task_id: i32) -> Result<Task> {
        let mut stmt = self.connection.prepare(
            "SELECT id, title, description, completed, priority FROM tasks where id = ?1",
        )?;
        stmt.query_row(params![task_id], |row| {
            Task::try_from(row)
        })
        .with_context(|| format!("Couldn't get task at index {task_id}"))
    }

    pub fn get_all_task_by_highest_priority(&self) -> Vec<Task> {
        let mut stmt = self
            .connection
            .prepare("SELECT id, title, description, completed, priority FROM tasks order by priority desc")
            .unwrap();
        let task_row_iter = stmt
            .query_map([], |row| {
                Task::try_from(row)
            })
            .context("Couldn't get results from DB.")
            .unwrap();
        let mut tasks = Vec::new();
        for task in task_row_iter {
            tasks.push(task.unwrap());
        }
        tasks
    }

    pub fn get_all_task_by_lowest_priority(&self) -> Vec<Task> {
        let mut stmt = self
            .connection
            .prepare("SELECT id, title, description, completed, priority FROM tasks order by priority asc")
            .unwrap();
        let task_row_iter = stmt
            .query_map([], |row| {
                Task::try_from(row)
            })
            .context("Couldn't get results from DB.")
            .unwrap();
        let mut tasks = Vec::new();
        for task in task_row_iter {
            tasks.push(task.unwrap());
        }
        tasks
    }

    pub fn toggle_task_completed(&self, task_id: i32, completed: bool) -> usize {
        let completed = match completed {
            true => 1,
            false => 0,
        };
        self.connection
            .execute(
                "UPDATE tasks SET completed = ?2 WHERE id = ?1",
                params![task_id, completed],
            )
            .context("Can't update the task completed property.")
            .unwrap()
    }

    pub fn update_task_priority(&self, task_id: i32, priority: Priority) -> usize {
        self.connection
            .execute(
                "UPDATE tasks SET priority = ?2 WHERE id = ?1",
                params![task_id, priority as u8],
            )
            .context("Can't update the task priority property.")
            .unwrap()
    }

    pub fn update_task(&self, task: &Task) -> usize {
        self.connection
            .execute(
                "UPDATE tasks SET title = ?2, description = ?3 WHERE id = ?1",
                params![task.id, task.title, task.description],
            )
            .context("Can't update the task.")
            .unwrap()
    }

    pub fn delete_task(&self, task_id: i32) -> usize {
        self.connection
            .execute("delete from tasks where id = ?1", params![task_id])
            .context("Can't delete the task.")
            .unwrap()
    }

    pub fn get_record_count(&self) -> i64 {
        let query = "SELECT count(*) FROM tasks";
        self.connection
            .query_row(query, [], |r| r.get(0))
            .context("Can't get record count")
            .unwrap()
    }

    pub fn clear(&self) -> usize {
        self.connection
            .execute("DELETE FROM tasks", [])
            .context("Can't clear database")
            .unwrap()
    }
}

impl TryFrom<&Row<'_>> for Task {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> rusqlite::Result<Self, Self::Error> {
        Ok(Task {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            completed: row.get(3)?,
            priority: Priority::from_u8(row.get(4)?).unwrap(),
            date: Some(String::from("30-09-24")),
        })
    }
}
