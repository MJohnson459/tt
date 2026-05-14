use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: u64,
    pub client: String,
    pub start: DateTime<Local>,
    pub end: Option<DateTime<Local>>,
    pub note: Option<String>,
    pub rate: f64,
}

impl Session {
    pub fn duration_hours(&self) -> f64 {
        let end = self.end.unwrap_or_else(Local::now);
        (end - self.start).num_seconds() as f64 / 3600.0
    }

    pub fn earnings(&self) -> f64 {
        self.duration_hours() * self.rate
    }

    pub fn is_active(&self) -> bool {
        self.end.is_none()
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Store {
    pub sessions: Vec<Session>,
    pub clients: HashMap<String, f64>,
    pub default_client: Option<String>,
    pub next_id: u64,
}

impl Store {
    pub fn load() -> anyhow::Result<Self> {
        let path = data_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = data_path()?;
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(&path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn active_session(&self) -> Option<&Session> {
        self.sessions.iter().find(|s| s.is_active())
    }

    pub fn active_session_mut(&mut self) -> Option<&mut Session> {
        self.sessions.iter_mut().find(|s| s.is_active())
    }

    pub fn new_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

pub fn data_path() -> anyhow::Result<PathBuf> {
    let base = std::env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").expect("$HOME not set"))
                .join(".local")
                .join("share")
        });
    Ok(base.join("timetracker").join("data.json"))
}
