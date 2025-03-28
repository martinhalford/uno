use super::game::UnoGame;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct GameSession {
    pub id: String,
    pub game: UnoGame,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl GameSession {
    pub fn new(id: String, game: UnoGame) -> Self {
        Self {
            id,
            game,
            last_updated: chrono::Utc::now(),
        }
    }

    pub fn save(&self, sessions_dir: &PathBuf) -> std::io::Result<()> {
        let session_path = sessions_dir.join(format!("{}.json", self.id));
        let json = serde_json::to_string_pretty(self)?;
        fs::write(session_path, json)
    }

    pub fn load(id: &str, sessions_dir: &PathBuf) -> std::io::Result<Self> {
        let session_path = sessions_dir.join(format!("{}.json", id));
        let json = fs::read_to_string(session_path)?;
        let mut session: Self = serde_json::from_str(&json)?;
        session.last_updated = chrono::Utc::now();
        Ok(session)
    }
}

pub struct SessionManager {
    pub sessions_dir: PathBuf,
}

impl SessionManager {
    pub fn new(sessions_dir: PathBuf) -> std::io::Result<Self> {
        fs::create_dir_all(&sessions_dir)?;
        Ok(Self { sessions_dir })
    }

    pub fn create_session(&self, game: UnoGame) -> std::io::Result<GameSession> {
        let id = uuid::Uuid::new_v4().to_string();
        let session = GameSession::new(id.clone(), game);
        session.save(&self.sessions_dir)?;
        Ok(session)
    }

    pub fn load_session(&self, id: &str) -> std::io::Result<GameSession> {
        GameSession::load(id, &self.sessions_dir)
    }

    pub fn list_sessions(&self) -> std::io::Result<Vec<String>> {
        let mut sessions = Vec::new();
        for entry in fs::read_dir(&self.sessions_dir)? {
            let entry = entry?;
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".json") {
                    sessions.push(file_name.trim_end_matches(".json").to_string());
                }
            }
        }
        Ok(sessions)
    }

    pub fn delete_session(&self, id: &str) -> std::io::Result<()> {
        let session_path = self.sessions_dir.join(format!("{}.json", id));
        fs::remove_file(session_path)
    }
}
