CREATE TABLE cards(id TEXT PRIMARY KEY, deck_id TEXT NOT NULL REFERENCES decks(id) ON DELETE CASCADE, due_ms INTEGER NOT NULL, created_ms INTEGER NOT NULL, review_count INTEGER NOT NULL, payload TEXT NOT NULL);
CREATE INDEX cards_due ON cards(deck_id,due_ms,created_ms,id);
CREATE INDEX cards_new ON cards(deck_id,review_count,due_ms);
