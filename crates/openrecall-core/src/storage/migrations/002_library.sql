CREATE TABLE profiles(id TEXT PRIMARY KEY, payload TEXT NOT NULL);
CREATE TABLE models(id TEXT PRIMARY KEY, payload TEXT NOT NULL);
CREATE TABLE decks(id TEXT PRIMARY KEY, profile_id TEXT NOT NULL REFERENCES profiles(id), payload TEXT NOT NULL);
CREATE INDEX decks_profile ON decks(profile_id);
