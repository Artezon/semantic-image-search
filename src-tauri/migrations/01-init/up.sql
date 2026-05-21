CREATE TABLE IF NOT EXISTS file (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    path             TEXT NOT NULL UNIQUE,
    type             TEXT NOT NULL CHECK (type IN ('DOC', 'IMG', 'VID', 'AUD'))
);

CREATE TABLE IF NOT EXISTS file_metadata (
    file_id          INTEGER NOT NULL REFERENCES file(id) ON DELETE CASCADE,
    meta_key         TEXT NOT NULL,
    meta_value       TEXT NOT NULL,
    PRIMARY KEY (file_id, meta_key)
);

CREATE TABLE IF NOT EXISTS emb_type (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    kind             TEXT NOT NULL,
    model_name       TEXT NOT NULL,
    CONSTRAINT       uq_emb_kind_model_name UNIQUE (kind, model_name)
);

CREATE TABLE IF NOT EXISTS emb_metadata (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    file_id          INTEGER NOT NULL REFERENCES file(id) ON DELETE CASCADE,
    emb_type_id      INTEGER NOT NULL REFERENCES emb_type(id) ON DELETE RESTRICT,
    last_file_mtime  INTEGER NOT NULL,
    last_file_size   INTEGER NOT NULL,
    indexed_at       INTEGER NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_emb_metadata_file_id_emb_type_id ON emb_metadata(file_id, emb_type_id);
CREATE INDEX IF NOT EXISTS idx_emb_metadata_emb_type_id ON emb_metadata(emb_type_id);
