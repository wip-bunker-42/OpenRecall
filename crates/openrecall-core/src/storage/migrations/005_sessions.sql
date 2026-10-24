CREATE TABLE sessions(id TEXT PRIMARY KEY,deck_id TEXT NOT NULL REFERENCES decks(id) ON DELETE CASCADE,status TEXT NOT NULL,payload TEXT NOT NULL);
CREATE INDEX sessions_active ON sessions(deck_id,status);
