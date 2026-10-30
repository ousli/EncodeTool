use crate::jobs::Job;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::path::Path;
use std::sync::Mutex;

pub struct Db {
    conn: Mutex<Connection>,
}

#[derive(Clone, Serialize)]
pub struct Lut {
    pub id: i64,
    pub name: String,
    pub path: String,
}

impl Db {
    pub fn open(path: &Path) -> rusqlite::Result<Db> {
        let conn = Connection::open(path)?;
        Self::init(conn)
    }

    #[cfg(test)]
    pub fn in_memory() -> rusqlite::Result<Db> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> rusqlite::Result<Db> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS luts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                path TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS jobs (
                id INTEGER PRIMARY KEY,
                data TEXT NOT NULL
            );",
        )?;
        Ok(Db { conn: Mutex::new(conn) })
    }

    pub fn list_luts(&self) -> rusqlite::Result<Vec<Lut>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, path FROM luts ORDER BY name COLLATE NOCASE")?;
        let rows = stmt.query_map([], |r| Ok(Lut { id: r.get(0)?, name: r.get(1)?, path: r.get(2)? }))?;
        rows.collect()
    }

    pub fn get_lut(&self, id: i64) -> rusqlite::Result<Option<Lut>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, path FROM luts WHERE id = ?1")?;
        let mut rows = stmt.query_map([id], |r| Ok(Lut { id: r.get(0)?, name: r.get(1)?, path: r.get(2)? }))?;
        rows.next().transpose()
    }

    pub fn add_lut(&self, name: &str, path: &str) -> rusqlite::Result<Lut> {
        let conn = self.conn.lock().unwrap();
        conn.execute("INSERT INTO luts (name, path) VALUES (?1, ?2)", params![name, path])?;
        Ok(Lut { id: conn.last_insert_rowid(), name: name.into(), path: path.into() })
    }

    pub fn delete_lut(&self, id: i64) -> rusqlite::Result<bool> {
        let conn = self.conn.lock().unwrap();
        Ok(conn.execute("DELETE FROM luts WHERE id = ?1", [id])? > 0)
    }

    pub fn load_jobs(&self) -> Vec<Job> {
        let conn = self.conn.lock().unwrap();
        let Ok(mut stmt) = conn.prepare("SELECT data FROM jobs ORDER BY id") else { return vec![] };
        let rows = stmt.query_map([], |r| r.get::<_, String>(0));
        match rows {
            Ok(rows) => rows.filter_map(|r| r.ok()).filter_map(|d| serde_json::from_str(&d).ok()).collect(),
            Err(_) => vec![],
        }
    }

    pub fn save_job(&self, job: &Job) {
        let Ok(data) = serde_json::to_string(job) else { return };
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute("INSERT OR REPLACE INTO jobs (id, data) VALUES (?1, ?2)", params![job.id, data]);
    }

    pub fn delete_job(&self, id: i64) {
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute("DELETE FROM jobs WHERE id = ?1", [id]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lut_crud() {
        let db = Db::in_memory().unwrap();
        let lut = db.add_lut("S-Log2", "/tmp/a.cube").unwrap();
        assert!(db.add_lut("S-Log2", "/tmp/b.cube").is_err());
        assert_eq!(db.list_luts().unwrap().len(), 1);
        assert_eq!(db.get_lut(lut.id).unwrap().unwrap().path, "/tmp/a.cube");
        assert!(db.delete_lut(lut.id).unwrap());
        assert!(db.get_lut(lut.id).unwrap().is_none());
    }
}
