CREATE TABLE reviews(id TEXT PRIMARY KEY,card_id TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,deck_id TEXT NOT NULL REFERENCES decks(id) ON DELETE CASCADE,session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,at_ms INTEGER NOT NULL,model_revision TEXT NOT NULL REFERENCES models(id),payload TEXT NOT NULL);
CREATE INDEX reviews_card_time ON reviews(card_id,at_ms);
CREATE INDEX reviews_deck_time ON reviews(deck_id,at_ms);
CREATE TABLE actions(id TEXT PRIMARY KEY,session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,payload TEXT NOT NULL);
CREATE TABLE undo_records(id TEXT PRIMARY KEY,session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,payload TEXT NOT NULL);
