use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use rusqlite::{params, Connection, Result};
use crate::models::{Board, BoardConnector, BoardObject};
use uuid::Uuid;

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

    let mut conn = Connection::open(db_path)?;
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

        CREATE TABLE IF NOT EXISTS board_connectors (
            id TEXT PRIMARY KEY,
            board_id TEXT NOT NULL,
            from_id TEXT,
            from_anchor TEXT,
            from_x REAL NOT NULL DEFAULT 0.0,
            from_y REAL NOT NULL DEFAULT 0.0,
            to_id TEXT,
            to_anchor TEXT,
            to_x REAL NOT NULL DEFAULT 0.0,
            to_y REAL NOT NULL DEFAULT 0.0,
            line_type TEXT NOT NULL DEFAULT 'curved',
            arrow_start TEXT NOT NULL DEFAULT 'none',
            arrow_end TEXT NOT NULL DEFAULT 'arrow',
            stroke_style TEXT NOT NULL DEFAULT 'solid',
            color TEXT NOT NULL DEFAULT '#a855f7',
            stroke_width REAL NOT NULL DEFAULT 2.0,
            label TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (board_id) REFERENCES boards(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_board_objects_board ON board_objects(board_id);
        CREATE INDEX IF NOT EXISTS idx_board_objects_kind ON board_objects(kind);
        CREATE INDEX IF NOT EXISTS idx_board_connectors_board ON board_connectors(board_id);
        "#,
    )?;

    // Insert default board if none exists
    let now = now_millis();
    conn.execute(
        "INSERT OR IGNORE INTO boards (id, name, pan_x, pan_y, zoom, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params!["default", "Untitled Board", 0.0, 0.0, 1.0, now],
    )?;

    // Seed default board if it has no objects
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM board_objects WHERE board_id = 'default'",
        [],
        |row| row.get(0),
    ).unwrap_or(0);

    if count == 0 {
        let _ = seed_default_board(&mut conn);
    }

    Ok(conn)
}

pub fn get_boards(conn: &Connection) -> Result<Vec<Board>> {
    let mut stmt = conn.prepare("SELECT id, name, pan_x, pan_y, zoom, updated_at FROM boards ORDER BY updated_at DESC")?;
    let iter = stmt.query_map([], |row| {
        Ok(Board {
            id: row.get(0)?,
            name: row.get(1)?,
            pan_x: row.get(2)?,
            pan_y: row.get(3)?,
            zoom: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })?;

    let mut boards = Vec::new();
    for b in iter {
        boards.push(b?);
    }
    Ok(boards)
}

pub fn create_board(conn: &Connection, name: &str) -> Result<Board> {
    let id = Uuid::new_v4().to_string();
    let now = now_millis();
    let board_name = if name.trim().is_empty() { "Untitled Board" } else { name };

    conn.execute(
        "INSERT INTO boards (id, name, pan_x, pan_y, zoom, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, board_name, 0.0, 0.0, 1.0, now],
    )?;

    Ok(Board {
        id,
        name: board_name.to_string(),
        pan_x: 0.0,
        pan_y: 0.0,
        zoom: 1.0,
        updated_at: now,
    })
}

pub fn rename_board(conn: &Connection, id: &str, name: &str) -> Result<()> {
    let now = now_millis();
    conn.execute(
        "UPDATE boards SET name = ?1, updated_at = ?2 WHERE id = ?3",
        params![name, now, id],
    )?;
    Ok(())
}

pub fn load_board(conn: &Connection, board_id: &str) -> Result<(Board, Vec<BoardObject>, Vec<BoardConnector>)> {
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

    // Objects
    let mut stmt = conn.prepare(
        "SELECT id, board_id, kind, x, y, width, height, z_index, title, content, meta, created_at, updated_at 
         FROM board_objects WHERE board_id = ?1 ORDER BY z_index ASC",
    )?;

    let object_iter = stmt.query_map(params![board_id], |row| {
        let meta_raw: Option<String> = row.get(10)?;
        let meta = meta_raw.and_then(|s| serde_json::from_str(&s).ok());
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
            meta,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?;

    let mut objects = Vec::new();
    for obj in object_iter {
        objects.push(obj?);
    }

    // Connectors
    let mut conn_stmt = conn.prepare(
        "SELECT id, board_id, from_id, from_anchor, from_x, from_y, to_id, to_anchor, to_x, to_y, 
                line_type, arrow_start, arrow_end, stroke_style, color, stroke_width, label, created_at, updated_at
         FROM board_connectors WHERE board_id = ?1",
    )?;

    let conn_iter = conn_stmt.query_map(params![board_id], |row| {
        Ok(BoardConnector {
            id: row.get(0)?,
            board_id: row.get(1)?,
            from_id: row.get(2)?,
            from_anchor: row.get(3)?,
            from_x: row.get(4)?,
            from_y: row.get(5)?,
            to_id: row.get(6)?,
            to_anchor: row.get(7)?,
            to_x: row.get(8)?,
            to_y: row.get(9)?,
            line_type: row.get(10)?,
            arrow_start: row.get(11)?,
            arrow_end: row.get(12)?,
            stroke_style: row.get(13)?,
            color: row.get(14)?,
            stroke_width: row.get(15)?,
            label: row.get(16)?,
            created_at: row.get(17)?,
            updated_at: row.get(18)?,
        })
    })?;

    let mut connectors = Vec::new();
    for c in conn_iter {
        connectors.push(c?);
    }

    Ok((board, objects, connectors))
}

pub fn save_board_state(
    conn: &mut Connection,
    board_id: &str,
    pan_x: f64,
    pan_y: f64,
    zoom: f64,
    objects: &[BoardObject],
    connectors: &[BoardConnector],
) -> Result<()> {
    let tx = conn.transaction()?;
    let now = now_millis();

    tx.execute(
        "UPDATE boards SET pan_x = ?1, pan_y = ?2, zoom = ?3, updated_at = ?4 WHERE id = ?5",
        params![pan_x, pan_y, zoom, now, board_id],
    )?;

    // 1. Sync Objects
    if objects.is_empty() {
        tx.execute("DELETE FROM board_objects WHERE board_id = ?1", params![board_id])?;
    } else {
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
            let meta_str = obj.meta.as_ref().map(|v| v.to_string());
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
                meta_str,
                created,
                updated
            ])?;
        }
    }

    // 2. Sync Connectors
    if connectors.is_empty() {
        tx.execute("DELETE FROM board_connectors WHERE board_id = ?1", params![board_id])?;
    } else {
        let placeholders: Vec<String> = (0..connectors.len()).map(|_| "?".to_string()).collect();
        let query = format!(
            "DELETE FROM board_connectors WHERE board_id = ?1 AND id NOT IN ({})",
            placeholders.join(",")
        );
        let mut del_params: Vec<&dyn rusqlite::ToSql> = vec![&board_id];
        for c in connectors {
            del_params.push(&c.id);
        }
        tx.execute(&query, del_params.as_slice())?;

        let mut stmt = tx.prepare(
            r#"
            INSERT INTO board_connectors
            (id, board_id, from_id, from_anchor, from_x, from_y, to_id, to_anchor, to_x, to_y,
             line_type, arrow_start, arrow_end, stroke_style, color, stroke_width, label, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)
            ON CONFLICT(id) DO UPDATE SET
                from_id = excluded.from_id,
                from_anchor = excluded.from_anchor,
                from_x = excluded.from_x,
                from_y = excluded.from_y,
                to_id = excluded.to_id,
                to_anchor = excluded.to_anchor,
                to_x = excluded.to_x,
                to_y = excluded.to_y,
                line_type = excluded.line_type,
                arrow_start = excluded.arrow_start,
                arrow_end = excluded.arrow_end,
                stroke_style = excluded.stroke_style,
                color = excluded.color,
                stroke_width = excluded.stroke_width,
                label = excluded.label,
                updated_at = excluded.updated_at
            "#,
        )?;

        for c in connectors {
            let created = c.created_at.unwrap_or(now);
            let updated = c.updated_at.unwrap_or(now);
            stmt.execute(params![
                c.id,
                board_id,
                c.from_id,
                c.from_anchor,
                c.from_x,
                c.from_y,
                c.to_id,
                c.to_anchor,
                c.to_x,
                c.to_y,
                c.line_type,
                c.arrow_start,
                c.arrow_end,
                c.stroke_style,
                c.color,
                c.stroke_width,
                c.label,
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

    // Also clean up or detach connectors referencing deleted objects
    let conn_del_query = format!(
        "DELETE FROM board_connectors WHERE from_id IN ({}) OR to_id IN ({})",
        placeholders.join(","),
        placeholders.join(",")
    );
    let mut conn_params: Vec<&dyn rusqlite::ToSql> = Vec::new();
    for id in ids { conn_params.push(id as &dyn rusqlite::ToSql); }
    for id in ids { conn_params.push(id as &dyn rusqlite::ToSql); }
    conn.execute(&conn_del_query, conn_params.as_slice())?;

    Ok(())
}

pub fn seed_default_board(conn: &mut Connection) -> Result<()> {
    let now = now_millis();
    let objects = vec![
        BoardObject {
            id: "sample-img-tree".to_string(),
            board_id: "default".to_string(),
            kind: "image".to_string(),
            x: 60.0,
            y: 50.0,
            width: 285.0,
            height: 162.0,
            z_index: 1,
            title: Some("Fantasy Castle".to_string()),
            content: "/samples/sample_tree.jpg".to_string(),
            meta: None,
            created_at: Some(now),
            updated_at: Some(now),
        },
        BoardObject {
            id: "sample-img-room".to_string(),
            board_id: "default".to_string(),
            kind: "image".to_string(),
            x: 60.0,
            y: 232.0,
            width: 285.0,
            height: 183.0,
            z_index: 2,
            title: Some("Studio Desk".to_string()),
            content: "/samples/sample_room.jpg".to_string(),
            meta: None,
            created_at: Some(now),
            updated_at: Some(now),
        },
        BoardObject {
            id: "sample-note-mind-studio".to_string(),
            board_id: "default".to_string(),
            kind: "text".to_string(),
            x: 375.0,
            y: 50.0,
            width: 215.0,
            height: 165.0,
            z_index: 3,
            title: Some("Ideas for Mind Studio".to_string()),
            content: "- [ ] Lobby room (base)\n- [ ] Task / quest flow\n- [x] Character idle & poses\n- [ ] Unlockable furniture\n- [ ] Mobile (iPad) build\n\n#game #design #todo".to_string(),
            meta: None,
            created_at: Some(now),
            updated_at: Some(now),
        },
        BoardObject {
            id: "sample-img-character".to_string(),
            board_id: "default".to_string(),
            kind: "image".to_string(),
            x: 615.0,
            y: 50.0,
            width: 246.0,
            height: 150.0,
            z_index: 4,
            title: Some("Character Concept".to_string()),
            content: "/samples/sample_character.jpg".to_string(),
            meta: None,
            created_at: Some(now),
            updated_at: Some(now),
        },
        BoardObject {
            id: "sample-img-mountain".to_string(),
            board_id: "default".to_string(),
            kind: "image".to_string(),
            x: 375.0,
            y: 235.0,
            width: 247.0,
            height: 131.0,
            z_index: 5,
            title: Some("Mountain Lake".to_string()),
            content: "/samples/sample_mountain.jpg".to_string(),
            meta: None,
            created_at: Some(now),
            updated_at: Some(now),
        },
        BoardObject {
            id: "sample-code-snippet".to_string(),
            board_id: "default".to_string(),
            kind: "text".to_string(),
            x: 885.0,
            y: 50.0,
            width: 220.0,
            height: 145.0,
            z_index: 6,
            title: Some("Snippet".to_string()),
            content: "const refs = [\n  \"lobby\", \"character\", \"ui\"\n]\nsave(refs)".to_string(),
            meta: Some(serde_json::json!({ "is_code": true, "language": "javascript" })),
            created_at: Some(now),
            updated_at: Some(now),
        },
        BoardObject {
            id: "sample-url-godot".to_string(),
            board_id: "default".to_string(),
            kind: "url".to_string(),
            x: 885.0,
            y: 215.0,
            width: 220.0,
            height: 75.0,
            z_index: 7,
            title: Some("Godot 4.7 Docs".to_string()),
            content: "https://docs.godotengine.org/".to_string(),
            meta: None,
            created_at: Some(now),
            updated_at: Some(now),
        },
        // Network topology nodes
        BoardObject {
            id: "node-router".to_string(),
            board_id: "default".to_string(),
            kind: "diagram_node".to_string(),
            x: 125.0,
            y: 475.0,
            width: 130.0,
            height: 48.0,
            z_index: 8,
            title: Some("Router".to_string()),
            content: "192.0.2.1".to_string(),
            meta: Some(serde_json::json!({ "node_type": "router" })),
            created_at: Some(now),
            updated_at: Some(now),
        },
        BoardObject {
            id: "node-switch".to_string(),
            board_id: "default".to_string(),
            kind: "diagram_node".to_string(),
            x: 325.0,
            y: 475.0,
            width: 150.0,
            height: 48.0,
            z_index: 9,
            title: Some("Core Switch".to_string()),
            content: "10.0.0.1".to_string(),
            meta: Some(serde_json::json!({ "node_type": "switch" })),
            created_at: Some(now),
            updated_at: Some(now),
        },
        BoardObject {
            id: "node-olt".to_string(),
            board_id: "default".to_string(),
            kind: "diagram_node".to_string(),
            x: 535.0,
            y: 415.0,
            width: 130.0,
            height: 48.0,
            z_index: 10,
            title: Some("OLT".to_string()),
            content: "10.0.10.1".to_string(),
            meta: Some(serde_json::json!({ "node_type": "olt" })),
            created_at: Some(now),
            updated_at: Some(now),
        },
        BoardObject {
            id: "node-ap".to_string(),
            board_id: "default".to_string(),
            kind: "diagram_node".to_string(),
            x: 540.0,
            y: 530.0,
            width: 135.0,
            height: 48.0,
            z_index: 11,
            title: Some("AP".to_string()),
            content: "10.0.20.1".to_string(),
            meta: Some(serde_json::json!({ "node_type": "ap" })),
            created_at: Some(now),
            updated_at: Some(now),
        },
        BoardObject {
            id: "node-onu1".to_string(),
            board_id: "default".to_string(),
            kind: "diagram_node".to_string(),
            x: 755.0,
            y: 390.0,
            width: 125.0,
            height: 48.0,
            z_index: 12,
            title: Some("ONU-1".to_string()),
            content: "10.0.10.101".to_string(),
            meta: Some(serde_json::json!({ "node_type": "onu" })),
            created_at: Some(now),
            updated_at: Some(now),
        },
        BoardObject {
            id: "node-onu2".to_string(),
            board_id: "default".to_string(),
            kind: "diagram_node".to_string(),
            x: 755.0,
            y: 460.0,
            width: 125.0,
            height: 48.0,
            z_index: 13,
            title: Some("ONU-2".to_string()),
            content: "10.0.10.102".to_string(),
            meta: Some(serde_json::json!({ "node_type": "onu" })),
            created_at: Some(now),
            updated_at: Some(now),
        },
        BoardObject {
            id: "node-client-phone".to_string(),
            board_id: "default".to_string(),
            kind: "diagram_node".to_string(),
            x: 750.0,
            y: 530.0,
            width: 145.0,
            height: 48.0,
            z_index: 14,
            title: Some("Client".to_string()),
            content: "10.0.20.100".to_string(),
            meta: Some(serde_json::json!({ "node_type": "client_phone" })),
            created_at: Some(now),
            updated_at: Some(now),
        },
        BoardObject {
            id: "node-client-laptop".to_string(),
            board_id: "default".to_string(),
            kind: "diagram_node".to_string(),
            x: 940.0,
            y: 390.0,
            width: 125.0,
            height: 48.0,
            z_index: 15,
            title: Some("Client".to_string()),
            content: "10.0.10.200".to_string(),
            meta: Some(serde_json::json!({ "node_type": "client_laptop" })),
            created_at: Some(now),
            updated_at: Some(now),
        },
        BoardObject {
            id: "node-client-pc".to_string(),
            board_id: "default".to_string(),
            kind: "diagram_node".to_string(),
            x: 940.0,
            y: 460.0,
            width: 125.0,
            height: 48.0,
            z_index: 16,
            title: Some("Client".to_string()),
            content: "10.0.10.201".to_string(),
            meta: Some(serde_json::json!({ "node_type": "client_pc" })),
            created_at: Some(now),
            updated_at: Some(now),
        },
    ];

    let connectors = vec![
        BoardConnector {
            id: "conn-router-switch".to_string(),
            board_id: "default".to_string(),
            from_id: Some("node-router".to_string()),
            from_anchor: Some("right".to_string()),
            from_x: 255.0,
            from_y: 499.0,
            to_id: Some("node-switch".to_string()),
            to_anchor: Some("left".to_string()),
            to_x: 325.0,
            to_y: 499.0,
            line_type: "straight".to_string(),
            arrow_start: "none".to_string(),
            arrow_end: "arrow".to_string(),
            stroke_style: "solid".to_string(),
            color: "#a855f7".to_string(),
            stroke_width: 2.0,
            label: None,
            created_at: Some(now),
            updated_at: Some(now),
        },
        BoardConnector {
            id: "conn-switch-olt".to_string(),
            board_id: "default".to_string(),
            from_id: Some("node-switch".to_string()),
            from_anchor: Some("right".to_string()),
            from_x: 475.0,
            from_y: 499.0,
            to_id: Some("node-olt".to_string()),
            to_anchor: Some("left".to_string()),
            to_x: 535.0,
            to_y: 439.0,
            line_type: "curved".to_string(),
            arrow_start: "none".to_string(),
            arrow_end: "none".to_string(),
            stroke_style: "solid".to_string(),
            color: "#a855f7".to_string(),
            stroke_width: 2.0,
            label: None,
            created_at: Some(now),
            updated_at: Some(now),
        },
        BoardConnector {
            id: "conn-switch-ap".to_string(),
            board_id: "default".to_string(),
            from_id: Some("node-switch".to_string()),
            from_anchor: Some("right".to_string()),
            from_x: 475.0,
            from_y: 499.0,
            to_id: Some("node-ap".to_string()),
            to_anchor: Some("left".to_string()),
            to_x: 540.0,
            to_y: 554.0,
            line_type: "curved".to_string(),
            arrow_start: "none".to_string(),
            arrow_end: "none".to_string(),
            stroke_style: "solid".to_string(),
            color: "#a855f7".to_string(),
            stroke_width: 2.0,
            label: None,
            created_at: Some(now),
            updated_at: Some(now),
        },
        BoardConnector {
            id: "conn-olt-onu1".to_string(),
            board_id: "default".to_string(),
            from_id: Some("node-olt".to_string()),
            from_anchor: Some("right".to_string()),
            from_x: 665.0,
            from_y: 439.0,
            to_id: Some("node-onu1".to_string()),
            to_anchor: Some("left".to_string()),
            to_x: 755.0,
            to_y: 414.0,
            line_type: "curved".to_string(),
            arrow_start: "none".to_string(),
            arrow_end: "none".to_string(),
            stroke_style: "solid".to_string(),
            color: "#c084fc".to_string(),
            stroke_width: 2.5,
            label: Some("PON (GPON)".to_string()),
            created_at: Some(now),
            updated_at: Some(now),
        },
        BoardConnector {
            id: "conn-olt-onu2".to_string(),
            board_id: "default".to_string(),
            from_id: Some("node-olt".to_string()),
            from_anchor: Some("right".to_string()),
            from_x: 665.0,
            from_y: 439.0,
            to_id: Some("node-onu2".to_string()),
            to_anchor: Some("left".to_string()),
            to_x: 755.0,
            to_y: 484.0,
            line_type: "curved".to_string(),
            arrow_start: "none".to_string(),
            arrow_end: "none".to_string(),
            stroke_style: "solid".to_string(),
            color: "#a855f7".to_string(),
            stroke_width: 2.0,
            label: None,
            created_at: Some(now),
            updated_at: Some(now),
        },
        BoardConnector {
            id: "conn-onu1-laptop".to_string(),
            board_id: "default".to_string(),
            from_id: Some("node-onu1".to_string()),
            from_anchor: Some("right".to_string()),
            from_x: 880.0,
            from_y: 414.0,
            to_id: Some("node-client-laptop".to_string()),
            to_anchor: Some("left".to_string()),
            to_x: 940.0,
            to_y: 414.0,
            line_type: "straight".to_string(),
            arrow_start: "none".to_string(),
            arrow_end: "arrow".to_string(),
            stroke_style: "solid".to_string(),
            color: "#a855f7".to_string(),
            stroke_width: 2.0,
            label: None,
            created_at: Some(now),
            updated_at: Some(now),
        },
        BoardConnector {
            id: "conn-onu2-pc".to_string(),
            board_id: "default".to_string(),
            from_id: Some("node-onu2".to_string()),
            from_anchor: Some("right".to_string()),
            from_x: 880.0,
            from_y: 484.0,
            to_id: Some("node-client-pc".to_string()),
            to_anchor: Some("left".to_string()),
            to_x: 940.0,
            to_y: 484.0,
            line_type: "straight".to_string(),
            arrow_start: "none".to_string(),
            arrow_end: "arrow".to_string(),
            stroke_style: "solid".to_string(),
            color: "#a855f7".to_string(),
            stroke_width: 2.0,
            label: None,
            created_at: Some(now),
            updated_at: Some(now),
        },
    ];

    save_board_state(conn, "default", 0.0, 0.0, 1.0, &objects, &connectors)?;
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
            CREATE TABLE IF NOT EXISTS board_connectors (
                id TEXT PRIMARY KEY,
                board_id TEXT NOT NULL,
                from_id TEXT,
                from_anchor TEXT,
                from_x REAL NOT NULL DEFAULT 0.0,
                from_y REAL NOT NULL DEFAULT 0.0,
                to_id TEXT,
                to_anchor TEXT,
                to_x REAL NOT NULL DEFAULT 0.0,
                to_y REAL NOT NULL DEFAULT 0.0,
                line_type TEXT NOT NULL DEFAULT 'curved',
                arrow_start TEXT NOT NULL DEFAULT 'none',
                arrow_end TEXT NOT NULL DEFAULT 'arrow',
                stroke_style TEXT NOT NULL DEFAULT 'solid',
                color TEXT NOT NULL DEFAULT '#a855f7',
                stroke_width REAL NOT NULL DEFAULT 2.0,
                label TEXT,
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
    fn test_save_and_load_board_with_connectors() {
        let mut conn = setup_test_db();
        let obj1 = BoardObject {
            id: "obj-1".to_string(),
            board_id: "default".to_string(),
            kind: "diagram_node".to_string(),
            x: 100.0,
            y: 150.0,
            width: 170.0,
            height: 60.0,
            z_index: 1,
            title: Some("Router".to_string()),
            content: "192.0.2.1".to_string(),
            meta: Some(serde_json::json!({"node_type":"router"})),
            created_at: Some(1000),
            updated_at: Some(1000),
        };
        let obj2 = BoardObject {
            id: "obj-2".to_string(),
            board_id: "default".to_string(),
            kind: "diagram_node".to_string(),
            x: 350.0,
            y: 150.0,
            width: 170.0,
            height: 60.0,
            z_index: 2,
            title: Some("Core Switch".to_string()),
            content: "10.0.0.1".to_string(),
            meta: Some(serde_json::json!({"node_type":"switch"})),
            created_at: Some(1001),
            updated_at: Some(1001),
        };

        let connector = BoardConnector {
            id: "conn-1".to_string(),
            board_id: "default".to_string(),
            from_id: Some("obj-1".to_string()),
            from_anchor: Some("right".to_string()),
            from_x: 270.0,
            from_y: 180.0,
            to_id: Some("obj-2".to_string()),
            to_anchor: Some("left".to_string()),
            to_x: 350.0,
            to_y: 180.0,
            line_type: "curved".to_string(),
            arrow_start: "none".to_string(),
            arrow_end: "arrow".to_string(),
            stroke_style: "solid".to_string(),
            color: "#a855f7".to_string(),
            stroke_width: 2.0,
            label: Some("10G Fiber".to_string()),
            created_at: Some(1002),
            updated_at: Some(1002),
        };

        save_board_state(&mut conn, "default", 50.0, 60.0, 2.0, &[obj1, obj2], &[connector]).unwrap();

        let (board, objects, connectors) = load_board(&conn, "default").unwrap();
        assert_eq!(board.id, "default");
        assert_eq!(objects.len(), 2);
        assert_eq!(connectors.len(), 1);
        assert_eq!(connectors[0].id, "conn-1");
        assert_eq!(connectors[0].label.as_deref(), Some("10G Fiber"));
        assert_eq!(connectors[0].from_anchor.as_deref(), Some("right"));
    }

    #[test]
    fn test_delete_and_clean_connectors() {
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
        let connector = BoardConnector {
            id: "conn-1".to_string(),
            board_id: "default".to_string(),
            from_id: Some("obj-1".to_string()),
            from_anchor: Some("right".to_string()),
            from_x: 100.0,
            from_y: 50.0,
            to_id: None,
            to_anchor: None,
            to_x: 200.0,
            to_y: 50.0,
            line_type: "straight".to_string(),
            arrow_start: "none".to_string(),
            arrow_end: "none".to_string(),
            stroke_style: "solid".to_string(),
            color: "#a855f7".to_string(),
            stroke_width: 2.0,
            label: None,
            created_at: Some(101),
            updated_at: Some(101),
        };

        save_board_state(&mut conn, "default", 0.0, 0.0, 1.0, &[obj1], &[connector]).unwrap();

        // Delete obj-1
        delete_objects(&conn, &["obj-1".to_string()]).unwrap();
        let (_, objects_after_delete, connectors_after_delete) = load_board(&conn, "default").unwrap();
        assert_eq!(objects_after_delete.len(), 0);
        assert_eq!(connectors_after_delete.len(), 0);
    }
}
