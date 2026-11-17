use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use rusqlite::{params, Connection, Result};
use crate::models::{Board, BoardObject};

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

pub fn init_db(db_path: &Path) -> Result<Connection> {
    if let Some(parent) = db_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let conn = Connection::open(db_path)?;
    conn.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS boards (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            pan_x REAL NOT NULL DEFAULT 0.0,
            pan_y REAL NOT NULL DEFAULT 0.0,
            zoom REAL NOT NULL DEFAULT 1.0,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS board_objects (
            id TEXT PRIMARY KEY,
            board_id TEXT NOT NULL,
            kind TEXT NOT NULL,
            x REAL NOT NULL,
            y REAL NOT NULL,
            width REAL NOT NULL,
            height REAL NOT NULL,
            z_index INTEGER NOT NULL DEFAULT 0,
            title TEXT,
            content TEXT NOT NULL,
            meta TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (board_id) REFERENCES boards(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_board_objects_board ON board_objects(board_id);
        CREATE INDEX IF NOT EXISTS idx_board_objects_kind ON board_objects(kind);
        "#,
    )?;

    // Insert default board if none exists
    let now = now_millis();
    conn.execute(
        "INSERT OR IGNORE INTO boards (id, name, pan_x, pan_y, zoom, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params!["default", "Main Board", 0.0, 0.0, 1.0, now],
    )?;

    Ok(conn)
}

pub fn load_board(conn: &Connection, board_id: &str) -> Result<(Board, Vec<BoardObject>)> {
    let board = conn.query_row(
        "SELECT id, name, pan_x, pan_y, zoom, updated_at FROM boards WHERE id = ?1",
        params![board_id],
        |row| {
            Ok(Board {
                id: row.get(0)?,
                name: row.get(1)?,
                pan_x: row.get(2)?,
                pan_y: row.get(3)?,
                zoom: row.get(4)?,
                updated_at: row.get(5)?,
            })
        },
    )?;

    let mut stmt = conn.prepare(
        "SELECT id, board_id, kind, x, y, width, height, z_index, title, content, meta, created_at, updated_at 
         FROM board_objects WHERE board_id = ?1 ORDER BY z_index ASC",
    )?;

    let object_iter = stmt.query_map(params![board_id], |row| {
        Ok(BoardObject {
            id: row.get(0)?,
            board_id: row.get(1)?,
            kind: row.get(2)?,
            x: row.get(3)?,
            y: row.get(4)?,
            width: row.get(5)?,
            height: row.get(6)?,
            z_index: row.get(7)?,
            title: row.get(8)?,
            content: row.get(9)?,
            meta: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?;

    let mut objects = Vec::new();
    for obj in object_iter {
        objects.push(obj?);
    }

    Ok((board, objects))
}

pub fn save_board_state(
    conn: &mut Connection,
    board_id: &str,
    pan_x: f64,
    pan_y: f64,
    zoom: f64,
    objects: &[BoardObject],
) -> Result<()> {
    let tx = conn.transaction()?;
    let now = now_millis();

    tx.execute(
        "UPDATE boards SET pan_x = ?1, pan_y = ?2, zoom = ?3, updated_at = ?4 WHERE id = ?5",
        params![pan_x, pan_y, zoom, now, board_id],
    )?;

    // Collect IDs present in current state
    if objects.is_empty() {
        tx.execute("DELETE FROM board_objects WHERE board_id = ?1", params![board_id])?;
    } else {
        // Delete objects not in incoming list
        let placeholders: Vec<String> = (0..objects.len()).map(|_| "?".to_string()).collect();
        let query = format!(
            "DELETE FROM board_objects WHERE board_id = ?1 AND id NOT IN ({})",
            placeholders.join(",")
        );
        let mut del_params: Vec<&dyn rusqlite::ToSql> = vec![&board_id];
        for obj in objects {
            del_params.push(&obj.id);
        }
        tx.execute(&query, del_params.as_slice())?;

        // Upsert incoming objects
        let mut stmt = tx.prepare(
            r#"
            INSERT INTO board_objects 
            (id, board_id, kind, x, y, width, height, z_index, title, content, meta, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(id) DO UPDATE SET
                kind = excluded.kind,
                x = excluded.x,
                y = excluded.y,
                width = excluded.width,
                height = excluded.height,
                z_index = excluded.z_index,
                title = excluded.title,
                content = excluded.content,
                meta = excluded.meta,
                updated_at = excluded.updated_at
            "#,
        )?;

        for obj in objects {
            let created = obj.created_at.unwrap_or(now);
            let updated = obj.updated_at.unwrap_or(now);
            stmt.execute(params![
                obj.id,
                board_id,
                obj.kind,
                obj.x,
                obj.y,
                obj.width,
                obj.height,
                obj.z_index,
                obj.title,
                obj.content,
                obj.meta,
                created,
                updated
            ])?;
        }
    }

    tx.commit()?;
    Ok(())
}

pub fn delete_objects(conn: &Connection, ids: &[String]) -> Result<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let placeholders: Vec<String> = (0..ids.len()).map(|_| "?".to_string()).collect();
    let query = format!(
        "DELETE FROM board_objects WHERE id IN ({})",
        placeholders.join(",")
    );
    let params: Vec<&dyn rusqlite::ToSql> = ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
    conn.execute(&query, params.as_slice())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            CREATE TABLE IF NOT EXISTS boards (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                pan_x REAL NOT NULL DEFAULT 0.0,
                pan_y REAL NOT NULL DEFAULT 0.0,
                zoom REAL NOT NULL DEFAULT 1.0,
                updated_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS board_objects (
                id TEXT PRIMARY KEY,
                board_id TEXT NOT NULL,
                kind TEXT NOT NULL,
                x REAL NOT NULL,
                y REAL NOT NULL,
                width REAL NOT NULL,
                height REAL NOT NULL,
                z_index INTEGER NOT NULL DEFAULT 0,
                title TEXT,
                content TEXT NOT NULL,
                meta TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                FOREIGN KEY (board_id) REFERENCES boards(id) ON DELETE CASCADE
            );
            "#,
        ).unwrap();
        conn.execute(
            "INSERT INTO boards (id, name, pan_x, pan_y, zoom, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params!["default", "Test Board", 10.0, 20.0, 1.5, 1000],
        ).unwrap();
        conn
    }

    #[test]
    fn test_save_and_load_board() {
        let mut conn = setup_test_db();
        let obj1 = BoardObject {
            id: "obj-1".to_string(),
            board_id: "default".to_string(),
            kind: "text".to_string(),
            x: 100.0,
            y: 150.0,
            width: 300.0,
            height: 200.0,
            z_index: 1,
            title: Some("Terminal Note".to_string()),
            content: "cargo build --release\n".to_string(),
            meta: Some("{\"font_size\": 14}".to_string()),
            created_at: Some(1000),
            updated_at: Some(1000),
        };
        let obj2 = BoardObject {
            id: "obj-2".to_string(),
            board_id: "default".to_string(),
            kind: "image".to_string(),
            x: 500.0,
            y: 200.0,
            width: 400.0,
            height: 300.0,
            z_index: 2,
            title: Some("Screenshot".to_string()),
            content: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==".to_string(),
            meta: Some("{\"natural_width\": 800}".to_string()),
            created_at: Some(1001),
            updated_at: Some(1001),
        };

        save_board_state(&mut conn, "default", 50.0, 60.0, 2.0, &[obj1.clone(), obj2.clone()]).unwrap();

        let (board, objects) = load_board(&conn, "default").unwrap();
        assert_eq!(board.id, "default");
        assert_eq!(board.pan_x, 50.0);
        assert_eq!(board.pan_y, 60.0);
        assert_eq!(board.zoom, 2.0);
        assert_eq!(objects.len(), 2);

        let loaded1 = &objects[0];
        assert_eq!(loaded1.id, "obj-1");
        assert_eq!(loaded1.content, "cargo build --release\n");
        assert_eq!(loaded1.title.as_deref(), Some("Terminal Note"));

        let loaded2 = &objects[1];
        assert_eq!(loaded2.id, "obj-2");
        assert_eq!(loaded2.kind, "image");
    }

    #[test]
    fn test_delete_and_upsert() {
        let mut conn = setup_test_db();
        let obj1 = BoardObject {
            id: "obj-1".to_string(),
            board_id: "default".to_string(),
            kind: "text".to_string(),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            z_index: 1,
            title: None,
            content: "Original".to_string(),
            meta: None,
            created_at: Some(100),
            updated_at: Some(100),
        };
        save_board_state(&mut conn, "default", 0.0, 0.0, 1.0, &[obj1]).unwrap();

        let (_, objects) = load_board(&conn, "default").unwrap();
        assert_eq!(objects.len(), 1);

        // Update obj-1 content
        let mut obj1_updated = objects[0].clone();
        obj1_updated.content = "Updated content".to_string();
        save_board_state(&mut conn, "default", 0.0, 0.0, 1.0, &[obj1_updated]).unwrap();

        let (_, objects_after_update) = load_board(&conn, "default").unwrap();
        assert_eq!(objects_after_update.len(), 1);
        assert_eq!(objects_after_update[0].content, "Updated content");

        // Delete obj-1
        delete_objects(&conn, &["obj-1".to_string()]).unwrap();
        let (_, objects_after_delete) = load_board(&conn, "default").unwrap();
        assert_eq!(objects_after_delete.len(), 0);
    }
}
