use anyhow::Result;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct SkillStore {
    conn: Mutex<Connection>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillRecord {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub source_type: String,
    pub source_ref: Option<String>,
    pub source_revision: Option<String>,
    pub central_path: String,
    pub content_hash: Option<String>,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillTargetRecord {
    pub id: String,
    pub skill_id: String,
    pub tool: String,
    pub target_path: String,
    pub mode: String,
    pub status: String,
    pub synced_at: Option<i64>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiscoveredSkillRecord {
    pub id: String,
    pub tool: String,
    pub found_path: String,
    pub name_guess: Option<String>,
    pub fingerprint: Option<String>,
    pub found_at: i64,
    pub imported_skill_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScenarioRecord {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub created_at: i64,
    pub updated_at: i64,
}

impl SkillStore {
    pub fn new(db_path: &PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS skills (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                source_type TEXT NOT NULL,
                source_ref TEXT,
                source_revision TEXT,
                central_path TEXT NOT NULL UNIQUE,
                content_hash TEXT,
                enabled INTEGER DEFAULT 1,
                created_at INTEGER,
                updated_at INTEGER,
                status TEXT DEFAULT 'ok'
            );
            CREATE INDEX IF NOT EXISTS idx_skills_name ON skills(name);

            CREATE TABLE IF NOT EXISTS skill_targets (
                id TEXT PRIMARY KEY,
                skill_id TEXT NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
                tool TEXT NOT NULL,
                target_path TEXT NOT NULL,
                mode TEXT NOT NULL,
                status TEXT DEFAULT 'ok',
                synced_at INTEGER,
                last_error TEXT,
                UNIQUE(skill_id, tool)
            );

            CREATE TABLE IF NOT EXISTS discovered_skills (
                id TEXT PRIMARY KEY,
                tool TEXT NOT NULL,
                found_path TEXT NOT NULL,
                name_guess TEXT,
                fingerprint TEXT,
                found_at INTEGER NOT NULL,
                imported_skill_id TEXT REFERENCES skills(id) ON DELETE SET NULL
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS skillssh_cache (
                cache_key TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                fetched_at INTEGER
            );

            CREATE TABLE IF NOT EXISTS scenarios (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                sort_order INTEGER DEFAULT 0,
                created_at INTEGER,
                updated_at INTEGER
            );

            CREATE TABLE IF NOT EXISTS scenario_skills (
                scenario_id TEXT NOT NULL REFERENCES scenarios(id) ON DELETE CASCADE,
                skill_id TEXT NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
                added_at INTEGER,
                PRIMARY KEY(scenario_id, skill_id)
            );

            CREATE TABLE IF NOT EXISTS active_scenario (
                key TEXT PRIMARY KEY DEFAULT 'current',
                scenario_id TEXT REFERENCES scenarios(id) ON DELETE SET NULL
            );
            ",
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    // ── Skills CRUD ──

    pub fn insert_skill(&self, skill: &SkillRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO skills (id, name, description, source_type, source_ref, source_revision, central_path, content_hash, enabled, created_at, updated_at, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                skill.id,
                skill.name,
                skill.description,
                skill.source_type,
                skill.source_ref,
                skill.source_revision,
                skill.central_path,
                skill.content_hash,
                skill.enabled,
                skill.created_at,
                skill.updated_at,
                skill.status,
            ],
        )?;
        Ok(())
    }

    pub fn get_all_skills(&self) -> Result<Vec<SkillRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, source_type, source_ref, source_revision, central_path, content_hash, enabled, created_at, updated_at, status FROM skills ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(SkillRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                source_type: row.get(3)?,
                source_ref: row.get(4)?,
                source_revision: row.get(5)?,
                central_path: row.get(6)?,
                content_hash: row.get(7)?,
                enabled: row.get::<_, i32>(8)? != 0,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                status: row.get(11)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_skill_by_id(&self, id: &str) -> Result<Option<SkillRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, source_type, source_ref, source_revision, central_path, content_hash, enabled, created_at, updated_at, status FROM skills WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(SkillRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                source_type: row.get(3)?,
                source_ref: row.get(4)?,
                source_revision: row.get(5)?,
                central_path: row.get(6)?,
                content_hash: row.get(7)?,
                enabled: row.get::<_, i32>(8)? != 0,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                status: row.get(11)?,
            })
        })?;
        Ok(rows.next().and_then(|r| r.ok()))
    }

    pub fn delete_skill(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM skills WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ── Targets ──

    pub fn insert_target(&self, target: &SkillTargetRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO skill_targets (id, skill_id, tool, target_path, mode, status, synced_at, last_error)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                target.id,
                target.skill_id,
                target.tool,
                target.target_path,
                target.mode,
                target.status,
                target.synced_at,
                target.last_error,
            ],
        )?;
        Ok(())
    }

    pub fn get_targets_for_skill(&self, skill_id: &str) -> Result<Vec<SkillTargetRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, skill_id, tool, target_path, mode, status, synced_at, last_error FROM skill_targets WHERE skill_id = ?1",
        )?;
        let rows = stmt.query_map(params![skill_id], |row| {
            Ok(SkillTargetRecord {
                id: row.get(0)?,
                skill_id: row.get(1)?,
                tool: row.get(2)?,
                target_path: row.get(3)?,
                mode: row.get(4)?,
                status: row.get(5)?,
                synced_at: row.get(6)?,
                last_error: row.get(7)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_all_targets(&self) -> Result<Vec<SkillTargetRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, skill_id, tool, target_path, mode, status, synced_at, last_error FROM skill_targets",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(SkillTargetRecord {
                id: row.get(0)?,
                skill_id: row.get(1)?,
                tool: row.get(2)?,
                target_path: row.get(3)?,
                mode: row.get(4)?,
                status: row.get(5)?,
                synced_at: row.get(6)?,
                last_error: row.get(7)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn delete_target(&self, skill_id: &str, tool: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM skill_targets WHERE skill_id = ?1 AND tool = ?2",
            params![skill_id, tool],
        )?;
        Ok(())
    }

    // ── Discovered Skills ──

    pub fn clear_discovered(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM discovered_skills", [])?;
        Ok(())
    }

    pub fn insert_discovered(&self, rec: &DiscoveredSkillRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO discovered_skills (id, tool, found_path, name_guess, fingerprint, found_at, imported_skill_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                rec.id,
                rec.tool,
                rec.found_path,
                rec.name_guess,
                rec.fingerprint,
                rec.found_at,
                rec.imported_skill_id,
            ],
        )?;
        Ok(())
    }

    pub fn get_all_discovered(&self) -> Result<Vec<DiscoveredSkillRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, tool, found_path, name_guess, fingerprint, found_at, imported_skill_id FROM discovered_skills",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(DiscoveredSkillRecord {
                id: row.get(0)?,
                tool: row.get(1)?,
                found_path: row.get(2)?,
                name_guess: row.get(3)?,
                fingerprint: row.get(4)?,
                found_at: row.get(5)?,
                imported_skill_id: row.get(6)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    // ── Cache ──

    pub fn get_cache(&self, key: &str, ttl_secs: i64) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();
        let mut stmt = conn.prepare(
            "SELECT data FROM skillssh_cache WHERE cache_key = ?1 AND fetched_at > ?2",
        )?;
        let cutoff = now - ttl_secs;
        let mut rows = stmt.query_map(params![key, cutoff], |row| row.get::<_, String>(0))?;
        Ok(rows.next().and_then(|r| r.ok()))
    }

    pub fn set_cache(&self, key: &str, data: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT OR REPLACE INTO skillssh_cache (cache_key, data, fetched_at) VALUES (?1, ?2, ?3)",
            params![key, data, now],
        )?;
        Ok(())
    }

    // ── Settings ──

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query_map(params![key], |row| row.get::<_, String>(0))?;
        Ok(rows.next().and_then(|r| r.ok()))
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    // ── Scenarios ──

    pub fn insert_scenario(&self, scenario: &ScenarioRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO scenarios (id, name, description, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                scenario.id,
                scenario.name,
                scenario.description,
                scenario.sort_order,
                scenario.created_at,
                scenario.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_all_scenarios(&self) -> Result<Vec<ScenarioRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, sort_order, created_at, updated_at FROM scenarios ORDER BY sort_order, created_at",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ScenarioRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                sort_order: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn update_scenario(&self, id: &str, name: &str, description: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp_millis();
        conn.execute(
            "UPDATE scenarios SET name = ?1, description = ?2, updated_at = ?3 WHERE id = ?4",
            params![name, description, now, id],
        )?;
        Ok(())
    }

    pub fn delete_scenario(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM scenarios WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn reorder_scenarios(&self, ids: &[String]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        for (i, id) in ids.iter().enumerate() {
            conn.execute(
                "UPDATE scenarios SET sort_order = ?1 WHERE id = ?2",
                params![i as i32, id],
            )?;
        }
        Ok(())
    }

    // ── Scenario-Skill mapping ──

    pub fn add_skill_to_scenario(&self, scenario_id: &str, skill_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp_millis();
        conn.execute(
            "INSERT OR IGNORE INTO scenario_skills (scenario_id, skill_id, added_at) VALUES (?1, ?2, ?3)",
            params![scenario_id, skill_id, now],
        )?;
        Ok(())
    }

    pub fn remove_skill_from_scenario(&self, scenario_id: &str, skill_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM scenario_skills WHERE scenario_id = ?1 AND skill_id = ?2",
            params![scenario_id, skill_id],
        )?;
        Ok(())
    }

    pub fn get_skill_ids_for_scenario(&self, scenario_id: &str) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT skill_id FROM scenario_skills WHERE scenario_id = ?1",
        )?;
        let rows = stmt.query_map(params![scenario_id], |row| row.get::<_, String>(0))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_skills_for_scenario(&self, scenario_id: &str) -> Result<Vec<SkillRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT s.id, s.name, s.description, s.source_type, s.source_ref, s.source_revision, s.central_path, s.content_hash, s.enabled, s.created_at, s.updated_at, s.status
             FROM skills s
             INNER JOIN scenario_skills ss ON s.id = ss.skill_id
             WHERE ss.scenario_id = ?1
             ORDER BY s.name",
        )?;
        let rows = stmt.query_map(params![scenario_id], |row| {
            Ok(SkillRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                source_type: row.get(3)?,
                source_ref: row.get(4)?,
                source_revision: row.get(5)?,
                central_path: row.get(6)?,
                content_hash: row.get(7)?,
                enabled: row.get::<_, i32>(8)? != 0,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                status: row.get(11)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn count_skills_for_scenario(&self, scenario_id: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM scenario_skills WHERE scenario_id = ?1",
            params![scenario_id],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn get_scenarios_for_skill(&self, skill_id: &str) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT scenario_id FROM scenario_skills WHERE skill_id = ?1",
        )?;
        let rows = stmt.query_map(params![skill_id], |row| row.get::<_, String>(0))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    // ── Active Scenario ──

    pub fn get_active_scenario_id(&self) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT scenario_id FROM active_scenario WHERE key = 'current'",
        )?;
        let mut rows = stmt.query_map([], |row| row.get::<_, Option<String>>(0))?;
        Ok(rows.next().and_then(|r| r.ok()).flatten())
    }

    pub fn set_active_scenario(&self, scenario_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO active_scenario (key, scenario_id) VALUES ('current', ?1)",
            params![scenario_id],
        )?;
        Ok(())
    }
}
