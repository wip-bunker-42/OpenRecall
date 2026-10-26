CREATE TABLE training_runs(id TEXT PRIMARY KEY,profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,payload TEXT NOT NULL);
CREATE INDEX training_profile ON training_runs(profile_id);
