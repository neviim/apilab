use std::time::Instant;

pub struct Session {
    pub id: String,
    pub protocol_version: String,
    pub initialized: bool,
    pub created_at: Instant,
    pub last_active: Instant,
}

impl Session {
    pub fn new(id: String, protocol_version: String) -> Self {
        let now = Instant::now();
        Self {
            id,
            protocol_version,
            initialized: false,
            created_at: now,
            last_active: now,
        }
    }

    pub fn touch(&mut self) {
        self.last_active = Instant::now();
    }
}
