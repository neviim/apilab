use std::collections::HashMap;
use std::sync::RwLock;

use super::state::Session;

pub struct SessionManager {
    sessions: RwLock<HashMap<String, Session>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }

    pub fn create(&self, session: Session) -> String {
        let id = session.id.clone();
        self.sessions.write().unwrap().insert(id.clone(), session);
        id
    }

    pub fn with<F, R>(&self, id: &str, f: F) -> Option<R>
    where
        F: FnOnce(&Session) -> R,
    {
        self.sessions.read().unwrap().get(id).map(f)
    }

    pub fn with_mut<F, R>(&self, id: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut Session) -> R,
    {
        self.sessions.write().unwrap().get_mut(id).map(f)
    }

    pub fn destroy(&self, id: &str) -> bool {
        self.sessions.write().unwrap().remove(id).is_some()
    }
}
