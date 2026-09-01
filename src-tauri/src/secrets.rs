use keyring::Entry;

use crate::models::Db;

const SERVICE: &str = "com.llmcanary.app";
const GITHUB_USER: &str = "github-token";

fn entry(user: &str) -> Result<Entry, String> {
    Entry::new(SERVICE, user).map_err(|e| format!("credential vault: {e}"))
}

fn get_secret(user: &str) -> String {
    entry(user)
        .ok()
        .and_then(|e| e.get_password().ok())
        .unwrap_or_default()
}

fn set_secret(user: &str, value: &str) -> Result<(), String> {
    let e = entry(user)?;
    if value.is_empty() {
        match e.delete_credential() {
            Ok(()) => Ok(()),
            Err(err) => {
                let msg = err.to_string().to_lowercase();
                if msg.contains("no entry")
                    || msg.contains("not found")
                    || msg.contains("no password")
                    || msg.contains("not exist")
                {
                    Ok(())
                } else {
                    Err(format!("credential vault: {err}"))
                }
            }
        }
    } else {
        e.set_password(value)
            .map_err(|e| format!("credential vault: {e}"))
    }
}

fn provider_user(id: &str) -> String {
    format!("provider:{id}")
}

/// Load secrets from the OS vault. If `db.json` still has plaintext keys
/// (pre-0.4.1 installs), copy them into the vault so the next save can wipe them.
pub fn hydrate(db: &mut Db) {
    for p in &mut db.providers {
        if p.api_key.trim().is_empty() {
            p.api_key = get_secret(&provider_user(&p.id));
        } else {
            let _ = set_secret(&provider_user(&p.id), p.api_key.trim());
        }
    }
    if db.github_token.trim().is_empty() {
        db.github_token = get_secret(GITHUB_USER);
    } else {
        let _ = set_secret(GITHUB_USER, db.github_token.trim());
    }
}

pub fn persist_secrets(db: &Db) -> Result<(), String> {
    for p in &db.providers {
        set_secret(&provider_user(&p.id), p.api_key.trim())?;
    }
    set_secret(GITHUB_USER, db.github_token.trim())?;
    Ok(())
}

pub fn strip_for_disk(db: &Db) -> Db {
    let mut disk = db.clone();
    for p in &mut disk.providers {
        p.api_key.clear();
    }
    disk.github_token.clear();
    disk
}
