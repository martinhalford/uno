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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn create_test_session_manager() -> (SessionManager, tempfile::TempDir) {
        let temp_dir = tempdir().unwrap();
        let manager = SessionManager::new(temp_dir.path().to_path_buf()).unwrap();
        (manager, temp_dir)
    }

    #[test]
    fn test_create_and_load_session() {
        let (manager, _temp_dir) = create_test_session_manager();
        let player_names = vec!["Alice".to_string(), "Bob".to_string()];
        let game = UnoGame::new(player_names).unwrap();

        let session = manager.create_session(game).unwrap();
        assert!(!session.id.is_empty());

        let loaded = manager.load_session(&session.id).unwrap();
        assert_eq!(loaded.id, session.id);
        assert_eq!(loaded.game.players.len(), 2);
    }

    #[test]
    fn test_list_sessions() {
        let (manager, _temp_dir) = create_test_session_manager();
        let player_names = vec!["Alice".to_string(), "Bob".to_string()];
        let game = UnoGame::new(player_names).unwrap();

        let session1 = manager.create_session(game).unwrap();
        let session2 = manager
            .create_session(UnoGame::new(vec!["Charlie".to_string(), "David".to_string()]).unwrap())
            .unwrap();

        let sessions = manager.list_sessions().unwrap();
        assert_eq!(sessions.len(), 2);
        assert!(sessions.contains(&session1.id));
        assert!(sessions.contains(&session2.id));
    }

    #[test]
    fn test_delete_session() {
        let (manager, _temp_dir) = create_test_session_manager();
        let player_names = vec!["Alice".to_string(), "Bob".to_string()];
        let game = UnoGame::new(player_names).unwrap();

        let session = manager.create_session(game).unwrap();
        assert!(manager.delete_session(&session.id).is_ok());

        let sessions = manager.list_sessions().unwrap();
        assert!(!sessions.contains(&session.id));
    }

    #[test]
    fn test_session_persistence() {
        let (manager, _temp_dir) = create_test_session_manager();
        let player_names = vec!["Alice".to_string(), "Bob".to_string()];
        let game = UnoGame::new(player_names).unwrap();

        let session = manager.create_session(game).unwrap();
        let session_path = manager.sessions_dir.join(format!("{}.json", session.id));

        assert!(session_path.exists());
        let contents = fs::read_to_string(session_path).unwrap();
        assert!(contents.contains("Alice"));
        assert!(contents.contains("Bob"));
    }
}
