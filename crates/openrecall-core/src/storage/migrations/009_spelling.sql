ALTER TABLE decks ADD COLUMN kind TEXT NOT NULL DEFAULT 'standard'
    CHECK(kind IN ('standard','spelling') AND kind=coalesce(json_extract(payload,'$.kind'),'standard'));
CREATE UNIQUE INDEX one_spelling_deck ON decks(kind) WHERE kind='spelling';
CREATE INDEX cards_created ON cards(deck_id,created_ms,id);
CREATE TRIGGER deck_kind_immutable BEFORE UPDATE OF kind ON decks
    WHEN NEW.kind<>OLD.kind BEGIN SELECT RAISE(ABORT,'immutable deck kind'); END;

CREATE TABLE spelling_configs(
    deck_id TEXT PRIMARY KEY REFERENCES decks(id) ON DELETE CASCADE,
    source_deck_id TEXT REFERENCES decks(id) ON DELETE SET NULL,
    source_deck_name TEXT,
    extraction_policy_version INTEGER NOT NULL,
    revision TEXT NOT NULL
);
CREATE TABLE spelling_words(
    card_id TEXT PRIMARY KEY REFERENCES cards(id) ON DELETE CASCADE,
    deck_id TEXT NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
    canonical TEXT NOT NULL,
    display TEXT NOT NULL,
    UNIQUE(deck_id,canonical)
);
CREATE TABLE spelling_snapshots(
    id TEXT PRIMARY KEY,
    spelling_deck_id TEXT NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
    content_sha256 TEXT NOT NULL,
    payload TEXT NOT NULL CHECK(id IS json_extract(payload,'$.id')
        AND spelling_deck_id IS json_extract(payload,'$.spellingDeckId')
        AND content_sha256 IS json_extract(payload,'$.contentSha256')),
    UNIQUE(spelling_deck_id,content_sha256)
);
CREATE TRIGGER spelling_snapshot_immutable BEFORE UPDATE ON spelling_snapshots
    BEGIN SELECT RAISE(ABORT,'immutable source snapshot'); END;
CREATE TABLE spelling_origins(
    spelling_card_id TEXT NOT NULL REFERENCES spelling_words(card_id) ON DELETE CASCADE,
    source_card_id TEXT NOT NULL,
    source_form_id TEXT NOT NULL,
    snapshot_id TEXT NOT NULL REFERENCES spelling_snapshots(id) DEFERRABLE INITIALLY DEFERRED,
    first_seen_at TEXT NOT NULL,
    PRIMARY KEY(spelling_card_id,source_card_id,source_form_id)
);
CREATE INDEX spelling_origins_source ON spelling_origins(source_card_id,source_form_id);
CREATE INDEX spelling_origins_snapshot ON spelling_origins(snapshot_id);
CREATE TABLE spelling_exclusions(
    deck_id TEXT NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
    canonical TEXT NOT NULL,
    excluded_at TEXT NOT NULL,
    PRIMARY KEY(deck_id,canonical)
);
