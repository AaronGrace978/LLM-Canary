use std::fs;
use std::path::{Path, PathBuf};

use crate::models::{merge_providers, Db};

pub fn db_path(dir: &Path) -> PathBuf {
    dir.join("db.json")
}

pub fn load(dir: &Path) -> Db {
    let path = db_path(dir);
    let mut db = if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<Db>(&s).ok())
            .unwrap_or_else(empty_db)
    } else {
        empty_db()
    };
    merge_providers(&mut db.providers);
    db
}

pub fn save(dir: &Path, db: &Db) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let path = db_path(dir);
    let tmp = dir.join("db.json.tmp");
    let bytes = serde_json::to_vec_pretty(db).map_err(|e| e.to_string())?;
    fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
    fs::rename(&tmp, path).map_err(|e| e.to_string())
}

fn empty_db() -> Db {
    Db {
        canaries: vec![],
        providers: crate::models::default_providers(),
        probes: vec![],
    }
}
